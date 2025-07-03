module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x10::debug;
    use std::signer;

    public entry fun main_basic(account: &signer) {
        let rv = 17;
        debug::print(&rv);
        let address = signer::address_of(account);
        debug::print(&address);
    }

    public entry fun abort_with_code(code: u64) {
        abort code
    }
}
