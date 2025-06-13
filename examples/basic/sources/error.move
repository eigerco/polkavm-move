module 0xa002::error {
    use std::error;

    public entry fun error() {
        let rv = error::not_found(42);
        let expected_output = 393258;
        assert!(rv == expected_output, 0);
    }

}

