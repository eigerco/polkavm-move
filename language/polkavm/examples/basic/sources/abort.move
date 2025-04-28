module 0x5678::MyAbortBasic {
    public entry fun my_abort_example(x: u64, y: u64): u64 {
        assert!(x != y, 69);
        x + y
    }
}

