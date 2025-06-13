module 0xa000::basic {

    public fun div(a: u64, b:u64): u64 {
        assert!(b != 0, 0x1001);
        a / b
    }

    public fun mul(a: u64, b:u64): u64 {
        a * b
    }
}
