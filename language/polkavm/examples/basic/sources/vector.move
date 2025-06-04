module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x10::debug;
    use std::vector;

    public entry fun foo(): u64 {
        let v = vector::empty<u8>();
        vector::push_back(&mut v, 0u8);
        vector::push_back(&mut v, 1u8);
        vector::push_back(&mut v, 2u8);
        vector::push_back(&mut v, 3u8);
        vector::push_back(&mut v, 4u8);
        vector::push_back(&mut v, 5u8);
        vector::push_back(&mut v, 6u8);
        vector::push_back(&mut v, 7u8);
        vector::push_back(&mut v, 8u8);
        vector::push_back(&mut v, 9u8);
        debug::print(&v);
        let e = vector::borrow(&mut v, 2);
        let b = (*e as u64);
        debug::print(&b);
        b
    }

}
