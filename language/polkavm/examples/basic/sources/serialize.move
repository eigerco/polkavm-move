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
    }

    public entry fun ser_string() {
        let rv = b"Hello, PolkaVM!";
        let str = string::utf8(rv);
        let bytes = bcs::to_bytes(&str);
        debug::print(&bytes);
    }
}
