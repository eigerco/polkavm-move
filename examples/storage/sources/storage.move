module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::storage {
    use 0x10::debug;
    use std::signer;

    struct Containee has key, drop, store, copy {
        value: u64,
        s: vector<u8>,
    }

    struct Container has key, drop, store {
        value: u64,
        inner: Containee,
    }

    public entry fun store(account: &signer) {
        let container = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        debug::print(&container);
        debug::print(account);
        move_to(account, container);
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(exists, 1);
    }

    public entry fun store_twice(account: &signer) {
        let container = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        move_to(account, container);
        let container2 = Container { value: 69, inner: Containee { value: 42, s: x"DEADBEEF" } };
        move_to(account, container2); // this should abort
    }

    public entry fun load(account: &signer) acquires Container {
        let address = signer::address_of(account);
        let container: Container = move_from(address);
        debug::print(&container);
        assert!(container.value == 42, 0);
        let expected = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        assert!(container == expected, 1);
        let exists = exists<Container>(signer::address_of(account));
        assert!(!exists, 0);
    }

    public entry fun borrow(account: &signer) acquires Container {
        let address = signer::address_of(account);
        let container = borrow_global<Container>(address);
        debug::print(container);
        assert!(container.value == 42, 0);
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(!exists, 1);
    }

    public entry fun load_non_existent(account: &signer) acquires Container {
        let address = signer::address_of(account);
        let _container: Container = move_from(address); // should abort
    }
}

