module 0x1234::structbasic {
    struct Counter has copy, drop, store {
        value: u64,
    }

    public entry fun create_counter(x: u64, y: u64): u64 {
        let c = Counter { value: x + y };
        c.value
    }
}
