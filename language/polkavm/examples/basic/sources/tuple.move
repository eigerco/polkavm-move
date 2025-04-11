module 0x1234::tuplebasic {
    // this should be enough to check tuple structuring/destructuring
    fun giveMeTuple(a: u64, b: u32): (u64, u32) {
        (a, b)
    }

    public entry fun multiply(a: u64, b: u32): u64 {
        let (x, y) = giveMeTuple(a, b);
        x * (y as u64)
    }
}
