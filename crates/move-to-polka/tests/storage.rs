use move_to_polka::{
    initialize_logger,
    linker::{copy_to_guest, create_blob, create_instance},
};
use once_cell::sync::OnceCell;
use polkavm::ProgramBlob;
use polkavm_move_native::types::{MoveAddress, MoveSigner, ACCOUNT_ADDRESS_LENGTH};

static COMPILE_ONCE: OnceCell<ProgramBlob> = OnceCell::new();

const TAG: [u8; 32] = [
    140, 175, 104, 51, 93, 103, 176, 59, 233, 233, 62, 75, 146, 109, 86, 116, 156, 138, 197, 255,
    19, 217, 64, 48, 181, 63, 171, 97, 181, 234, 157, 250,
];

fn create_blob_once() -> ProgramBlob {
    COMPILE_ONCE
        .get_or_init(|| {
            initialize_logger();
            create_blob(
                "output/storage/storage.polkavm",
                "../../examples/storage/",
                vec![],
            )
            .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}

#[test]
pub fn storage_does_not_exist() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "does_not_exist", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn storage_store_load() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "store", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "load", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn storage_store_different() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "store2", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "load2", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn storage_borrow_once() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "store", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "borrow", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // should have released the borrow
    let is_borrowed = runtime.storage.is_borrowed(move_signer.0, TAG);
    assert!(!is_borrowed);

    Ok(())
}

#[test]
pub fn storage_borrow_mut_once() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "store", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "borrow_mut", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // should have released the borrow
    let is_borrowed = runtime.storage.is_borrowed(move_signer.0, TAG);
    assert!(!is_borrowed);
    let value = runtime
        .storage
        .load(move_signer.0, TAG, false, false)
        .unwrap();
    assert_eq!(value[0], 100);

    Ok(())
}

#[test]
pub fn storage_borrow_mut_abort() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "store", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let result = instance.call_typed_and_get_result::<(), (u32,)>(
        &mut runtime,
        "borrow_mut_abort",
        (signer_address,),
    );
    assert!(result.is_err()); // the test aborts

    // should have released the borrow
    let is_borrowed = runtime.storage.is_borrowed(move_signer.0, TAG);
    assert!(!is_borrowed);

    Ok(())
}

#[test]
pub fn storage_borrow_mut_twice() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "store", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let result = instance
        .call_typed_and_get_result::<(), (u32,)>(
            &mut runtime,
            "borrow_mut_twice",
            (signer_address,),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_err(),
        "Expected error when borrowing mutably twice, but got: {result:?}",
    );

    Ok(())
}

#[test]
pub fn storage_store_twice() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    let result = instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "store_twice", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_err(),
        "Expected error when storing twice, but got: {result:?}"
    );

    Ok(())
}

#[test]
pub fn storage_load_non_existent() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    let result = instance
        .call_typed_and_get_result::<(), (u32,)>(
            &mut runtime,
            "load_non_existent",
            (signer_address,),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_err(),
        "Expected error when storing twice, but got: {result:?}"
    );

    Ok(())
}
