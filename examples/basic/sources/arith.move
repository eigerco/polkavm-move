module 0xa001::arith {

    fun div(a: u64, b:u64): u64 {
        a / b
    }

    fun mul(a: u64, b:u64): u64 {
        a * b
    }

    public entry fun main_arith() {
        let mul = mul(10, 20);
        assert!(mul == 200, 0x1002);
        let div = div(100, 20);
        assert!(div == 5, 0x1003);
    }

    public entry fun abort_on_div_by_zero() {
        div(100, 0);
    }

}
