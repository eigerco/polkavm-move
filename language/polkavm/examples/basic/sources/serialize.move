module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x10::debug;
    use std::bcs;
    use std::string;

    public entry fun ser_signer(account: &signer) {
        let bytes = bcs::to_bytes(account);
        debug::print(&bytes);
        let expected_output = x"ab010101010101010101010101010101010101010101010101010101010101ce";
        assert!(bytes == expected_output, 0);
    }

    public entry fun ser_string() {
        let rv = b"Hello, PolkaVM!";
        let str = string::utf8(rv);
        let bytes = bcs::to_bytes(&str);
        debug::print(&bytes);
        let expected_output = x"0f00000048656c6c6f2c20506f6c6b61564d21";
        assert!(bytes == expected_output, 0);
    }
}
