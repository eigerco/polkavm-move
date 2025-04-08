use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use move_to_polka::{compile, get_env_from_source, options::Options};
use polkavm::{Config, Engine, Linker, Module, ProgramBlob};

#[test]
pub fn test_program_execution() -> anyhow::Result<()> {
    env_logger::init();

    let mut move_compile_options = Options::default();
    move_compile_options.compile = true;
    move_compile_options.sources = vec!["../examples/basic/sources/morebasic.move".to_string()];
    // parse move source files
    let mut color_writer = create_colored_stdout();
    let move_env = get_env_from_source(&mut color_writer, &move_compile_options)?;

    // translate to riscV object file
    let mut llvm_translate_options = Options::default();
    llvm_translate_options.compile = true; // we don't need linking stage
    llvm_translate_options.output = "output/morebasic.o".to_string();
    llvm_translate_options.llvm_ir = false;
    compile(&move_env, &llvm_translate_options)?;

    // TODO (polkatool liking phase)

    //TODO(tadas): this should be replaced with full compilation process from move sources, polka linker etc.
    let program_bytes = include_bytes!("../../../../output/morebasic.polkavm");

    let blob =
        ProgramBlob::parse(program_bytes[..].into()).map_err(|e| anyhow::anyhow!("{e:?}"))?;

    let config = Config::from_env()?;
    let engine = Engine::new(&config)?;
    let module = Module::from_blob(&engine, &Default::default(), blob)?;

    let linker: Linker = Linker::new();

    // Link the host functions with the module.
    let instance_pre = linker.instantiate_pre(&module)?;

    // Instantiate the module.
    let mut instance = instance_pre.instantiate()?;

    // Grab the function and call it.
    println!("Calling into the guest program (high level):");
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut (), "sum", (1, 10))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 11);

    Ok(())
}

fn create_colored_stdout() -> StandardStream {
    let color = if atty::is(atty::Stream::Stderr) && atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };
    StandardStream::stderr(color)
}
