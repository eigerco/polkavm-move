module 0xb000::void {
    use std::vector;

    public entry fun deploy() {
    }
    public entry fun call() {
        let v = vector::empty<u8>();
        vector::push_back(&mut v, 0u8);
    }
}
