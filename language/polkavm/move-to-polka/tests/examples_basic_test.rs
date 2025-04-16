use polkavm::{Config, Engine, Linker, Module, ProgramBlob};

mod common;
use common::*;

#[test]
pub fn test_morebasic_program_execution() -> anyhow::Result<()> {
    env_logger::init();

    let build_options =
        BuildOptions::new("output/morebasic.o").source("../examples/basic/sources/morebasic.move");

    let move_byte_code = build_move_program(build_options)?;
    let program_bytes = load_from_elf_with_polka_linker(&move_byte_code)?;
    let blob = parse_to_blob(&program_bytes)?;

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
#[ignore = "doesnt work yet - need further push LLVM implementation"]
pub fn test_basic_program_execution() -> anyhow::Result<()> {
    let build_options = BuildOptions::new("output/basic.o")
        .source("../examples/basic/sources/basic.move")
        .dependency(&resolve_move_std_lib_sources())
        .address_mapping("std=0x1");

    let move_byte_code = build_move_program(build_options)?;
    let program_bytes = load_from_elf_with_polka_linker(&move_byte_code)?;
    let blob = parse_to_blob(&program_bytes)?;

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
pub fn test_tuple_implementation() -> anyhow::Result<()> {
    let build_options =
        BuildOptions::new("output/tuple.o").source("../examples/basic/sources/tuple.move");

    let move_byte_code = build_move_program(build_options)?;
    let program_bytes = load_from_elf_with_polka_linker(&move_byte_code)?;
    let blob = parse_to_blob(&program_bytes)?;

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
        .call_typed_and_get_result::<u64, (u32, u64)>(&mut (), "add", (10, 5))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 15);

    Ok(())
}

#[test]
#[ignore = "need to add support for imports between modules"]
pub fn test_multi_module_call() -> anyhow::Result<()> {
    let build_options = BuildOptions::new("output/multi_module_call.o")
        .source("../examples/multi_module/sources/modules2.move")
        .address_mapping("multi_module=0x7");

    let move_byte_code = build_move_program(build_options)?;
    let program_bytes = load_from_elf_with_polka_linker(&move_byte_code)?;
    let blob = parse_to_blob(&program_bytes)?;

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
        .call_typed_and_get_result::<u64, (u32, u64)>(&mut (), "add", (10, 5))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 15);

    Ok(())
}
