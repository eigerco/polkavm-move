module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x10::debug;
    use std::vector;

    public entry fun vecnew(): u64 {
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

    public entry fun vecisempty(): bool {
        let v = vector::empty<u8>();
        assert!(vector::is_empty(&v), 0);
        vector::push_back(&mut v, 0u8);
        let empty = vector::is_empty(&v);
        debug::print(&empty);
        assert!(!empty, 0);
        empty
    }

    public entry fun veccmp(): bool {
        let v1 = x"616263";
        let v2 = vector::empty<u8>();
        vector::push_back(&mut v2, 97u8);
        vector::push_back(&mut v2, 98u8);
        vector::push_back(&mut v2, 99u8);
        assert!(vector::length(&v1) == vector::length(&v2), 0);
        assert!(v1 == v2, 0);
        let eq = v1 == v2;
        eq
    }

}
