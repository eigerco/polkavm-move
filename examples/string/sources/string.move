module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa004::my_string {
    use 0x10::debug;
    use std::string;

    public entry fun foo() {
        let rv = b"Hello, PolkaVM!";
        let str = string::utf8(rv);
        debug::print(&rv);
        debug::print(&str);
    }

    public entry fun index_of() {
        let rv = b"Hello, PolkaVM!";
        let str = string::utf8(rv);
        let p = string::utf8(b"P");
        let i = string::index_of(&str, &p);
        assert!(i == 7, 0);
    }

    public entry fun substring() {
        let rv = b"Hello, PolkaVM!";
        let str = string::utf8(rv);
        debug::print(&str);
        let sub = string::sub_string(&str, 7, 14);
        debug::print(&sub);
        let polka = string::utf8(b"PolkaVM");
        assert!(polka == sub, 0);
    }

    public entry fun append() {
        let b = b"Hello, PolkaVM!";
        let str = string::utf8(b);
        let b2 = b" How are you?";
        let str2 = string::utf8(b2);
        string::append(&mut str, str2);
        debug::print(string::bytes(&str));
        assert!(string::length(&str) == 28, 0);
    }

    public entry fun insert() {
        let b = b"Hello, PolkaVM?";
        let str = string::utf8(b);
        let b2 = b" How are you,";
        let str2 = string::utf8(b2);
        string::insert(&mut str, 6, str2);
        debug::print(string::bytes(&str));
        assert!(string::length(&str) == 28, 0);
        let h = string::utf8(b"Ho");
        let i = string::index_of(&str, &h);
        debug::print(&i);
        assert!(i == 7, 0);
    }

}

