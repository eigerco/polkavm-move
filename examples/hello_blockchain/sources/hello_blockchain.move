module hello_blockchain::message {
    use std::error;
    use std::signer;
    use std::string;
    use std::vector;
    use std::unit_test;
    use aptos_framework::event;

    struct MessageHolder has key {
        message: string::String,
    }

    #[event]
    struct MessageChange has drop, store {
        account: address,
        from_message: string::String,
        to_message: string::String,
    }

    /// There is no message present
    const ENO_MESSAGE: u64 = 0;

    #[view]
    public fun get_message(addr: address): string::String acquires MessageHolder {
        assert!(exists<MessageHolder>(addr), error::not_found(ENO_MESSAGE));
        borrow_global<MessageHolder>(addr).message
    }

    public entry fun set_message(account: &signer, message: string::String)
    acquires MessageHolder {
        let account_addr = signer::address_of(account);
        if (!exists<MessageHolder>(account_addr)) {
            move_to(account, MessageHolder {
                message,
            })
        } else {
            let old_message_holder = borrow_global_mut<MessageHolder>(account_addr);
            let from_message = old_message_holder.message;
            event::emit(MessageChange {
                account: account_addr,
                from_message,
                to_message: copy message,
            });
            old_message_holder.message = message;
        }
    }

    // === Test entry points ===

    public entry fun test_noop() {
        // Trivial test to isolate compilation issues
    }

    public entry fun test_create_signer() {
        let _signers = unit_test::create_signers_for_testing(1);
    }

    public entry fun test_basic_assert() {
        let x: u64 = 42;
        assert!(x == 42, 1);
    }

    public entry fun test_set_only(account: &signer) {
        set_message(account, string::utf8(b"Hello, Blockchain"));
    }

    public entry fun test_set_and_get(account: &signer) {
        let addr = signer::address_of(account);
        set_message(account, string::utf8(b"Hello, Blockchain"));
        assert!(
            get_message(addr) == string::utf8(b"Hello, Blockchain"),
            ENO_MESSAGE
        );
    }
}
