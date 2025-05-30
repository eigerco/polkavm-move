use move_to_polka::{
    initialize_logger,
    linker::{new_move_program, BuildOptions},
};
use polkavm::{CallError, Instance};

use polkavm_move_native::{
    host::{MemAllocator, ProgramError},
    types::{MoveAddress, MoveSigner, ACCOUNT_ADDRESS_LENGTH},
};
use serial_test::serial;

#[test]
#[serial] // TODO: find the reason this needs to run serially on macOS
pub fn test_morebasic_program_execution() -> anyhow::Result<()> {
    let (mut instance, mut allocator) = build_instance(
        "output/morebasic.polkavm",
        "../examples/basic/sources/morebasic.move",
        vec![],
    )?;
    // Grab the function and call it.
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut allocator, "sum", (1, 10))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 11);

    Ok(())
}

#[test]
#[serial]
pub fn test_void_program_execution() -> anyhow::Result<()> {
    let (mut instance, mut allocator) = build_instance(
        "output/void.polkavm",
        "../examples/basic/sources/void.move",
        vec![],
    )?;
    // Grab the function and call it.
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "foo", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_arith() -> anyhow::Result<()> {
    let (mut instance, mut allocator) = build_instance(
        "output/arith.polkavm",
        "../examples/basic/sources/arith.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut allocator, "div", (12, 3))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 4);

    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut allocator, "mul", (12, 3))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 36);

    // div by zero and overflow should fail
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut allocator, "div", (12, 0))
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_err());

    // overflow on multiplication should fail
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut allocator, "mul", (u64::MAX, 2))
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_err());

    Ok(())
}

#[test]
#[serial]
pub fn test_basic_program_execution() -> anyhow::Result<()> {
    let (mut instance, mut allocator) = build_instance(
        "output/basic.polkavm",
        "../examples/basic/sources/basic.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "bar", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 19);

    let result =
        instance.call_typed_and_get_result::<(), (u64,)>(&mut allocator, "abort_with_code", (42,));
    assert!(matches!(
        result,
        Err(CallError::User(ProgramError::Abort(42)))
    ));

    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    let result = instance
        .call_typed_and_get_result::<u64, _>(&mut allocator, "foo", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 17);

    Ok(())
}

#[test]
#[serial]
pub fn test_tuple_implementation() -> anyhow::Result<()> {
    let (mut instance, mut allocator) = build_instance(
        "output/tuple.polkavm",
        "../examples/basic/sources/tuple.move",
        vec![],
    )?;
    // Grab the function and call it.
    let result = instance
        .call_typed_and_get_result::<u64, (u32, u64)>(&mut allocator, "add", (10, 5))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 15);

    Ok(())
}

#[test]
#[serial]
pub fn test_multi_module_call() -> anyhow::Result<()> {
    let (mut instance, mut allocator) = build_instance(
        "output/modules.polkavm",
        "../examples/multi_module/sources/modules.move",
        vec!["multi_module=0x7"],
    )?;

    // first try to call the void params function
    instance
        .call_typed_and_get_result::<(), _>(&mut allocator, "foo_bar", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // now set up the signer
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    let result = instance
        .call_typed_and_get_result::<u64, _>(&mut allocator, "foo", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 17);

    Ok(())
}

#[test]
#[serial]
pub fn test_multi_module_call2() -> anyhow::Result<()> {
    let (mut instance, mut allocator) = build_instance(
        "output/multi_module_call.polkavm",
        "../examples/multi_module/sources/modules2.move",
        vec!["multi_module=0x7"],
    )?;

    // Grab the function and call it.
    let result = instance
        .call_typed_and_get_result::<u32, (u32, u32, u32)>(&mut allocator, "add_all", (10, 5, 5))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 20);

    Ok(())
}

fn build_instance(
    output: &str,
    source: &str,
    mapping: Vec<&str>,
) -> anyhow::Result<(Instance<MemAllocator, ProgramError>, MemAllocator)> {
    initialize_logger();
    pub const MOVE_STDLIB_PATH: &str = env!("MOVE_STDLIB_PATH");
    let move_src = format!("{MOVE_STDLIB_PATH}/sources");
    let mut build_options = BuildOptions::new(output)
        .dependency(&move_src)
        .source(source)
        .address_mapping("std=0x1");

    for m in mapping {
        build_options = build_options.address_mapping(m);
    }
    new_move_program(build_options)
}
