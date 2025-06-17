module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::storage {
    use 0x10::debug;
    use std::signer;

    struct Container has key, drop, store {
        value: u64,
    }

    public entry fun store(account: &signer) {
        let container = Container { value: 42 };
        debug::print(&container);
        debug::print(account);
        move_to(account, container);
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(exists, 1);
    }

    public entry fun load(account: &signer) acquires Container {
        let address = signer::address_of(account);
        let container: Container = move_from(address);
        debug::print(&container);
        assert!(container.value == 42, 0);
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(!exists, 0);
    }
}

