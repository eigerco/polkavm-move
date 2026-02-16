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

