module 0x10::debug {
    native public fun print<T>(x: &T);
    native public fun hex_dump();
}

module 0xa000::storage {
    use 0x10::debug;
    use std::signer;

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

    public entry fun store(account: &signer) {
        let container = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        debug::print(&container);
        debug::print(account);
        move_to(account, container);
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(exists, 1);
    }

    public entry fun store2(account: &signer) {
        let container = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        debug::print(&container);
        debug::print(account);
        move_to(account, container);
        let cont_exists = exists<Container>(signer::address_of(account));
        debug::print(&cont_exists);
        assert!(cont_exists, 1);
        let another = Another { first: 1, second: 2 };
        move_to(account, another);
        let exists_another: bool = exists<Another>(signer::address_of(account));
        debug::print(&exists_another);
        assert!(exists_another, 2);
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

    public entry fun load2(account: &signer) acquires Container, Another {
        let address = signer::address_of(account);
        let container: Container = move_from(address);
        debug::print(&container);
        assert!(container.value == 42, 0);
        let expected = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        assert!(container == expected, 1);
        let exists = exists<Container>(signer::address_of(account));
        assert!(!exists, 0);
        let another: Another = move_from(address);
        debug::print(&another);
        assert!(another.first == 1, 2);
    }

    public entry fun borrow(account: &signer) acquires Container {
        let address = signer::address_of(account);
        let container = borrow_global<Container>(address);
        debug::print(container);
        debug::hex_dump();
        assert!(container.value == 42, 0);
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(!exists, 1);
    }

    public entry fun load_non_existent(account: &signer) acquires Container {
        let address = signer::address_of(account);
        let _container: Container = move_from(address); // should abort
    }

    public entry fun does_not_exist(account: &signer) {
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(!exists, 1);
    }
}

