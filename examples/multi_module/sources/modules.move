module 0x7::A {

    public fun add(a: u32, b: u32): u32 {
        a + b
    }

}

module 0x7::B {
    use 0x7::A;

    fun add_all(a: u32, b: u32, c: u32): u32 {
        let res = A::add(a, b);
        A::add(res, c)
    }

    public entry fun main(account: &signer) {
        let a: u32 = 10;
        let b: u32 = 20;
        let c: u32 = 30;
        let res = add_all(a, b, c);
        assert!(res == 60, 0x1001);
    }
}
