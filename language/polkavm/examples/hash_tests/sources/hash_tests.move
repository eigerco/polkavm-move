module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa002::hash_tests {
    use 0x10::debug;
    use std::vector;
    use std::hash;

    public entry fun sha2_256_expected_hash() {
        let input = x"616263";
        debug::print(&input);
        let len = vector::length(&input);
        assert!(len == 3, 0);
        debug::print(&len);
        let digest = hash::sha2_256(input);
        debug::print(&digest);
        let digest_len = vector::length(&digest);
        //assert!(digest_len == 24, 0);
        debug::print(&digest_len);
        let expected_output = x"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(digest == expected_output, 0);
    }

    public entry fun sha3_256_expected_hash() {
        let input = x"616263";
        debug::print(&input);
        let expected_output = x"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532";
        assert!(hash::sha3_256(input) == expected_output, 0);
    }
}
