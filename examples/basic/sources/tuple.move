module 0x1234::tuplebasic {
    // this should be enough to check tuple structuring/destructuring
    fun giveMeTuple(a: u32, b: u64): (u32, u64) {
        (a, b)
    }

    fun add(a: u32, b: u64): u64 {
        let (x, y) = giveMeTuple(a, b);
        (x as u64) + y
    }

    public entry fun main() {
        let a: u32 = 10;
        let b: u64 = 20;
        let res = add(a, b);
        assert!(res == 30, 0x1001);
    }
}
