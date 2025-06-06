module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x10::debug;
    use std::string;

    public entry fun foo() {
        let rv = b"Hello, PolkaVM!";
        let str = string::utf8(rv);
        debug::print(&rv);
        debug::print(&str);
    }

}

