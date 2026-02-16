module 0xa002::hash_tests {
    use aptos_std::aptos_hash;

    // Note: sha2_512/sha3_512 have feature-gate checks that read from global
    // storage (Features resource), which is not available in PolkaVM.
    // Use keccak256 which is a direct native without feature gates.

    public entry fun keccak256_expected_hash(_account: &signer) {
        let input = x"616263";
        let digest = aptos_hash::keccak256(input);
        let expected_output = x"4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45";
        assert!(digest == expected_output, 0);
    }
}

