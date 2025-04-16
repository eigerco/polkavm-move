module multi_module::A {

    public fun add(a: u32, b: u32): u32 {
        a + b
    }

}

module multi_module::B {
    use multi_module::A;

    public entry fun add_all(a: u32, b: u32, c: u32): u32 {
        let res = A::add(a, b);
        A::add(res, c)
    }
}
