use move_to_polka::{initialize_logger, linker::new_move_program};
use polkavm::CallError;

use polkavm_move_native::{
    host::ProgramError,
    types::{MoveAddress, MoveSigner, ACCOUNT_ADDRESS_LENGTH},
};
use serial_test::serial;

#[test]
#[serial] // TODO: find the reason this needs to run serially on macOS
pub fn test_morebasic_program_execution() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/morebasic.polkavm",
        "../examples/basic/sources/morebasic.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut allocator, "sum", (1, 10))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 11);

    Ok(())
}

#[test]
#[serial]
pub fn test_void_program_execution() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/void.polkavm",
        "../examples/basic/sources/void.move",
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
        "../examples/basic/sources/error.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "error", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    // error numbers are category << 16 + the reason code (42 in this case)
    let expected = (6 << 16) + 42;
    assert_eq!(result, expected);

    Ok(())
}

#[test]
#[serial]
pub fn test_string() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/string.polkavm",
        "../examples/basic/sources/string.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "foo", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_rv_bool() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/returns.polkavm",
        "../examples/basic/sources/returns.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "rv_bool_false", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 0);
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "rv_bool_true", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 1);

    Ok(())
}

#[test]
#[serial]
pub fn test_rv_u16() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/returns.polkavm",
        "../examples/basic/sources/returns.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "rv_u16", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 19);

    Ok(())
}

#[test]
#[serial]
pub fn test_rv_u32() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/returns.polkavm",
        "../examples/basic/sources/returns.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "rv_u32", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 19);

    Ok(())
}

#[test]
#[serial]
pub fn test_rv_u8() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/returns.polkavm",
        "../examples/basic/sources/returns.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<i32, ()>(&mut allocator, "rv_u8", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 19);

    Ok(())
}

#[test]
#[serial]
pub fn test_sha2() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/hash_tests.polkavm",
        "../examples/hash_tests/sources/hash_tests.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "sha2_256_expected_hash", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_ok());

    Ok(())
}

#[test]
#[serial]
pub fn test_sha3() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/hash_tests.polkavm",
        "../examples/hash_tests/sources/hash_tests.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "sha3_256_expected_hash", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_ok());

    Ok(())
}

#[test]
#[serial]
pub fn test_arith() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
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
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
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
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/tuple.polkavm",
        "../examples/basic/sources/tuple.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u64, (u32, u64)>(&mut allocator, "add", (10, 5))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 15);

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_new() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../examples/basic/sources/vector.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "vecnew", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 2);

    Ok(())
}

#[test]
#[serial]
pub fn test_struct() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../examples/basic/sources/struct.move",
        vec![],
    )?;
    // Grab the function and call it.
    let result = instance
        .call_typed_and_get_result::<u64, (u64, u64)>(&mut allocator, "create_counter", (10, 32))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 42);

    Ok(())
}

pub fn test_vector_isempty() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../examples/basic/sources/vector.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<i32, ()>(&mut allocator, "vecisempty", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 0);

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_cmp() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../examples/basic/sources/vector.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "veccmp", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 1);

    Ok(())
}

#[test]
#[serial]
pub fn test_multi_module_call() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
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
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/multi_module_call.polkavm",
        "../examples/multi_module/sources/modules2.move",
        vec!["multi_module=0x7"],
    )?;

    let result = instance
        .call_typed_and_get_result::<u32, (u32, u32, u32)>(&mut allocator, "add_all", (10, 5, 5))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 20);

    Ok(())
}
