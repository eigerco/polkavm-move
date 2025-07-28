// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cstr;
pub mod linker;
pub mod native;
pub mod options;
pub mod stackless;

use crate::options::Options;

use anyhow::Context;
use codespan_reporting::term::termcolor::WriteColor;
use itertools::Itertools;
use linker::load_from_elf_with_polka_linker;
use log::{debug, Level, LevelFilter};
use move_binary_format::{
    binary_views::BinaryIndexedView, file_format::CompiledScript, CompiledModule,
};
use move_bytecode_source_map::{
    mapping::SourceMapping, source_map::SourceMap, utils::source_map_from_file,
};
use move_command_line_common::files::{
    FileHash, MOVE_COMPILED_EXTENSION, MOVE_EXTENSION, SOURCE_MAP_EXTENSION,
};
use move_compiler_v2::{run_move_compiler, Experiment, Options as CompilerV2Options};
use move_ir_types::location::Spanned;
use move_model::{
    model::{GlobalEnv, ModuleId, MoveIrLoc},
    parse_addresses_from_options,
};
use std::{
    fs::{self},
    io::Write,
    iter::once,
    path::{Path, PathBuf},
};

// init logger from RUST_LOG env var, defaults to INFO
pub fn initialize_logger() {
    static LOGGER_INIT: std::sync::Once = std::sync::Once::new();
    LOGGER_INIT.call_once(|| {
        use anstyle::{AnsiColor, Color};
        env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .parse_default_env()
            .format(|formatter, record| {
                let level = record.level();
                let style = formatter.default_level_style(level);
                match record.level() {
                    Level::Error => style.fg_color(Some(Color::Ansi(AnsiColor::Red))),
                    Level::Warn => style.fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
                    Level::Info => style.fg_color(Some(Color::Ansi(AnsiColor::Green))),
                    Level::Debug => style.fg_color(Some(Color::Ansi(AnsiColor::Blue))),
                    Level::Trace => style.fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
                };
                writeln!(
                    formatter,
                    "[{} {}:{}] {}",
                    level,
                    record.target(),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();
    });
}

fn link_object_files(
    out_path: PathBuf,
    objects: &[PathBuf],
    polka_object_file: PathBuf,
    move_native_path: Option<&str>,
) -> anyhow::Result<PathBuf> {
    log::debug!("link_object_files");

    let lld = build_tools::Lld::try_init()?;

    let native_lib_content = native::move_native_lib_content();

    let move_native = if let Some(move_native) = move_native_path {
        // if passed explicitly through args - use that
        PathBuf::from(move_native)
    } else {
        let move_native = out_path.join("move_native.o");
        std::fs::write(&move_native, native_lib_content)?;
        move_native
    };

    debug!("Native lib available at: {move_native:?}");

    let merged_object = out_path.join("merged.o");
    lld.merge_object_files(
        &objects.iter().chain(once(&move_native)).collect_vec(),
        &merged_object,
        true,
    )?;
    debug!("Merged object file created at: {}", merged_object.display());

    let object_bytes = std::fs::read(&merged_object)?;
    debug!(
        "Read merged object file bytes, size: {}",
        object_bytes.len()
    );
    let polka_object = load_from_elf_with_polka_linker(&object_bytes)?;
    debug!("Polka object created, size: {}", polka_object.len());
    std::fs::write(&polka_object_file, &polka_object)?;
    debug!(
        "Polka object file written to: {}",
        polka_object_file.display()
    );
    Ok(polka_object_file)
}

pub fn get_env_from_source<W: WriteColor>(
    error_writer: &mut W,
    options: &Options,
) -> anyhow::Result<GlobalEnv> {
    let addrs = parse_addresses_from_options(options.named_address_mapping.clone())?;
    debug!("Named addresses {addrs:?}");

    let mut v2_options = CompilerV2Options {
        sources: options.sources.clone(),
        dependencies: options.dependencies.clone(),
        named_address_mapping: options.named_address_mapping.clone(),
        output_dir: options.output.clone(),
        whole_program: true,
        ..Default::default()
    };

    v2_options = v2_options.set_experiment(Experiment::SPEC_REWRITE, true);
    v2_options = v2_options.set_experiment(Experiment::ATTACH_COMPILED_MODULE, true);
    let mut emitter = v2_options.error_emitter(error_writer);
    let (env, _units) = run_move_compiler(emitter.as_mut(), v2_options)?;
    env.treat_everything_as_target(false);

    if env.has_errors() {
        anyhow::bail!("Move source code errors")
    } else {
        Ok(env)
    }
}

fn get_env_from_bytecode(options: &Options) -> anyhow::Result<GlobalEnv> {
    let move_extension = MOVE_EXTENSION;
    let mv_bytecode_extension = MOVE_COMPILED_EXTENSION;
    let source_map_extension = SOURCE_MAP_EXTENSION;

    let bytecode_file_path = (options.bytecode_file_path.as_ref()).unwrap();
    let source_path = Path::new(&bytecode_file_path);
    let extension = source_path
        .extension()
        .context("Missing file extension for bytecode file")?;
    if extension != mv_bytecode_extension {
        anyhow::bail!(
            "Bad source file extension {:?}; expected {}",
            extension,
            mv_bytecode_extension
        );
    }

    let bytecode_bytes = fs::read(bytecode_file_path).context("Unable to read bytecode file")?;

    let mut dep_bytecode_bytes = vec![];
    for dep in &options.dependencies {
        let bytes = fs::read(dep).context("Unable to read dependency bytecode file {dep}")?;
        dep_bytecode_bytes.push(bytes);
    }

    let source_path = Path::new(&bytecode_file_path).with_extension(move_extension);
    let source = fs::read_to_string(&source_path).ok();
    let source_map =
        source_map_from_file(&Path::new(&bytecode_file_path).with_extension(source_map_extension));

    let no_loc = Spanned::unsafe_no_loc(()).loc;
    let module: CompiledModule;
    let script: CompiledScript;
    let bytecode = if options.is_script {
        script = CompiledScript::deserialize(&bytecode_bytes)
            .context("Script blob can't be deserialized")?;
        BinaryIndexedView::Script(&script)
    } else {
        module = CompiledModule::deserialize(&bytecode_bytes)
            .context("Module blob can't be deserialized")?;
        BinaryIndexedView::Module(&module)
    };

    let mut source_mapping = {
        if let Ok(s) = source_map {
            SourceMapping::new(s, bytecode)
        } else {
            SourceMapping::new_from_view(bytecode, no_loc)
                .context("Unable to build dummy source mapping")?
        }
    };

    if let Some(source_code) = source {
        source_mapping.with_source_code((source_path.to_str().unwrap().to_string(), source_code));
    }

    let main_move_module = if options.is_script {
        let script = CompiledScript::deserialize(&bytecode_bytes)
            .context("Script blob can't be deserialized")?;
        move_model::script_into_module(script, "main")
    } else {
        CompiledModule::deserialize(&bytecode_bytes).context("Module blob can't be deserialized")?
    };

    let mut dep_move_modules = vec![];

    for bytes in &dep_bytecode_bytes {
        let dep_module = CompiledModule::deserialize(bytes)
            .context("Dependency module blob can't be deserialized")?;
        dep_move_modules.push(dep_module);
    }

    let modules = dep_move_modules
        .into_iter()
        .chain(Some(main_move_module))
        .collect::<Vec<_>>();

    run_bytecode_model_builder(&modules)
}

pub fn compile(global_env: &GlobalEnv, options: &Options) -> anyhow::Result<()> {
    use crate::stackless::{extensions::ModuleEnvExt, *};

    let tgt_platform = TargetPlatform::PVM;
    tgt_platform.initialize_llvm();
    let lltarget = Target::from_triple(tgt_platform.triple())?;
    let llmachine = lltarget.create_target_machine(
        tgt_platform.triple(),
        tgt_platform.llvm_cpu(),
        tgt_platform.llvm_features(),
        &options.opt_level,
    );
    let global_cx = GlobalContext::new(global_env, tgt_platform, &llmachine);
    let output_file_path = options.output.clone();
    let file_stem = Path::new(&output_file_path).file_stem().unwrap();
    // If building a shared object library, then -o is the
    // directory to output the compiled modules, each module
    // 'mod' will get file name 'mod.o'
    let out_path = Path::new(&output_file_path)
        .parent()
        .unwrap()
        .to_path_buf()
        .join(file_stem);
    if !(options.compile || options.llvm_ir) {
        fs::create_dir_all(&out_path)
            .or_else(|err| anyhow::bail!("Error creating directory: {}", err))?;
    }
    let mut objects = vec![];

    // Deserialization is only for one (the last) module.
    let skip_cnt = if options.bytecode_file_path.is_some() {
        global_env.get_modules().count() - 1
    } else {
        0
    };
    // Keep a list of exported functions to avoid generating the polkaVM sections multiple times.
    let mut exports: Vec<String> = vec![];
    // Note: don't reverse order of modules, since DI may be inter module dependent and needs the direct order.
    for mod_id in global_env
        .get_modules()
        .collect::<Vec<_>>()
        .iter()
        .skip(skip_cnt)
        .map(|m| m.get_id())
    {
        let module = global_env.get_module(mod_id);
        let modname = module.llvm_module_name();
        debug!("--------------------------------------");
        debug!("Generating code for module {modname}");
        let llmod = global_cx.llvm_cx.create_module(&modname);
        let module_source_path = module.get_source_path().to_str().expect("utf-8");
        let mod_cx =
            &mut global_cx.create_module_context(mod_id, &llmod, options, module_source_path);
        mod_cx.translate(&mut exports);

        let mut out_path = out_path.join(&modname);
        out_path.set_extension(&options.output_file_extension);
        let mut output_file = out_path.to_str().unwrap().to_string();
        // llmod is moved and dropped in both branches of this
        // if-then-else when the module is written to a file.
        if options.llvm_ir {
            output_file = options.output.clone();
            let path = Path::new(&output_file);
            if path.exists() && path.is_dir() {
                let mut path = path.join(modname);
                path.set_extension(&options.output_file_extension);
                output_file = path.to_string_lossy().to_string();
            }
            llmod.write_to_file(options.llvm_ir, &output_file)?;
        } else {
            if options.compile {
                output_file = options.output.clone();
            }
            write_object_file(llmod, &llmachine, &output_file)?;
        }
        if !(options.compile || options.llvm_ir) {
            objects.push(Path::new(&output_file).to_path_buf());
        }
    }
    if !(options.compile || options.llvm_ir) {
        link_object_files(
            out_path,
            objects.as_slice(),
            Path::new(&output_file_path).to_path_buf(),
            options.move_native_archive.as_deref(),
        )?;
    }
    Ok(())
}

pub fn run_to_polka<W: WriteColor>(error_writer: &mut W, options: Options) -> anyhow::Result<()> {
    // Normally the compiler is invoked on a package from `move build`
    // coomand, and builds an entire package as a .so file.  The test
    // harness is currently designed to invoke stand-alone compiler
    // tool on individual Move bytecode files, compiling each to a .o
    // file. To build a .so file loadable into a VM, it's necessary to
    // link the separate .o files into a .so file.  If all input files
    // are .o object files, the compiler assumes that it should link
    // them into an output .so file.
    if !options.llvm_ir
        && !options.compile
        && options.bytecode_file_path.is_none()
        && options.sources.iter().all(|s| s.ends_with(".o"))
    {
        let output = Path::new(&options.output).to_path_buf();
        let objects: Vec<PathBuf> = options
            .sources
            .iter()
            .map(|s| Path::new(s).to_path_buf())
            .collect();
        link_object_files(
            output.parent().unwrap().to_path_buf(),
            objects.as_slice(),
            output,
            options.move_native_archive.as_deref(),
        )?;
        return Ok(());
    }
    match &*options.gen_dot_cfg {
        "write" | "view" | "" => {}
        _ => {
            eprintln!(
                "unexpected gen-dot-cfg option '{}', ignored.",
                &options.gen_dot_cfg
            );
        }
    };

    let global_env: GlobalEnv = if options.bytecode_file_path.is_some() {
        get_env_from_bytecode(&options)?
    } else {
        get_env_from_source(error_writer, &options)?
    };

    compile(&global_env, &options)?;

    Ok(())
}

/// Build a `GlobalEnv` from a collection of `CompiledModule`'s. The `modules` list must be
/// topologically sorted by the dependency relation (i.e., a child node in the dependency graph
/// should appear earlier in the vector than its parents).
pub fn run_bytecode_model_builder<'a>(
    modules: impl IntoIterator<Item = &'a CompiledModule>,
) -> anyhow::Result<GlobalEnv> {
    let mut env = GlobalEnv::new();
    for (i, m) in modules.into_iter().enumerate() {
        let module_id = ModuleId::new(i);
        env.attach_compiled_module(
            module_id,
            m.clone(),
            SourceMap::new(MoveIrLoc::new(FileHash::empty(), 0, 0), None),
        );
    }
    Ok(env)
}
