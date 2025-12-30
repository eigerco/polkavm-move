module 0x10::debug {
    native public fun print<T>(x: &T);
}
module 0x42::entry {
    use 0x10::debug;
    use TaoHe::root;

    struct MyContent has store, key{
        id: u64,
    }

    public entry fun main(account: &signer) {
        let content = MyContent { id: 1 };
        debug::print(&content);
        root::create(account, content);
        debug::print(&b"done");
    }
}
