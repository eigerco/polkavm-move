module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x10::debug;
    use std::error;

    public entry fun error(): u64 {
        let rv = error::not_found(42);
        debug::print(&rv);
        rv
    }

}

