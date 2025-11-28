module 0xa002::hash_tests {
    use std::vector;
    use aptos_std::aptos_hash;

    public entry fun sha2_512_expected_hash(account: &signer) {
        let input = x"616263";
        let digest = aptos_hash::sha2_512(input);
        let expected_output = x"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(digest == expected_output, 0);
    }

    public entry fun sha3_256_expected_hash(account: &signer) {
        let input = x"616263";
        let expected_output = x"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532";
        assert!(aptos_hash::sha3_512(input) == expected_output, 0);
    }
}

