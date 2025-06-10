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

    public entry fun index_of(): u64 {
        let rv = b"Hello, PolkaVM!";
        let str = string::utf8(rv);
        let p = string::utf8(b"P");
        let i = string::index_of(&str, &p);
        i
    }

    public entry fun substring() {
        let rv = b"Hello, PolkaVM!";
        let str = string::utf8(rv);
        debug::print(&str);
        let sub = string::sub_string(&str, 7, 14);
        debug::print(&sub);
    }

}

