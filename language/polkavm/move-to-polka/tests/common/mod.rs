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

pub fn build_move_program(move_file_path: &str) -> anyhow::Result<Vec<u8>> {
    let mut move_compile_options = Options::default();
    move_compile_options.compile = true;
    move_compile_options.sources = vec![move_file_path.to_string()];
    // parse move source files
    let mut color_writer = create_colored_stdout();
    let move_env = get_env_from_source(&mut color_writer, &move_compile_options)?;

    // a little bit clumsy but does the trick for now
    let path = Path::new(move_file_path);
    let filename = path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("cant get filename from {move_file_path}"))?;

    let mut filename = OsString::from(filename);
    filename.push(".o");

    let output = Path::new("output").join(filename);
    let output_string = output
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("can't get string from path"))?
        .to_string();

    println!("{move_file_path} -> {output_string}");
    // translate to riscV object file
    let mut llvm_translate_options = Options::default();
    llvm_translate_options.compile = true; // we don't need linking stage
    llvm_translate_options.output = output_string;
    llvm_translate_options.llvm_ir = false;
    compile(&move_env, &llvm_translate_options)?;

    //TODO it would be so nice if compile won't access FS directly so we can work purely in-memory
    let data = std::fs::read(output)?;
    Ok(data)
}
