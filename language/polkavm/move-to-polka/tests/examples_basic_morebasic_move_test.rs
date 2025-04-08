use polkavm::{Config, Engine, Linker, Module, ProgramBlob};

#[test]
pub fn test_program_execution() {
    env_logger::init();
    //TODO(tadas): this should be replaced with full compilation process from move sources, polka linker etc.
    let program_bytes = include_bytes!("../../../../output/morebasic.polkavm");

    let blob = ProgramBlob::parse(program_bytes[..].into()).unwrap();

    let config = Config::from_env().unwrap();
    let engine = Engine::new(&config).unwrap();
    let module = Module::from_blob(&engine, &Default::default(), blob).unwrap();

    let linker: Linker = Linker::new();

    // Link the host functions with the module.
    let instance_pre = linker.instantiate_pre(&module).unwrap();

    // Instantiate the module.
    let mut instance = instance_pre.instantiate().unwrap();

    // Grab the function and call it.
    println!("Calling into the guest program (high level):");
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut (), "sum", (1, 10))
        .unwrap();
    assert_eq!(result, 11)
}
