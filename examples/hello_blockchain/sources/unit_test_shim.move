// Minimal unit_test module — normally #[test_only] in stdlib
module std::unit_test {
    native public fun create_signers_for_testing(num_signers: u64): vector<signer>;
}
