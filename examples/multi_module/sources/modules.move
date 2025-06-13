module 0x10::debug {
  native public fun print<T>(x: &T);
}

module multi_module::A {
    use 0x10::debug;
    use std::signer;

    public entry fun bar() {
        let rv = 19;
        debug::print(&rv);
    }

    public entry fun foo(account: &signer) {
        let rv = 17;
        debug::print(&rv);
        debug::print(&signer::address_of(account));
    }
}

module multi_module::B {
    use 0x10::debug;

    public entry fun foo_bar() {
        let val = 42;
        debug::print(&val);
    }
}
