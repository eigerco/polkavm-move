// Minimal object module — avoids pulling in full AptosFramework dependency
module 0x1::object {
    native fun exists_at<T: key>(object: address): bool;

    /// Derives a deterministic object address: sha3_256(source || derive_from || 0xFC)
    public fun create_user_derived_object_address(source: address, derive_from: address): address {
        let bytes = std::vector::empty<u8>();
        let source_bytes = std::bcs::to_bytes(&source);
        std::vector::append(&mut bytes, source_bytes);
        let derive_from_bytes = std::bcs::to_bytes(&derive_from);
        std::vector::append(&mut bytes, derive_from_bytes);
        std::vector::push_back(&mut bytes, 0xFC); // OBJECT_DERIVED_SCHEME
        let hash = std::hash::sha3_256(bytes);
        aptos_std::from_bcs::to_address(hash)
    }

    public fun object_exists_at<T: key>(addr: address): bool {
        exists_at<T>(addr)
    }
}

module 0xa002::object_test {
    use 0x1::object;

    struct TestResource has key { value: u64 }

    public entry fun test_derived_address(_account: &signer) {
        let source = @0x1;
        let derive_from = @0x2;
        let addr = object::create_user_derived_object_address(source, derive_from);
        let addr2 = object::create_user_derived_object_address(source, derive_from);
        assert!(addr == addr2, 1); // deterministic
    }

    public entry fun test_exists_at_empty(_account: &signer) {
        assert!(!object::object_exists_at<TestResource>(@0x99), 1); // nothing at empty address
    }
}

module 0xa002::ed25519_tests {
    use aptos_std::ed25519;

    // Test ed25519 public key validation with a known valid key.
    public entry fun test_ed25519_validate_key(_account: &signer) {
        // Valid 32-byte ed25519 public key (derived from RFC 8032 TEST 1 secret key)
        let valid_pk = x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
        let pk = ed25519::new_unvalidated_public_key_from_bytes(valid_pk);
        let result = ed25519::unvalidated_public_key_to_authentication_key(&pk);
        // Just check we got a result (32 bytes = sha3-256 of scheme_id || pk)
        assert!(std::vector::length(&result) == 32, 1);
    }

    // Test ed25519 signature verification with a known-good signature.
    public entry fun test_ed25519_verify_signature(_account: &signer) {
        let msg = x"74657374206d657373616765";
        let pk_bytes = x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
        let sig_bytes = x"98a39ec11a0dfbbfdbd7a7e2394b2b83a16586e92100bcb9be672ddfba3e7acb861c94d6ad4cf6e3e60136ca141fc4f2f1be0c1b8ef0bea12aee76f007a4c30a";

        let pk = ed25519::new_unvalidated_public_key_from_bytes(pk_bytes);
        let sig = ed25519::new_signature_from_bytes(sig_bytes);
        let valid = ed25519::signature_verify_strict(&sig, &pk, msg);
        assert!(valid, 2);
    }

    // Test that verification fails with wrong message
    public entry fun test_ed25519_verify_wrong_message(_account: &signer) {
        let msg = x"deadbeef";
        let pk_bytes = x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
        let sig_bytes = x"98a39ec11a0dfbbfdbd7a7e2394b2b83a16586e92100bcb9be672ddfba3e7acb861c94d6ad4cf6e3e60136ca141fc4f2f1be0c1b8ef0bea12aee76f007a4c30a";

        let pk = ed25519::new_unvalidated_public_key_from_bytes(pk_bytes);
        let sig = ed25519::new_signature_from_bytes(sig_bytes);
        let valid = ed25519::signature_verify_strict(&sig, &pk, msg);
        assert!(!valid, 3);
    }
}

// multi_ed25519 tests — exercises option::some<UnvalidatedPublicKey> monomorphization
module 0xa002::multi_ed25519_tests {
    use aptos_std::multi_ed25519;

    // Test creating an unvalidated multi-ed25519 public key and deriving its auth key.
    // A 1-of-1 multi-ed25519 key is just 32 bytes of public key + 1 byte threshold.
    public entry fun test_multi_ed25519_auth_key(_account: &signer) {
        // 32-byte ed25519 public key (RFC 8032 TEST 1) + 1 byte threshold (0x01)
        let pk_bytes = x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a01";
        let pk = multi_ed25519::new_unvalidated_public_key_from_bytes(pk_bytes);
        let auth_key = multi_ed25519::unvalidated_public_key_to_authentication_key(&pk);
        assert!(std::vector::length(&auth_key) == 32, 1);
    }

    // Test that we can extract the number of sub-public-keys from a multi-ed25519 key.
    // This exercises option::some<u8> monomorphization alongside option::some<UnvalidatedPublicKey>.
    public entry fun test_multi_ed25519_num_sub_pks(_account: &signer) {
        let pk_bytes = x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a01";
        let pk = multi_ed25519::new_unvalidated_public_key_from_bytes(pk_bytes);
        // 1-of-1 key: should have exactly 1 sub-public-key
        let num = multi_ed25519::unvalidated_public_key_num_sub_pks(&pk);
        assert!(num == 1, 2);
    }
}

// BLS12-381 aggregate tests using hardcoded test vectors (non-test-only API)
module 0xa002::bls12381_tests {
    use aptos_std::bls12381;

    // Test: aggregate two public keys
    public entry fun test_bls12381_aggregate_pubkeys(_account: &signer) {
        let pk1_bytes = x"95a254501b7733239ed3cec4d56737977bd09ede881d8a234560e83e5525017add3b1dcc3eabfb85e12a4131b19c253b";
        let pop1_bytes = x"846aa12a4402eb67cb92a497e0716db573c817a4163783153f0ddca475f4870200049d8e9ed35087c786059c1f26fc9d0d39e3098f1bae074c062f84f24353210666bd58c0d9be3ff76ba9dd9ce905c5b602a12e78a04350275faacce8b7137d";
        let pk2_bytes = x"ac80a5e08c712d5f08f0306ad743f7d8c215d982489b84a1d6ba805733d94c006e8938f9089a75db3ffa135af33bc69a";
        let pop2_bytes = x"b1b22261eeb641b36d4f701f7e5635c5dd0ee53102e7ad8c11594be0d785f0bb5d75bd063ec2caa415e953f85e6e18e110d7ae595d18940e60894bd0a39eb157c1f646ee0f2079d64bd7f4e3c6cbc297e74ce69f3ae4e0728f915f1aac3cdf9b";

        let pop1 = bls12381::proof_of_possession_from_bytes(pop1_bytes);
        let pop2 = bls12381::proof_of_possession_from_bytes(pop2_bytes);
        let pk_with_pop1_opt = bls12381::public_key_from_bytes_with_pop(pk1_bytes, &pop1);
        let pk_with_pop2_opt = bls12381::public_key_from_bytes_with_pop(pk2_bytes, &pop2);
        assert!(std::option::is_some(&pk_with_pop1_opt), 10);
        assert!(std::option::is_some(&pk_with_pop2_opt), 11);

        let pk_with_pop1 = std::option::extract(&mut pk_with_pop1_opt);
        let pk_with_pop2 = std::option::extract(&mut pk_with_pop2_opt);

        let pks = std::vector::empty<bls12381::PublicKeyWithPoP>();
        std::vector::push_back(&mut pks, pk_with_pop1);
        std::vector::push_back(&mut pks, pk_with_pop2);
        let agg_pk = bls12381::aggregate_pubkeys(pks);

        // Verify the aggregated key has the expected bytes (48 bytes)
        let agg_pk_bytes = bls12381::aggregate_pubkey_to_bytes(&agg_pk);
        assert!(std::vector::length(&agg_pk_bytes) == 48, 1);
        let expected = x"af9c7a267f7990fc590f743837a3b7e5c171128f0127279e8df2886ccfef3439c5d77f4330bb1a9659af3e378d6a161c";
        assert!(agg_pk_bytes == expected, 2);
    }

    // Test: aggregate two signatures and verify with aggregate_verify
    public entry fun test_bls12381_aggregate_sigs_and_verify(_account: &signer) {
        let pk1_bytes = x"95a254501b7733239ed3cec4d56737977bd09ede881d8a234560e83e5525017add3b1dcc3eabfb85e12a4131b19c253b";
        let pop1_bytes = x"846aa12a4402eb67cb92a497e0716db573c817a4163783153f0ddca475f4870200049d8e9ed35087c786059c1f26fc9d0d39e3098f1bae074c062f84f24353210666bd58c0d9be3ff76ba9dd9ce905c5b602a12e78a04350275faacce8b7137d";
        let pk2_bytes = x"ac80a5e08c712d5f08f0306ad743f7d8c215d982489b84a1d6ba805733d94c006e8938f9089a75db3ffa135af33bc69a";
        let pop2_bytes = x"b1b22261eeb641b36d4f701f7e5635c5dd0ee53102e7ad8c11594be0d785f0bb5d75bd063ec2caa415e953f85e6e18e110d7ae595d18940e60894bd0a39eb157c1f646ee0f2079d64bd7f4e3c6cbc297e74ce69f3ae4e0728f915f1aac3cdf9b";

        let sig1_bytes = x"8835d17af1d2f32a24c07a7fc11923fec80e73826ff704a7142fd5c80ad3931ac28eb8a09277e05fa1bac358069c8ad014bb868e91d5d71c47273ee566e28722415c49882d03d98a525749e9aaee06a29e5fccd50920eb2f9f5c7da7a34508b2";
        let sig2_bytes = x"a2563d91b24bed6290cd753dbbabc390992acf730a2a78b154e128c6f54e39f7b891dfa1731009ddd553fadef75ebdcf0e89b75ba27aff21baf6d97002d87919c2e8aacadc5ce87661db8e40ffcc52a1f5671af841f30df2d7016b02b6c77d35";

        // Build PublicKeyWithPoP from raw bytes
        let pop1 = bls12381::proof_of_possession_from_bytes(pop1_bytes);
        let pop2 = bls12381::proof_of_possession_from_bytes(pop2_bytes);
        let pk_with_pop1 = std::option::extract(&mut bls12381::public_key_from_bytes_with_pop(pk1_bytes, &pop1));
        let pk_with_pop2 = std::option::extract(&mut bls12381::public_key_from_bytes_with_pop(pk2_bytes, &pop2));

        // Build Signatures from raw bytes
        let sig1 = bls12381::signature_from_bytes(sig1_bytes);
        let sig2 = bls12381::signature_from_bytes(sig2_bytes);

        // Aggregate signatures
        let sigs = std::vector::empty<bls12381::Signature>();
        std::vector::push_back(&mut sigs, sig1);
        std::vector::push_back(&mut sigs, sig2);
        let agg_sig_opt = bls12381::aggregate_signatures(sigs);
        assert!(std::option::is_some(&agg_sig_opt), 2);

        // Verify aggregate signature
        let agg_sig = std::option::extract(&mut agg_sig_opt);
        let pks = std::vector::empty<bls12381::PublicKeyWithPoP>();
        std::vector::push_back(&mut pks, pk_with_pop1);
        std::vector::push_back(&mut pks, pk_with_pop2);

        let msgs = std::vector::empty<vector<u8>>();
        std::vector::push_back(&mut msgs, b"message one");
        std::vector::push_back(&mut msgs, b"message two");

        let valid = bls12381::verify_aggregate_signature(&agg_sig, pks, msgs);
        assert!(valid, 3);
    }
}

// cmp::compare tests — exercises the native compare function
module 0xa002::cmp_test {
    use std::cmp;

    public entry fun test_cmp_integers(_account: &signer) {
        assert!(cmp::compare(&1u64, &2u64).is_lt(), 1);
        assert!(cmp::compare(&2u64, &2u64).is_eq(), 2);
        assert!(cmp::compare(&3u64, &2u64).is_gt(), 3);
        assert!(cmp::compare(&0u8, &255u8).is_lt(), 4);
        assert!(cmp::compare(&255u8, &0u8).is_gt(), 5);
        assert!(cmp::compare(&100u128, &100u128).is_eq(), 6);
    }

    public entry fun test_cmp_vectors(_account: &signer) {
        let v1 = vector[1u8, 2u8, 3u8];
        let v2 = vector[1u8, 2u8, 4u8];
        let v3 = vector[1u8, 2u8, 3u8];
        let v4 = vector[1u8, 2u8];

        assert!(cmp::compare(&v1, &v2).is_lt(), 10);
        assert!(cmp::compare(&v1, &v3).is_eq(), 11);
        assert!(cmp::compare(&v2, &v1).is_gt(), 12);
        // Shorter prefix is less
        assert!(cmp::compare(&v4, &v1).is_lt(), 13);
        assert!(cmp::compare(&v1, &v4).is_gt(), 14);
    }

    public entry fun test_cmp_bools(_account: &signer) {
        assert!(cmp::compare(&false, &true).is_lt(), 20);
        assert!(cmp::compare(&true, &true).is_eq(), 21);
        assert!(cmp::compare(&true, &false).is_gt(), 22);
    }
}

// type_info tests
module 0xa002::type_info_test {
    use aptos_std::type_info;
    use std::string;

    public entry fun test_type_name_primitives(_account: &signer) {
        assert!(type_info::type_name<u64>() == string::utf8(b"u64"), 1);
        assert!(type_info::type_name<bool>() == string::utf8(b"bool"), 2);
        assert!(type_info::type_name<vector<u8>>() == string::utf8(b"vector<u8>"), 3);
    }

    public entry fun test_type_of_struct(_account: &signer) {
        let ti = type_info::type_of<type_info::TypeInfo>();
        assert!(type_info::module_name(&ti) == b"type_info", 10);
    }
}

// from_bcs tests
module 0xa002::from_bcs_test {
    use aptos_std::from_bcs;
    use std::bcs;

    public entry fun test_from_bcs_u64(_account: &signer) {
        let v: u64 = 42;
        let bytes = bcs::to_bytes(&v);
        let result = from_bcs::to_u64(bytes);
        assert!(result == 42, 1);
    }

    public entry fun test_from_bcs_bool(_account: &signer) {
        let bytes = bcs::to_bytes(&true);
        let result = from_bcs::to_bool(bytes);
        assert!(result == true, 2);
    }
}

// bcs size tests
module 0xa002::bcs_size_test {
    use std::bcs;

    struct FixedStruct has drop { a: u64, b: u64 }
    struct NestedFixed has drop { inner: FixedStruct, flag: bool }

    public entry fun test_serialized_size_primitives(_account: &signer) {
        assert!(bcs::serialized_size(&true) == 1, 1);
        assert!(bcs::serialized_size(&0u8) == 1, 2);
        assert!(bcs::serialized_size(&0u64) == 8, 3);
        assert!(bcs::serialized_size(&0u128) == 16, 4);
        assert!(bcs::serialized_size(&@0x1) == 32, 5);
    }

    public entry fun test_serialized_size_matches_to_bytes(_account: &signer) {
        let v: u64 = 42;
        let bytes = bcs::to_bytes(&v);
        assert!(bcs::serialized_size(&v) == std::vector::length(&bytes), 1);
    }

    public entry fun test_constant_serialized_size_primitives(_account: &signer) {
        assert!(bcs::constant_serialized_size<bool>() == std::option::some(1), 1);
        assert!(bcs::constant_serialized_size<u8>() == std::option::some(1), 2);
        assert!(bcs::constant_serialized_size<u64>() == std::option::some(8), 3);
        assert!(bcs::constant_serialized_size<u128>() == std::option::some(16), 4);
        assert!(bcs::constant_serialized_size<address>() == std::option::some(32), 5);
    }

    public entry fun test_constant_serialized_size_variable(_account: &signer) {
        // Vectors have variable size
        assert!(bcs::constant_serialized_size<vector<u8>>() == std::option::none(), 1);
        // Options (enums) have variable size
        assert!(bcs::constant_serialized_size<std::option::Option<u64>>() == std::option::none(), 2);
    }

    public entry fun test_constant_serialized_size_structs(_account: &signer) {
        // Struct with all constant-size fields
        assert!(bcs::constant_serialized_size<FixedStruct>() == std::option::some(16), 1);
        // Nested struct with all constant-size fields
        assert!(bcs::constant_serialized_size<NestedFixed>() == std::option::some(17), 2);
    }
}

// Minimal enum regression test: Option<T> is now an enum in Aptos stdlib
module 0xa002::enum_test {
    fun make_some_u64(): std::option::Option<u64> {
        std::option::some(42u64)
    }
    public entry fun test_option_return(_account: &signer) {
        let opt = make_some_u64();
        assert!(std::option::is_some(&opt), 1);
    }
}

// Minimal test: a simple native function returning bool (non-tuple)
module 0xa002::debug_test {
    native fun debug_return_true(): bool;
    native fun debug_return_tuple(): (u64, bool);

    // A big struct (>16 bytes) to force sret return
    struct BigResult has drop { a: u64, b: u64, c: u64 }

    // Wrapper that returns a big struct via sret, calls tuple-returning native inside
    public fun wrapper_sret(): BigResult {
        let (_val, success) = debug_return_tuple();
        if (success) {
            BigResult { a: 1, b: 2, c: 3 }
        } else {
            BigResult { a: 0, b: 0, c: 0 }
        }
    }

    native fun debug_return_vec_bool(): (vector<u8>, bool);
    native fun debug_vec_args_tuple(msg: vector<u8>, id: u8, sig: vector<u8>): (vector<u8>, bool);

    public entry fun test_debug_vec_tuple(_account: &signer) {
        let (_vec, success) = debug_return_vec_bool();
        assert!(success, 96);
    }

    // sret wrapper that calls (vector<u8>, bool) native
    public fun wrapper_vec_sret(): BigResult {
        let (_vec, success) = debug_return_vec_bool();
        if (success) {
            BigResult { a: 1, b: 2, c: 3 }
        } else {
            BigResult { a: 0, b: 0, c: 0 }
        }
    }

    public entry fun test_debug_vec_sret_tuple(_account: &signer) {
        let result = wrapper_vec_sret();
        assert!(result.a == 1, 95);
    }

    // Test with arguments matching ecdsa_recover_internal signature
    public entry fun test_debug_with_args(_account: &signer) {
        let msg = x"0102030405";
        let sig = x"0607080910";
        let (_vec, success) = debug_vec_args_tuple(msg, 1, sig);
        assert!(success, 94);
    }

    // Test with args + sret wrapper (matching ecdsa_recover pattern)
    public fun wrapper_args_sret(msg: vector<u8>, id: u8, sig: vector<u8>): BigResult {
        let (_vec, success) = debug_vec_args_tuple(msg, id, sig);
        if (success) {
            BigResult { a: 1, b: 2, c: 3 }
        } else {
            BigResult { a: 0, b: 0, c: 0 }
        }
    }

    public entry fun test_debug_args_sret(_account: &signer) {
        let msg = x"0102030405";
        let sig = x"0607080910";
        let result = wrapper_args_sret(msg, 1, sig);
        assert!(result.a == 1, 93);
    }

    // Replicate ecdsa_recover's exact pattern: complex branching + sret + tuple
    public fun wrapper_complex(msg: vector<u8>, id: u8, sig: vector<u8>): BigResult {
        if(id != 0 && id != 1 && id != 2 && id != 3) {
            abort 42
        };
        let (_vec, success) = debug_vec_args_tuple(msg, id, sig);
        if (success) {
            BigResult { a: 1, b: 2, c: 3 }
        } else {
            BigResult { a: 0, b: 0, c: 0 }
        }
    }

    public entry fun test_debug_complex(_account: &signer) {
        let msg = x"0102030405";
        let sig = x"0607080910";
        let result = wrapper_complex(msg, 1, sig);
        assert!(result.a == 1, 92);
    }

    public entry fun test_debug_return_true(_account: &signer) {
        assert!(debug_return_true(), 99);
    }

    public entry fun test_debug_return_tuple(_account: &signer) {
        let (_val, success) = debug_return_tuple();
        assert!(success, 98);
    }

    // Test: tuple-returning native called from sret-returning function
    public entry fun test_debug_sret_tuple(_account: &signer) {
        let result = wrapper_sret();
        assert!(result.a == 1, 97);
    }
}

// table tests — uses table_with_length which wraps table and provides destroy_empty
module 0xa002::table_test {
    use aptos_std::table_with_length as table;

    public entry fun test_table_add_and_borrow(_account: &signer) {
        let t = table::new<u64, u64>();
        table::add(&mut t, 1, 100);
        table::add(&mut t, 2, 200);
        assert!(*table::borrow(&t, 1) == 100, 1);
        assert!(*table::borrow(&t, 2) == 200, 2);
        // Clean up: remove all then destroy
        table::remove(&mut t, 1);
        table::remove(&mut t, 2);
        table::destroy_empty(t);
    }

    public entry fun test_table_contains(_account: &signer) {
        let t = table::new<u64, u64>();
        assert!(!table::contains(&t, 42), 1);
        table::add(&mut t, 42, 999);
        assert!(table::contains(&t, 42), 2);
        table::remove(&mut t, 42);
        table::destroy_empty(t);
    }

    public entry fun test_table_remove(_account: &signer) {
        let t = table::new<u64, u64>();
        table::add(&mut t, 1, 100);
        let val = table::remove(&mut t, 1);
        assert!(val == 100, 1);
        assert!(!table::contains(&t, 1), 2);
        table::destroy_empty(t);
    }

    public entry fun test_table_borrow_mut(_account: &signer) {
        let t = table::new<u64, u64>();
        table::add(&mut t, 1, 100);
        let val_ref = table::borrow_mut(&mut t, 1);
        *val_ref = 200;
        assert!(*table::borrow(&t, 1) == 200, 1);
        table::remove(&mut t, 1);
        table::destroy_empty(t);
    }

    public entry fun test_table_upsert(_account: &signer) {
        let t = table::new<u64, u64>();
        table::upsert(&mut t, 1, 10);
        assert!(*table::borrow(&t, 1) == 10, 1);
        table::upsert(&mut t, 1, 20);
        assert!(*table::borrow(&t, 1) == 20, 2);
        table::remove(&mut t, 1);
        table::destroy_empty(t);
    }

}

// Tests tuple return ABI: secp256k1::ecdsa_recover_internal returns (vector<u8>, bool)
module 0xa002::secp256k1_tests {
    use aptos_std::secp256k1;

    // Test secp256k1 ECDSA recovery with a known test vector.
    // Exercises the (vector<u8>, bool) tuple return from ecdsa_recover_internal.
    public entry fun test_secp256k1_ecdsa_recover(_account: &signer) {
        // SHA-256 hash of "test secp256k1 recovery"
        let msg_hash = x"55252c877383405ae1778aebe88516a322e3d59c623e60f3b6ad51ba3bd7a23d";
        let sig_bytes = x"65468b2162c62e3d6abe36a051aa5f3d81f9a1b6dc85637c94fbf9877298041211c9eb4fd4f63bc65b13e523ab2c03242bb0e96473761a10bff600a5e299215a";
        let recovery_id: u8 = 1;
        let expected_pk = x"36209f7744d76e40e5893e6d5c1ac494c312751c0f8a125a3d9e1cdb089a45645fd963ae12f33f9056d5a8856b489138582e83aa812b66578d62a7263bf7f8c1";

        let sig = secp256k1::ecdsa_signature_from_bytes(sig_bytes);
        let result = secp256k1::ecdsa_recover(msg_hash, recovery_id, &sig);

        // Verify recovery succeeded
        assert!(std::option::is_some(&result), 1);
        let pk = std::option::borrow(&result);
        let pk_bytes = secp256k1::ecdsa_raw_public_key_to_bytes(pk);
        assert!(pk_bytes == expected_pk, 2);
    }

    // Test recovery fails with wrong recovery_id
    public entry fun test_secp256k1_ecdsa_recover_wrong_id(_account: &signer) {
        let msg_hash = x"55252c877383405ae1778aebe88516a322e3d59c623e60f3b6ad51ba3bd7a23d";
        let sig_bytes = x"65468b2162c62e3d6abe36a051aa5f3d81f9a1b6dc85637c94fbf9877298041211c9eb4fd4f63bc65b13e523ab2c03242bb0e96473761a10bff600a5e299215a";
        // Wrong recovery_id (0 instead of 1)
        let recovery_id: u8 = 0;

        let sig = secp256k1::ecdsa_signature_from_bytes(sig_bytes);
        let result = secp256k1::ecdsa_recover(msg_hash, recovery_id, &sig);

        // Recovery should succeed (returns a key), but it will be the wrong key
        // OR it may fail entirely depending on the curve point
        // Either way, the result should NOT match the expected key
        let expected_pk = x"36209f7744d76e40e5893e6d5c1ac494c312751c0f8a125a3d9e1cdb089a45645fd963ae12f33f9056d5a8856b489138582e83aa812b66578d62a7263bf7f8c1";
        if (std::option::is_some(&result)) {
            let pk = std::option::borrow(&result);
            let pk_bytes = secp256k1::ecdsa_raw_public_key_to_bytes(pk);
            assert!(pk_bytes != expected_pk, 3);
        };
        // If result is None, that's also acceptable (wrong recovery_id)
    }
}

// Minimal unit_test module — normally #[test_only] in stdlib
module std::unit_test {
    native public fun create_signers_for_testing(num_signers: u64): vector<signer>;
}

module 0xa002::unit_test_test {
    use std::unit_test;
    use std::signer;

    public entry fun test_create_signers_count(_account: &signer) {
        let signers = unit_test::create_signers_for_testing(5);
        assert!(std::vector::length(&signers) == 5, 1);
    }

    public entry fun test_create_signers_deterministic(_account: &signer) {
        let signers1 = unit_test::create_signers_for_testing(3);
        let signers2 = unit_test::create_signers_for_testing(3);
        // Same input should produce same addresses
        let i = 0;
        while (i < 3) {
            let addr1 = signer::address_of(std::vector::borrow(&signers1, i));
            let addr2 = signer::address_of(std::vector::borrow(&signers2, i));
            assert!(addr1 == addr2, 2);
            i = i + 1;
        };
    }

    public entry fun test_create_signers_unique(_account: &signer) {
        let signers = unit_test::create_signers_for_testing(3);
        let addr0 = signer::address_of(std::vector::borrow(&signers, 0));
        let addr1 = signer::address_of(std::vector::borrow(&signers, 1));
        let addr2 = signer::address_of(std::vector::borrow(&signers, 2));
        // All addresses should be different
        assert!(addr0 != addr1, 3);
        assert!(addr1 != addr2, 4);
        assert!(addr0 != addr2, 5);
    }

    public entry fun test_create_signers_empty(_account: &signer) {
        let signers = unit_test::create_signers_for_testing(0);
        assert!(std::vector::length(&signers) == 0, 6);
    }
}

// --- Ristretto255 module ---
// --- Ristretto255 test module ---
module 0xa002::ristretto255_test {
    use aptos_std::ristretto255;

    // Test scalar arithmetic roundtrip
    public entry fun test_scalar_arithmetic(_account: &signer) {
        let two = ristretto255::new_scalar_from_u64(2);
        let three = ristretto255::new_scalar_from_u64(3);
        let five = ristretto255::new_scalar_from_u64(5);
        let six = ristretto255::new_scalar_from_u64(6);

        // 2 + 3 = 5
        let sum = ristretto255::scalar_add(&two, &three);
        assert!(ristretto255::scalar_to_bytes(&sum) == ristretto255::scalar_to_bytes(&five), 1);

        // 2 * 3 = 6
        let prod = ristretto255::scalar_mul(&two, &three);
        assert!(ristretto255::scalar_to_bytes(&prod) == ristretto255::scalar_to_bytes(&six), 2);

        // 5 - 3 = 2
        let diff = ristretto255::scalar_sub(&five, &three);
        assert!(ristretto255::scalar_to_bytes(&diff) == ristretto255::scalar_to_bytes(&two), 3);

        // neg(2) + 2 = 0
        let neg_two = ristretto255::scalar_neg(&two);
        let zero = ristretto255::scalar_add(&two, &neg_two);
        let scalar_zero = ristretto255::scalar_zero();
        assert!(ristretto255::scalar_to_bytes(&zero) == ristretto255::scalar_to_bytes(&scalar_zero), 4);
    }

    // Test point identity compresses to the known identity encoding (all zeros)
    public entry fun test_point_identity_compress(_account: &signer) {
        let id = ristretto255::point_identity();
        let compressed = ristretto255::point_compress(&id);
        let bytes = ristretto255::point_to_bytes(&compressed);
        assert!(std::vector::length(&bytes) == 32, 1);
        // Ristretto identity compresses to 32 zero bytes
        let i = 0;
        while (i < 32) {
            assert!(*std::vector::borrow(&bytes, i) == 0, 2);
            i = i + 1;
        };
    }

    // Test P + Q - Q == P
    public entry fun test_point_add_sub_roundtrip(_account: &signer) {
        let s1 = ristretto255::new_scalar_from_u64(42);
        let s2 = ristretto255::new_scalar_from_u64(99);
        let p = ristretto255::basepoint_mul(&s1);
        let q = ristretto255::basepoint_mul(&s2);

        let pq = ristretto255::point_add(&p, &q);
        let p_back = ristretto255::point_sub(&pq, &q);
        assert!(ristretto255::point_equals(&p, &p_back), 1);
    }

    // Test basepoint_mul and compress
    public entry fun test_basepoint_mul(_account: &signer) {
        let one = ristretto255::scalar_one();
        let bp = ristretto255::basepoint_mul(&one);
        let compressed = ristretto255::point_compress(&bp);
        let bytes = ristretto255::point_to_bytes(&compressed);
        assert!(std::vector::length(&bytes) == 32, 1);
        // Known basepoint compressed encoding (first byte 0xe2)
        assert!(*std::vector::borrow(&bytes, 0) == 0xe2, 2);
    }

    // Test decompress roundtrip: compress then decompress equals original
    // Note: point_decompress in Aptos stdlib returns RistrettoPoint (not Option)
    public entry fun test_decompress_roundtrip(_account: &signer) {
        let s = ristretto255::new_scalar_from_u64(12345);
        let p = ristretto255::basepoint_mul(&s);
        let compressed = ristretto255::point_compress(&p);
        let decompressed = ristretto255::point_decompress(&compressed);
        assert!(ristretto255::point_equals(&p, &decompressed), 1);
    }

    // Test multi_scalar_mul consistency: s1*P1 + s2*P2 via MSM == via individual ops
    // Note: avoid point_clone (feature-gated), create fresh points instead
    public entry fun test_multi_scalar_mul(_account: &signer) {
        let s1 = ristretto255::new_scalar_from_u64(7);
        let s2 = ristretto255::new_scalar_from_u64(11);
        let s3 = ristretto255::new_scalar_from_u64(3);
        let s5 = ristretto255::new_scalar_from_u64(5);
        let p1 = ristretto255::basepoint_mul(&s3);
        let p2 = ristretto255::basepoint_mul(&s5);

        // MSM: s1*P1 + s2*P2 (create fresh points for the vector)
        let p1_msm = ristretto255::basepoint_mul(&s3);
        let p2_msm = ristretto255::basepoint_mul(&s5);
        let points = vector[p1_msm, p2_msm];
        let scalars = vector[s1, s2];
        let msm_result = ristretto255::multi_scalar_mul(&points, &scalars);

        // Manual: s1*P1 + s2*P2
        let s1 = ristretto255::new_scalar_from_u64(7);
        let s2 = ristretto255::new_scalar_from_u64(11);
        let t1 = ristretto255::point_mul(&p1, &s1);
        let t2 = ristretto255::point_mul(&p2, &s2);
        let manual_result = ristretto255::point_add(&t1, &t2);

        assert!(ristretto255::point_equals(&msm_result, &manual_result), 1);
    }
}

// --- Minimal aptos_framework shim modules ---

module aptos_framework::event {
    native fun write_module_event_to_store<T: drop + store>(msg: T);
    native fun write_to_event_store<T: drop + store>(guid: vector<u8>, count: u64, msg: T);

    public fun emit<T: drop + store>(msg: T) {
        write_module_event_to_store(msg);
    }
}

module aptos_framework::create_signer {
    public(friend) native fun create_signer(addr: address): signer;

    friend aptos_framework::create_signer_test;
}

module aptos_framework::transaction_context {
    native fun get_txn_hash(): vector<u8>;
    native fun get_script_hash(): vector<u8>;
    native fun chain_id_internal(): u8;
    native fun max_gas_amount_internal(): u64;
    native fun gas_unit_price_internal(): u64;
    native fun sender_internal(): address;
    native fun gas_payer_internal(): address;
    native fun generate_unique_address(): address;
    native fun secondary_signers_internal(): vector<address>;

    public fun get_transaction_hash(): vector<u8> { get_txn_hash() }
    public fun get_chain_id(): u8 { chain_id_internal() }
    public fun get_max_gas_amount(): u64 { max_gas_amount_internal() }
    public fun get_gas_unit_price(): u64 { gas_unit_price_internal() }
    public fun get_sender(): address { sender_internal() }
    public fun get_gas_payer(): address { gas_payer_internal() }
    public fun generate_auid_address(): address { generate_unique_address() }
    public fun get_secondary_signers(): vector<address> { secondary_signers_internal() }
    public fun get_script_hash_public(): vector<u8> { get_script_hash() }
}

module aptos_framework::aggregator_v2 {
    struct Aggregator<IntElement> has store, drop {
        value: IntElement,
        max_value: IntElement,
    }

    struct AggregatorSnapshot<IntElement> has store, drop {
        value: IntElement,
    }

    public native fun create_aggregator<IntElement: copy + drop>(max_value: IntElement): Aggregator<IntElement>;
    public native fun create_unbounded_aggregator<IntElement: copy + drop>(): Aggregator<IntElement>;
    public native fun try_add<IntElement>(self: &mut Aggregator<IntElement>, value: IntElement): bool;
    public native fun try_sub<IntElement>(self: &mut Aggregator<IntElement>, value: IntElement): bool;
    native fun is_at_least_impl<IntElement>(self: &Aggregator<IntElement>, min_amount: IntElement): bool;
    public native fun read<IntElement>(self: &Aggregator<IntElement>): IntElement;
    public native fun snapshot<IntElement>(self: &Aggregator<IntElement>): AggregatorSnapshot<IntElement>;
    public native fun create_snapshot<IntElement: copy + drop>(value: IntElement): AggregatorSnapshot<IntElement>;
    public native fun read_snapshot<IntElement>(self: &AggregatorSnapshot<IntElement>): IntElement;

    public fun is_at_least<IntElement>(self: &Aggregator<IntElement>, min_amount: IntElement): bool {
        is_at_least_impl(self, min_amount)
    }
}

// --- Test modules for aptos_framework natives ---

module 0xa002::event_test {
    use aptos_framework::event;

    #[event]
    struct TestEvent has drop, store {
        value: u64,
    }

    public entry fun test_event_emit(_account: &signer) {
        // Events are no-ops — just verify they don't crash
        event::emit(TestEvent { value: 42 });
        event::emit(TestEvent { value: 100 });
    }
}

module aptos_framework::create_signer_test {
    use aptos_framework::create_signer;
    use std::signer;

    public entry fun test_create_signer(_account: &signer) {
        let addr = @0x42;
        let s = create_signer::create_signer(addr);
        assert!(signer::address_of(&s) == addr, 1);
    }

    public entry fun test_create_signer_zero(_account: &signer) {
        let addr = @0x0;
        let s = create_signer::create_signer(addr);
        assert!(signer::address_of(&s) == addr, 1);
    }
}

module 0xa002::transaction_context_test {
    use aptos_framework::transaction_context;

    public entry fun test_chain_id(_account: &signer) {
        let chain_id = transaction_context::get_chain_id();
        assert!(chain_id == 4, 1); // test chain_id is 4
    }

    public entry fun test_get_txn_hash(_account: &signer) {
        let hash = transaction_context::get_transaction_hash();
        assert!(std::vector::length(&hash) == 32, 1); // hash should be 32 bytes
    }

    public entry fun test_get_script_hash(_account: &signer) {
        let hash = transaction_context::get_script_hash_public();
        assert!(std::vector::length(&hash) == 0, 1); // empty in test env
    }

    public entry fun test_gas_amounts(_account: &signer) {
        let max_gas = transaction_context::get_max_gas_amount();
        assert!(max_gas > 0, 1);
        let gas_price = transaction_context::get_gas_unit_price();
        assert!(gas_price > 0, 2);
    }

    public entry fun test_sender(_account: &signer) {
        let sender = transaction_context::get_sender();
        // Just check it doesn't crash and returns a valid address
        let _ = sender;
    }

    public entry fun test_secondary_signers(_account: &signer) {
        let signers = transaction_context::get_secondary_signers();
        assert!(std::vector::length(&signers) == 0, 1); // empty in test env
    }
}

module 0xa002::aggregator_test {
    use aptos_framework::aggregator_v2;

    public entry fun test_aggregator_basic(_account: &signer) {
        let agg = aggregator_v2::create_aggregator<u64>(100);
        assert!(aggregator_v2::read(&agg) == 0, 1);
        assert!(aggregator_v2::try_add(&mut agg, 50), 2);
        assert!(aggregator_v2::read(&agg) == 50, 3);
        assert!(aggregator_v2::try_add(&mut agg, 30), 4);
        assert!(aggregator_v2::read(&agg) == 80, 5);
    }

    public entry fun test_aggregator_overflow(_account: &signer) {
        let agg = aggregator_v2::create_aggregator<u64>(100);
        assert!(aggregator_v2::try_add(&mut agg, 90), 1);
        assert!(!aggregator_v2::try_add(&mut agg, 20), 2); // would exceed max
        assert!(aggregator_v2::read(&agg) == 90, 3); // unchanged
    }

    public entry fun test_aggregator_sub(_account: &signer) {
        let agg = aggregator_v2::create_aggregator<u64>(100);
        assert!(aggregator_v2::try_add(&mut agg, 50), 1);
        assert!(aggregator_v2::try_sub(&mut agg, 30), 2);
        assert!(aggregator_v2::read(&agg) == 20, 3);
        assert!(!aggregator_v2::try_sub(&mut agg, 30), 4); // underflow
        assert!(aggregator_v2::read(&agg) == 20, 5); // unchanged
    }

    public entry fun test_aggregator_snapshot(_account: &signer) {
        let agg = aggregator_v2::create_aggregator<u64>(100);
        assert!(aggregator_v2::try_add(&mut agg, 42), 1);
        let snap = aggregator_v2::snapshot(&agg);
        assert!(aggregator_v2::read_snapshot(&snap) == 42, 2);

        let snap2 = aggregator_v2::create_snapshot<u64>(99);
        assert!(aggregator_v2::read_snapshot(&snap2) == 99, 3);
    }

    public entry fun test_aggregator_u128(_account: &signer) {
        let agg = aggregator_v2::create_aggregator<u128>(1000);
        assert!(aggregator_v2::try_add(&mut agg, 500), 1);
        assert!(aggregator_v2::read(&agg) == 500, 2);
        assert!(aggregator_v2::try_sub(&mut agg, 200), 3);
        assert!(aggregator_v2::read(&agg) == 300, 4);
        assert!(aggregator_v2::is_at_least(&agg, 200), 5);
        assert!(!aggregator_v2::is_at_least(&agg, 400), 6);
    }

    public entry fun test_aggregator_unbounded(_account: &signer) {
        let agg = aggregator_v2::create_unbounded_aggregator<u64>();
        assert!(aggregator_v2::try_add(&mut agg, 18446744073709551615), 1); // u64::MAX
        assert!(aggregator_v2::read(&agg) == 18446744073709551615, 2);
    }
}

