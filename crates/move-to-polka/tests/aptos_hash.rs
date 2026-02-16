use std::collections::HashSet;

use move_to_polka::{
    initialize_logger,
    linker::{create_blob, create_instance},
};
use once_cell::sync::OnceCell;
use polkavm::ProgramBlob;

static COMPILE_ONCE: OnceCell<ProgramBlob> = OnceCell::new();

fn create_blob_once() -> ProgramBlob {
    COMPILE_ONCE
        .get_or_init(|| {
            initialize_logger();
            // AptosStdLib has many modules with deeply nested types, requiring
            // a larger stack for the recursive LLVM type descriptor generation.
            std::thread::Builder::new()
                .stack_size(8 * 1024 * 1024)
                .spawn(|| {
                    create_blob(
                        "output/aptos_hash/aptos_hash.polkavm",
                        "../../examples/aptos_hash/",
                        HashSet::new(),
                    )
                    .expect("Failed to compile Move source to PolkaVM bytecode")
                })
                .expect("Failed to spawn compilation thread")
                .join()
                .expect("Compilation thread panicked")
        })
        .clone()
}

#[test]
pub fn test_aptos_keccak256() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "keccak256_expected_hash", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_ok(), "keccak256 failed: {:?}", result.err());

    Ok(())
}
