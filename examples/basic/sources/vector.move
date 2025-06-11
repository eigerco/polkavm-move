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
        let other = vector::empty<u8>();
        vector::push_back(&mut other, 10u8);
        vector::append(&mut v, other);
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

    public entry fun singleton() {
        let v = vector::singleton<u8>(42);
        assert!(vector::length(&v) == 1, 0);
        let first = vector::borrow_mut(&mut v, 0);
        *first = 43u8;
        assert!(*vector::borrow(&v, 0) == 43u8, 0);
    }

    public entry fun popback() {
        let v = x"616263";
        let last = vector::pop_back(&mut v);
        assert!(vector::length(&v) == 2, 0);
        assert!(last == 99u8, 0);
    }

    public entry fun reverse() {
        let v = x"616263";
        vector::reverse(&mut v);
        assert!(vector::length(&v) == 3, 0);
        assert!(*vector::borrow(&v, 0) == 99u8, 0);
    }

    public entry fun contains() {
        let v = x"616263";
        assert!(!vector::contains(&v, &3u8), 0);
        assert!(vector::contains(&v, &97u8), 0);
    }

    public entry fun swapremove() {
        let v1 = x"616263";
        let e = vector::swap_remove(&mut v1, 1);
        assert!(e == 98u8, 0);
        let v2 = x"6163";
        assert!(v1 == v2, 0);
    }

    public entry fun remove() {
        let v1 = x"616263";
        let e = vector::remove(&mut v1, 1);
        assert!(e == 98u8, 0);
        let v2 = x"6163";
        assert!(v1 == v2, 0);
    }

    public entry fun indexof() {
        let v = x"616263";
        let (b, i) = vector::index_of(&v, &3u8);
        assert!(!b, 0);
        assert!(i == 0, 0);
        let (b, i) = vector::index_of(&v, &97u8);
        assert!(b, 0);
        assert!(i == 0, 0);
        let (b, i) = vector::index_of(&v, &98u8);
        assert!(b, 0);
        assert!(i == 1, 0);
    }

    public entry fun foreach() {
        let v = x"616263";
        vector::for_each(v, |e| debug::print(&e));
    }

    public entry fun foreachref() {
        let v = x"818283";
        vector::for_each_ref(&v, |e| debug::print(e));
    }

    public entry fun fold() {
        let v = x"010203";
        let sum = 0u8;
        sum = vector::fold(v, sum, |sum, e| sum + e);
        assert!(sum == 6u8, 0);
    }

    public entry fun map() {
        let v = x"010203";
        let v2 = vector::map(v, |e| e * 2);
        assert!(*vector::borrow(&v2, 2) == 6u8, 0);
    }

    public entry fun filter() {
        let v = x"0102030405060708090a0b0c0d0e0f";
        let v2 = vector::filter(v, |e| *e > 5);
        assert!(*vector::borrow(&v2, 0) == 6u8, 0);
    }
}
