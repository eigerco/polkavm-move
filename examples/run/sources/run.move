module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa009::run {
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
        move_to(account, container);
        let exists = exists<Container>(signer::address_of(account));
        assert!(exists, 1);
        debug::print(&exists);
    }

    public entry fun load(account: &signer) acquires Container {
        let address = signer::address_of(account);
        let container: Container = move_from(address);
        assert!(container.value == 42, 0);
        let expected = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        assert!(container == expected, 1);
        let exists = exists<Container>(signer::address_of(account));
        assert!(!exists, 1);
        let s = b"done";
        debug::print(&s);
    }

    public entry fun pvm_start(account: &signer) acquires Container {
       debug::print(account);
       store(account);
        let s = b"stored";
        debug::print(&s);
       load(account);
    }
}

