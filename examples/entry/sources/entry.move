module 0x10::debug {
  native public fun print<T>(x: &T);
}

module 0xc::token {
    public struct Token has store, drop {
        owner: address
    }
    public fun new(owner: address): Token {
        Token { owner }
    }
}

module 0xe::entry_bar {
    use 0x10::debug;

    public struct Coin<T> has drop {
        token: T,
        value: u64,
    }

    fun bar<T: store>(coin: &Coin<T>) {
        let rv = coin.value;
        debug::print(&rv);
    }

    public entry fun main() {
        let t = 0xc::token::new(@0x1);
        let coin = Coin { token: t, value: 100 };
        bar(&coin);
    }
}
