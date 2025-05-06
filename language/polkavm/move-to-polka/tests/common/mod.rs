use std::{fs, path::Path};

use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use move_to_polka::{compile, get_env_from_source, options::Options};
use polkavm::ProgramBlob;

pub fn create_colored_stdout() -> StandardStream {
    let color = if atty::is(atty::Stream::Stderr) && atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };
    StandardStream::stderr(color)
}

pub fn load_from_elf_with_polka_linker(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    // config is taken from polkatool with default values
    let mut config = polkavm_linker::Config::default();
    config.set_strip(false);
    config.set_optimize(true);

    let res = polkavm_linker::program_from_elf(config, data)?;
    Ok(res)
}

pub fn parse_to_blob(program_bytes: &[u8]) -> anyhow::Result<ProgramBlob> {
    ProgramBlob::parse(program_bytes.into()).map_err(|e| anyhow::anyhow!("{e:?}"))
}

pub struct BuildOptions {
    options: Options,
    output_file: String,
}

impl BuildOptions {
    pub fn new(output_file: &str) -> Self {
        let mut options = Options::default();
        options.compile = true;
        Self {
            options,
            output_file: output_file.to_string(),
        }
    }

    pub fn source(mut self, source_file: &str) -> Self {
        self.options.sources.push(source_file.to_string());
        self
    }

    pub fn address_mapping(mut self, mapping: &str) -> Self {
        self.options.named_address_mapping.push(mapping.to_string());
        self
    }

    pub fn dependency(mut self, dependency_path: &str) -> Self {
        self.options.dependencies.push(dependency_path.to_string());
        self
    }
}

pub fn build_move_program(options: BuildOptions) -> anyhow::Result<Vec<u8>> {
    // parse move source files
    let mut color_writer = create_colored_stdout();
    let move_env = get_env_from_source(&mut color_writer, &options.options)?;
    let output_file = &options.output_file;
    println!("Object file output -> {output_file}");
    create_dir_all(output_file)?;
    // translate to riscV object file
    let mut llvm_translate_options = Options::default();
    llvm_translate_options.compile = true; // we don't need linking stage
    llvm_translate_options.output = output_file.to_string();
    llvm_translate_options.llvm_ir = false;
    compile(&move_env, &llvm_translate_options)?;

    //TODO it would be so nice if compile won't access FS directly so we can work purely in-memory
    let data = std::fs::read(output_file)?;
    Ok(data)
}
pub const MOVE_STDLIB_PATH: &str = env!("MOVE_STDLIB_PATH");

pub fn create_dir_all<T: AsRef<Path>>(path: T) -> anyhow::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}
