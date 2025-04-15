module 0x5678::AbortBasic {
    const EQUAL_VALUES_ERROR: u64 = 1;

    public fun abort_example(x: u64, y: u64): u64 {
        assert!(x != y, EQUAL_VALUES_ERROR);
        x + y
    }
}

