use polkavm::{Config, Engine, Linker, Module, ProgramBlob};

mod common;
use common::*;

#[test]
pub fn test_morebasic_program_execution() -> anyhow::Result<()> {
    env_logger::init();

    let move_byte_code = build_move_program("../examples/basic/sources/morebasic.move")?;

    // polka tool linking phase
    let program_bytes = load_from_elf_with_polka_linker(&move_byte_code)?;

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

#[test]
#[ignore = "doesnt work yet"]
pub fn test_basic_program_execution() -> anyhow::Result<()> {
    env_logger::init();

    let move_byte_code = build_move_program("../examples/basic/sources/basic.move")?;

    // polka tool linking phase
    let program_bytes = load_from_elf_with_polka_linker(&move_byte_code)?;

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
