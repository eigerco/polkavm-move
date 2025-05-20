use log::info;
use move_to_polka::initialize_logger;
use polkavm::{CallError, Config, Engine, Instance, Linker, Module, ModuleConfig};

mod common;
use common::*;
use serial_test::serial;

#[test]
#[serial] // TODO: find the reason this needs to run serially on macOS
pub fn test_morebasic_program_execution() -> anyhow::Result<()> {
    let mut instance = build_instance(
        "output/morebasic.polkavm",
        "../examples/basic/sources/morebasic.move",
        Vec::new(),
    )?;
    // Grab the function and call it.
    info!("Calling into the guest program (high level):");
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut (), "sum", (1, 10))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 11);

    Ok(())
}

#[test]
#[serial]
pub fn test_basic_program_execution() -> anyhow::Result<()> {
    initialize_logger();
    let move_src = format!("{MOVE_STDLIB_PATH}/sources");
    let build_options = BuildOptions::new("output/basic.o")
        .source("../examples/basic/sources/basic.move")
        .dependency(&move_src)
        .address_mapping("std=0x1");

    let program_bytes = build_polka_from_move(build_options)?;
    let blob = parse_to_blob(&program_bytes)?;

    let config = Config::from_env()?;
    let engine = Engine::new(&config)?;

    let mut module_config = ModuleConfig::new();
    module_config.set_strict(true); // enforce module loading fail if not all host functions are provided

    let module = Module::from_blob(&engine, &module_config, blob)?;

    let mut linker: Linker<_, ProgramError> = Linker::new();

    linker.define_typed("debug_print", |ptr_to_type: u32, ptr_to_data: u32| {
        info!("debug_print called. type ptr: {ptr_to_type:x} Data ptr: {ptr_to_data:x}");
        Ok(())
    })?;

    linker.define_typed("abort", |code: u64| {
        Result::<(), _>::Err(ProgramError::Abort(code))
    })?;

    // Link the host functions with the module.
    let instance_pre = linker.instantiate_pre(&module)?;

    // Instantiate the module.
    let mut instance = instance_pre.instantiate()?;

    // Grab the function and call it.
    let result = instance
        .call_typed_and_get_result::<u64, ()>(&mut (), "bar", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 19);

    let result =
        instance.call_typed_and_get_result::<(), (u64,)>(&mut (), "abort_with_code", (42,));
    assert!(matches!(
        result,
        Err(CallError::User(ProgramError::Abort(42)))
    ));
    Ok(())
}

#[test]
#[serial]
pub fn test_tuple_implementation() -> anyhow::Result<()> {
    let mut instance = build_instance(
        "output/tuple.polkavm",
        "../examples/basic/sources/tuple.move",
        Vec::new(),
    )?;
    // Grab the function and call it.
    info!("Calling into the guest program (high level):");
    let result = instance
        .call_typed_and_get_result::<u64, (u32, u64)>(&mut (), "add", (10, 5))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 15);

    Ok(())
}

#[test]
#[serial]
pub fn test_multi_module_call2() -> anyhow::Result<()> {
    let mut instance = build_instance(
        "output/multi_module_call.polkavm",
        "../examples/multi_module/sources/modules2.move",
        vec!["multi_module=0x7"],
    )?;

    // Grab the function and call it.
    info!("Calling into the guest program (high level):");
    let result = instance
        .call_typed_and_get_result::<u32, (u32, u32, u32)>(&mut (), "add_all", (10, 5, 5))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 20);

    Ok(())
}

fn build_instance(output: &str, source: &str, mapping: Vec<&str>) -> anyhow::Result<Instance> {
    initialize_logger();
    let mut build_options = BuildOptions::new(output).source(source);
    for m in mapping {
        build_options = build_options.address_mapping(m);
    }
    let program_bytes = build_polka_from_move(build_options)?;
    let blob = parse_to_blob(&program_bytes)?;

    let config = Config::from_env()?;
    let engine = Engine::new(&config)?;
    let module = Module::from_blob(&engine, &Default::default(), blob)?;

    let linker: Linker = Linker::new();

    // Link the host functions with the module.
    let instance_pre = linker.instantiate_pre(&module)?;

    // Instantiate the module.
    let instance = instance_pre.instantiate()?;

    Ok(instance)
}
