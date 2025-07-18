module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::my_vector {
    use 0x10::debug;
    use std::vector;

    public entry fun vecnew(account: &signer) {
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
        assert!(vector::length(&v) == 11, 0);
        let e = vector::borrow(&mut v, 2);
        let b = (*e as u64);
        assert!(b == 2, 0);
    }

    public entry fun vecisempty(account: &signer) {
        let v = vector::empty<u8>();
        assert!(vector::is_empty(&v), 0);
        vector::push_back(&mut v, 0u8);
        let empty = vector::is_empty(&v);
        assert!(!empty, 0);
    }

    public entry fun veccmp(account: &signer) {
        let v1 = x"616263";
        let v2 = vector::empty<u8>();
        vector::push_back(&mut v2, 97u8);
        vector::push_back(&mut v2, 98u8);
        vector::push_back(&mut v2, 99u8);
        assert!(vector::length(&v1) == vector::length(&v2), 0);
        assert!(v1 == v2, 0);
    }

    public entry fun singleton(account: &signer) {
        let v = vector::singleton<u8>(42);
        assert!(vector::length(&v) == 1, 0);
        let first = vector::borrow_mut(&mut v, 0);
        *first = 43u8;
        assert!(*vector::borrow(&v, 0) == 43u8, 0);
    }

    public entry fun popback(account: &signer) {
        let v = x"616263";
        let last = vector::pop_back(&mut v);
        assert!(vector::length(&v) == 2, 0);
        assert!(last == 99u8, 0);
    }

    public entry fun reverse(account: &signer) {
        let v = x"616263";
        vector::reverse(&mut v);
        assert!(vector::length(&v) == 3, 0);
        assert!(*vector::borrow(&v, 0) == 99u8, 0);
    }

    public entry fun contains(account: &signer) {
        let v = x"616263";
        assert!(!vector::contains(&v, &3u8), 0);
        assert!(vector::contains(&v, &97u8), 0);
    }

    public entry fun swapremove(account: &signer) {
        let v1 = x"616263";
        let e = vector::swap_remove(&mut v1, 1);
        assert!(e == 98u8, 0);
        let v2 = x"6163";
        assert!(v1 == v2, 0);
    }

    public entry fun remove(account: &signer) {
        let v1 = x"616263";
        let e = vector::remove(&mut v1, 1);
        assert!(e == 98u8, 0);
        let v2 = x"6163";
        assert!(v1 == v2, 0);
    }

    public entry fun indexof(account: &signer) {
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

    public entry fun foreach(account: &signer) {
        let v = x"616263";
        vector::for_each(v, |e| debug::print(&e));
    }

    public entry fun foreachref(account: &signer) {
        let v = x"818283";
        vector::for_each_ref(&v, |e| debug::print(e));
    }

    public entry fun fold(account: &signer) {
        let v = x"010203";
        let sum = 0u8;
        sum = vector::fold(v, sum, |sum, e| sum + e);
        assert!(sum == 6u8, 0);
    }

    public entry fun map(account: &signer) {
        let v = x"010203";
        let v2 = vector::map(v, |e| e * 2);
        assert!(*vector::borrow(&v2, 2) == 6u8, 0);
    }

    public entry fun filter(account: &signer) {
        let v = x"0102030405060708090a0b0c0d0e0f";
        let v2 = vector::filter(v, |e| *e > 5);
        assert!(*vector::borrow(&v2, 0) == 6u8, 0);
    }
}
