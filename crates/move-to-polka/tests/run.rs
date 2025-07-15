use move_to_polka::{
    initialize_logger,
    linker::{copy_to_guest, create_blob, create_instance, run_lowlevel},
};
use once_cell::sync::OnceCell;
use polkavm::{ProgramBlob, Reg};
use polkavm_move_native::types::{MoveAddress, MoveSigner, ACCOUNT_ADDRESS_LENGTH};

static COMPILE_ONCE: OnceCell<ProgramBlob> = OnceCell::new();

fn create_blob_once() -> ProgramBlob {
    COMPILE_ONCE
        .get_or_init(|| {
            initialize_logger();
            create_blob("output/run/run.polkavm", "../../examples/run/", vec![])
                .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}

#[test]
pub fn test_run_lowlevel() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let mut address_bytes = [1u8; ACCOUNT_ADDRESS_LENGTH];
    address_bytes[0] = 0xab;
    address_bytes[ACCOUNT_ADDRESS_LENGTH - 1] = 0xce;

    let move_signer = MoveSigner(MoveAddress(address_bytes));

    let signer_address = copy_to_guest(&mut instance, &mut allocator, &move_signer)?;
    instance.set_reg(Reg::A0, signer_address as u64);

    run_lowlevel(&mut instance, &mut allocator, "pvm_start")?;

    Ok(())
}
