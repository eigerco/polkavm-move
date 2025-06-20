use move_to_polka::{
    initialize_logger,
    linker::{create_blob, create_instance},
};
use once_cell::sync::OnceCell;
use polkavm::ProgramBlob;
use polkavm_move_native::types::{MoveAddress, MoveSigner, ACCOUNT_ADDRESS_LENGTH};

static COMPILE_ONCE: OnceCell<ProgramBlob> = OnceCell::new();

fn create_blob_once() -> ProgramBlob {
    COMPILE_ONCE
        .get_or_init(|| {
            initialize_logger();
            create_blob(
                "output/storage/storage.polkavm",
                "../../examples/storage/sources/storage.move",
                vec![],
            )
            .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}

#[test]
pub fn storage_does_not_exist() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(
            &mut allocator,
            "does_not_exist",
            (signer_address,),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn storage_store_load() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "store", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "load", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn storage_store_different() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "store2", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "load2", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn storage_borrow() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "store", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "borrow", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn storage_borrow_mut() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "store", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "borrow_mut", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn storage_store_twice() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    let restul = instance
        .call_typed_and_get_result::<(), (u32,)>(&mut allocator, "store_twice", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        restul.is_err(),
        "Expected error when storing twice, but got: {:?}",
        restul
    );

    Ok(())
}

#[test]
pub fn storage_load_non_existent() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = allocator.copy_to_guest(&mut instance, &move_signer)?;

    let restul = instance
        .call_typed_and_get_result::<(), (u32,)>(
            &mut allocator,
            "load_non_existent",
            (signer_address,),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        restul.is_err(),
        "Expected error when storing twice, but got: {:?}",
        restul
    );

    Ok(())
}
