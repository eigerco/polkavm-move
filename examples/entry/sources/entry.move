module 0x10::debug {
  native public fun print<T>(x: &T);
}

module 0xc::token {
    struct Token has store {
        owner: address
    }
}

module 0xe::entry_bar {
    use 0x10::debug;

    struct Coin<T> has key {
        token: T,
        value: u64,
    }

    public entry fun bar<T: store>(coin: &Coin<T>) {
        let rv = coin.value;
        debug::print(&rv);
    }
}
