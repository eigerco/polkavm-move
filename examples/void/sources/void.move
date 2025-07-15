module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xb000::void {
    use std::signer;
    use std::vector;
    use std::hash;
    use 0x10::debug;

    struct Containee has key, drop, store, copy {
        value: u64,
        s: vector<u8>,
    }

    struct Container has key, drop, store, copy {
        value: u64,
        inner: Containee,
    }

    struct Another has key, drop, store, copy {
        first: u64,
        second: u64,
    }

    fun store(account: &signer) acquires Container {
        let address = signer::address_of(account);
        let container = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        debug::print(&container);
        debug::print(account);
        move_to(account, container);
        let container = borrow_global<Container>(address);
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(exists, 1);
    }

    public entry fun main_void(account: &signer) acquires Container {
        let input = x"616263";
        debug::print(&input);
        let len = vector::length(&input);
        assert!(len == 3, 0);
        debug::print(&len);
        let digest = hash::sha2_256(input);
        debug::print(&digest);
        store(account);
    }
}
