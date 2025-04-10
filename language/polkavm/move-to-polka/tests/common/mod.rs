use std::{ffi::OsString, path::Path};

use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use move_to_polka::{compile, get_env_from_source, options::Options};

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

pub fn resolve_move_std_lib_sources() -> String {
    //TODO(tadas): Right now we expect that move-stdlib comes from move-on-aptos checked out on the same root level as this project itself
    //BUT since we depend on some rust crates coming from move-on-aptos repo, repo itself is available as cargo checkouts dir
    //it would be awesome to:
    // 1. scan cargo lock (cargo_metadata crate?) to find exact revision of move-on-aptos
    // 2. lookup cargo home dir according to rules (respect CARGO_HOME , default to HOME/.cargo if not set)
    // lookup actual move-on-aptos dependency and navigate to move-stdlib sources!
    "../../../../move-on-aptos/language/move-stdlib/sources".to_string()
}
