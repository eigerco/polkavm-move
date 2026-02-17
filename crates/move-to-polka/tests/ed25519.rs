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
                        "output/ed25519_tests/ed25519_tests.polkavm",
                        "../../examples/ed25519_tests/",
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
pub fn test_ed25519_validate_key() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_ed25519_validate_key", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_ed25519_validate_key failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_ed25519_verify_signature() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_ed25519_verify_signature", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_ed25519_verify_signature failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_ed25519_verify_wrong_message() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_ed25519_verify_wrong_message", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_ed25519_verify_wrong_message failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- Debug: simple bool return test ---

#[test]
pub fn test_debug_return_true() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_debug_return_true", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_debug_return_true failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_debug_vec_tuple() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_debug_vec_tuple", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_debug_vec_tuple failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_debug_with_args() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_debug_with_args", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_debug_with_args failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_debug_args_sret() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_debug_args_sret", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_debug_args_sret failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_debug_complex() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_debug_complex", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_debug_complex failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_debug_vec_sret_tuple() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_debug_vec_sret_tuple", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_debug_vec_sret_tuple failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_debug_sret_tuple() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_debug_sret_tuple", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_debug_sret_tuple failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_debug_return_tuple() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_debug_return_tuple", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_debug_return_tuple failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- Tuple return ABI tests ---

#[test]
#[ignore] // tuple_out_ptrs bug: writing to caller's stack corrupts bool return value
pub fn test_secp256k1_ecdsa_recover() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_secp256k1_ecdsa_recover", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_secp256k1_ecdsa_recover failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_secp256k1_ecdsa_recover_wrong_id() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(
            &mut runtime,
            "test_secp256k1_ecdsa_recover_wrong_id",
            (),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_secp256k1_ecdsa_recover_wrong_id failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- Multi-Ed25519 tests ---

#[test]
pub fn test_multi_ed25519_auth_key() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_multi_ed25519_auth_key", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_multi_ed25519_auth_key failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_multi_ed25519_num_sub_pks() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_multi_ed25519_num_sub_pks", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_multi_ed25519_num_sub_pks failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- cmp::compare tests ---

#[test]
pub fn test_cmp_integers() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_cmp_integers", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_cmp_integers failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_cmp_vectors() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_cmp_vectors", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_cmp_vectors failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_cmp_bools() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_cmp_bools", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(result.is_ok(), "test_cmp_bools failed: {:?}", result.err());
    Ok(())
}

// --- Enum regression tests ---

#[test]
pub fn test_option_return() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_option_return", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_option_return failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- type_info tests ---

#[test]
pub fn test_type_name_primitives() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_type_name_primitives", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_type_name_primitives failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_type_of_struct() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_type_of_struct", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_type_of_struct failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- from_bcs tests ---

#[test]
pub fn test_from_bcs_u64() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_from_bcs_u64", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_from_bcs_u64 failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_from_bcs_bool() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_from_bcs_bool", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_from_bcs_bool failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- BCS size tests ---

#[test]
pub fn test_serialized_size_primitives() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_serialized_size_primitives", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_serialized_size_primitives failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_serialized_size_matches_to_bytes() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(
            &mut runtime,
            "test_serialized_size_matches_to_bytes",
            (),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_serialized_size_matches_to_bytes failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_constant_serialized_size_primitives() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(
            &mut runtime,
            "test_constant_serialized_size_primitives",
            (),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_constant_serialized_size_primitives failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_constant_serialized_size_variable() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(
            &mut runtime,
            "test_constant_serialized_size_variable",
            (),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_constant_serialized_size_variable failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_constant_serialized_size_structs() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(
            &mut runtime,
            "test_constant_serialized_size_structs",
            (),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_constant_serialized_size_structs failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- table tests ---

#[test]
pub fn test_table_add_and_borrow() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_table_add_and_borrow", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_table_add_and_borrow failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_table_contains() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_table_contains", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_table_contains failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_table_remove() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_table_remove", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_table_remove failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_table_borrow_mut() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_table_borrow_mut", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_table_borrow_mut failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_table_upsert() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_table_upsert", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_table_upsert failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- Object tests ---

#[test]
pub fn test_derived_address() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_derived_address", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_derived_address failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_exists_at_empty() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_exists_at_empty", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_exists_at_empty failed: {:?}",
        result.err()
    );
    Ok(())
}

// --- BLS12-381 aggregate tests ---

#[test]
pub fn test_bls12381_aggregate_pubkeys() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(&mut runtime, "test_bls12381_aggregate_pubkeys", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_bls12381_aggregate_pubkeys failed: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
pub fn test_bls12381_aggregate_sigs_and_verify() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<(), ()>(
            &mut runtime,
            "test_bls12381_aggregate_sigs_and_verify",
            (),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"));
    assert!(
        result.is_ok(),
        "test_bls12381_aggregate_sigs_and_verify failed: {:?}",
        result.err()
    );
    Ok(())
}
