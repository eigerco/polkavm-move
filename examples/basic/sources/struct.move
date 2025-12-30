module 0x1234::structbasic {
    struct Counter has copy, drop, store {
        value: u64,
    }

    fun create_counter(x: u64, y: u64): u64 {
        let c = Counter { value: x + y };
        c.value
    }

    public entry fun main_struct(_account: &signer) {
        let x: u64 = 10;
        let y: u64 = 20;
        let res = create_counter(x, y);
        assert!(res == 30, 0x1001);
    }
}
