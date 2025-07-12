module 0xb000::void {
    use 0x1::vector;

    public entry fun main_void() {
        let x = vector::empty<u64>();
        vector::push_back(&mut x, 42);
    }
}
