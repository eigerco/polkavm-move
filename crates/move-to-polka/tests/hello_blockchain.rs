use std::collections::HashSet;

use move_to_polka::{
    initialize_logger,
    linker::{create_blob, create_instance_from_pre, create_instance_pre},
};
use once_cell::sync::OnceCell;
use polkavm::{Instance, InstancePre};
use polkavm_move_native::host::{ProgramError, Runtime};

static INSTANCE_PRE: OnceCell<InstancePre<Runtime, ProgramError>> = OnceCell::new();

fn get_instance_pre() -> &'static InstancePre<Runtime, ProgramError> {
    INSTANCE_PRE.get_or_init(|| {
        initialize_logger();
        let blob = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                create_blob(
                    "output/hello_blockchain/hello_blockchain.polkavm",
                    "../../examples/hello_blockchain/",
                    HashSet::new(),
                )
                .expect("Failed to compile Move source to PolkaVM bytecode")
            })
            .expect("Failed to spawn compilation thread")
            .join()
            .expect("Compilation thread panicked");
        create_instance_pre(blob).expect("Failed to create InstancePre")
    })
}

fn new_instance() -> anyhow::Result<(Instance<Runtime, ProgramError>, Runtime)> {
    create_instance_from_pre(get_instance_pre())
}

/// Write a 32-byte signer address to guest memory and return the guest pointer.
/// Places the signer near the end of the heap to avoid allocator conflicts.
fn setup_signer(instance: &mut Instance<Runtime, ProgramError>) -> anyhow::Result<u32> {
    let heap_base = instance.module().memory_map().heap_base();
    let heap_size = instance.heap_size();
    let signer_addr = heap_base + heap_size - 64;

    // 32-byte Move address, matching the origin address used by the linker
    let mut origin = [0u8; 32];
    origin[0..20].copy_from_slice(&hex_literal::hex!(
        "ab010101010101010101010101010101010101ce"
    ));
    instance.write_memory(signer_addr, &origin)?;
    Ok(signer_addr)
}

#[test]
pub fn test_noop() -> anyhow::Result<()> {
    let (mut instance, mut runtime) = new_instance()?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_noop", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_ok(), "test_noop failed: {:?}", result.err());
    Ok(())
}

#[test]
pub fn test_basic_assert() -> anyhow::Result<()> {
    let (mut instance, mut runtime) = new_instance()?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_basic_assert", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_basic_assert failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_set_only() -> anyhow::Result<()> {
    let (mut instance, mut runtime) = new_instance()?;
    let signer_ptr = setup_signer(&mut instance)?;
    let result = instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "test_set_only", (signer_ptr,))
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_ok(), "test_set_only failed: {:?}", result.err());
    Ok(())
}

#[test]
pub fn test_set_and_get() -> anyhow::Result<()> {
    let (mut instance, mut runtime) = new_instance()?;
    let signer_ptr = setup_signer(&mut instance)?;
    let result = instance
        .call_typed_and_get_result::<(), (u32,)>(&mut runtime, "test_set_and_get", (signer_ptr,))
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_set_and_get failed: {:?}",
        result.err()
    );
    Ok(())
}
