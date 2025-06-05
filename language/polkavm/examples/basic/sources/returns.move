module 0x10::debug {
    native public fun print<T>(x: &T);
}

module 0xa000::morebasic {
    use 0x10::debug;

    public entry fun rv_bool_false(): bool {
        let rv = false;
        debug::print(&rv);
        rv
    }

    public entry fun rv_bool_true(): bool {
        let rv = true;
        debug::print(&rv);
        rv
    }

    public entry fun rv_u8(): u8 {
        let rv = 19u8;
        debug::print(&rv);
        rv
    }

    public entry fun rv_u16(): u16 {
        let rv = 19u16;
        debug::print(&rv);
        rv
    }

    public entry fun rv_u32(): u32 {
        let rv = 19u32;
        debug::print(&rv);
        rv
    }
}
