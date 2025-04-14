module 0x1234::tuplebasic {
    // this should be enough to check tuple structuring/destructuring
    // TODO(tadas): removing public keyword compiles just fine, but LLVM phase still exports this function even if its marked as private
    public fun giveMeTuple(a: u32, b: u64): (u32, u64) {
        (a, b)
    }

    public entry fun add(a: u32, b: u64): u64 {
        let (x, y) = giveMeTuple(a, b);
        // we cannot use any arithmetic ops here which can cause under/over flow as move compiler
        // automatically inserts move_rt_abort fn call on such cases - we are not ready for this yet
        (x as u64) + y
    }
}
