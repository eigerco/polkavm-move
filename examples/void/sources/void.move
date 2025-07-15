module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xb000::void {
    use std::signer;
    use std::vector;
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

    fun store(account: &signer) {
        //let container = Container { value: 42, inner: Containee { value: 69, s: x"cafebabe" } };
        //move_to(account, container);
        let exists = exists<Container>(signer::address_of(account));
        debug::print(&exists);
        assert!(!exists, 1);
    }

    public entry fun main_void(account: &signer) {
        store(account);
    }
}
