use move_to_polka::{initialize_logger, linker::new_move_program};
use polkavm::CallError;

use polkavm_move_native::{
    host::ProgramError,
    types::{MoveAddress, MoveSigner, ACCOUNT_ADDRESS_LENGTH},
};
use serial_test::serial;

#[test]
#[serial]
pub fn test_void_program_execution() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/void.polkavm",
        "../../examples/basic/sources/void.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "foo", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_error() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/error.polkavm",
        "../../examples/basic/sources/error.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "error", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    // error numbers are category << 16 + the reason code (42 in this case)

    Ok(())
}

#[test]
#[serial]
pub fn test_arith() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/arith.polkavm",
        "../../examples/basic/sources/arith.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "main", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    let result = instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "abort_on_div_by_zero", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_err());

    Ok(())
}

#[test]
#[serial]
pub fn test_basic_program_execution() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/basic.polkavm",
        "../../examples/basic/sources/basic.move",
        vec![],
    )?;
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

    instance
        .call_typed_and_get_result::<(), _>(&mut allocator, "foo", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_tuple_implementation() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/tuple.polkavm",
        "../../examples/basic/sources/tuple.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "main", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_struct() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/struct.polkavm",
        "../../examples/basic/sources/struct.move",
        vec![],
    )?;
    // Grab the function and call it.
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "main", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_multi_module_call() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/modules.polkavm",
        "../../examples/multi_module/sources/modules.move",
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

    instance
        .call_typed_and_get_result::<(), _>(&mut allocator, "foo", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_multi_module_call2() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/multi_module_call.polkavm",
        "../../examples/multi_module/sources/modules2.move",
        vec!["multi_module=0x7"],
    )?;

    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "main", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
