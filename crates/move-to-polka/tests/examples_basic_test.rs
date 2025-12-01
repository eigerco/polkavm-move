use std::collections::HashSet;

use move_to_polka::{
    initialize_logger,
    linker::{copy_to_guest, create_blob, create_instance},
};
use once_cell::sync::OnceCell;
use polkavm::{CallError, ProgramBlob};

use polkavm_move_native::{
    host::ProgramError,
    types::{MoveAddress, MoveSigner, ACCOUNT_ADDRESS_LENGTH},
};

static COMPILE_ONCE: OnceCell<ProgramBlob> = OnceCell::new();

fn create_blob_once() -> ProgramBlob {
    COMPILE_ONCE
        .get_or_init(|| {
            initialize_logger();
            create_blob(
                "output/basic/basic.polkavm",
                "../../examples/basic/",
                HashSet::new(),
            )
            .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}

#[test]
#[ignore]
pub fn test_error() -> anyhow::Result<()> {
    initialize_logger();
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "error", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    // error numbers are category << 16 + the reason code (42 in this case)

    Ok(())
}

#[test]
#[ignore]
pub fn test_arith() -> anyhow::Result<()> {
    initialize_logger();
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u64, ()>(&mut runtime, "main_arith", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    let result = instance
        .call_typed_and_get_result::<u64, ()>(&mut runtime, "abort_on_div_by_zero", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_err());

    Ok(())
}

#[test]
#[ignore]
pub fn test_basic_program_execution() -> anyhow::Result<()> {
    initialize_logger();
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result =
        instance.call_typed_and_get_result::<(), (u64,)>(&mut runtime, "abort_with_code", (42,));
    if let CallError::User(ProgramError::Abort(code)) = result.err().unwrap() {
        assert_eq!(code, 42, "Expected an abort with code 42");
    } else {
        panic!("Expected a ProgramError::Abort(42)",);
    }

    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), _>(&mut runtime, "main_basic", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[ignore]
pub fn test_tuple_implementation() -> anyhow::Result<()> {
    initialize_logger();
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u64, ()>(&mut runtime, "main_tuple", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[ignore]
pub fn test_struct() -> anyhow::Result<()> {
    initialize_logger();
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "main_struct", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
