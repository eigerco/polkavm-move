module 0xca11::functions {
    public entry fun sum(a: u64, b: u64): u64 {
        a + b
    }

    public entry fun sum_plus_const_5(a: u64, b: u64): u64 {
        a + b + 5
    }

    public entry fun sum_of_3(a: u64, b: u64, c: u64): u64 {
        a + b + c
    }

    public entry fun sum_for_rich(a: u64, b: u64, c: u64): u64 {
        let yes_give_more_if_im_rich: bool = if (a > 5) true else false;

        let c = sum_of_3(a, b, c);
        if (yes_give_more_if_im_rich) {
            c + 100
        } else { c }
    }

    /*
    requires abort native call support
    public entry fun substract(a: u64, b: u64): u64 {
        a - b
    }
    */

    /*
    Error: found undefined symbol: '__muldi3'
    public entry fun multiply(a: u64, b: u64): u64 {
        a * b
    }
    */

    public entry fun sum_different_size_args(a: u32, b: u64, c: u32): u64 {
        (a as u64) + b + (c as u64)
    }

    public entry fun sum_if_extras(a: u32, no_extras: bool, b: u64): u64 {
        let c = sum(a as u64, b);
        if (!no_extras) {
            sum(c, 100)
        } else { c }
    }
}
