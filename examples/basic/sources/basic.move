module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x10::debug;
    use std::signer;

    public fun bar(): u64 {
        let rv = 19;
        debug::print(&rv);
        rv
    }

    public fun foo(account: &signer): u64 {
        let rv = 17;
        debug::print(&rv);
        debug::print(&signer::address_of(account));
        rv
    }

    public fun abort_with_code(code: u64) {
        abort code
    }
}
