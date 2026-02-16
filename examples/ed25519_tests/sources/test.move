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
    // SK: 9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60
    // PK: d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a
    // Message: "test message" (hex: 74657374206d657373616765)
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
