module 0x42::entry {
    use std::signer;
    use TaoHe::root;

    struct MyContent has store, key{
        id: u64,
    }

    public entry fun main(account: &signer) {
        let content = MyContent { id: 1 };
        root::create(account, content);
    }
}
