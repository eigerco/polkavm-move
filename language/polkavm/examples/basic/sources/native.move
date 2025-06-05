module 0x11::nat {
    native public fun get_vec(): vector<u8>;
}

module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x11::nat;
    use 0x10::debug;
    use std::vector;

    public entry fun native_get_vec() {
        let v = nat::get_vec();
        debug::print(&vector::length(&v));
        debug::print(vector::borrow(&mut v, 0));
        debug::print(vector::borrow(&mut v, 1));
    }
}

