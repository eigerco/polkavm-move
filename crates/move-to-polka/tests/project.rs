use std::collections::HashSet;

use move_to_polka::{
    initialize_logger,
    linker::{copy_to_guest, create_blob, create_instance},
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
                "output/project/project.polkavm",
                "../../examples/project/",
                HashSet::new(),
            )
            .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}

#[test]
pub fn test_project() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    // set markers for debug displaying
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut runtime.allocator, &move_signer)?;

    let result = instance
        .call_typed_and_get_result::<(), _>(&mut runtime, "main", (signer_address,))
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_ok());

    Ok(())
}
