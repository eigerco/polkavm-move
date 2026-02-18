/// Minimal event shim — events are fire-and-forget logging, no-op here.
module aptos_framework::event {
    public fun emit<T: drop + store>(_msg: T) {}
}
