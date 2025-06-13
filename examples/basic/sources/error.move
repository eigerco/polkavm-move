module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::basic {
    use 0x10::debug;
    use std::error;

    public entry fun error() {
        let rv = error::not_found(42);
        debug::print(&rv);
        let expected_output = 393258;
        assert!(rv == expected_output, 0);
    }

}

