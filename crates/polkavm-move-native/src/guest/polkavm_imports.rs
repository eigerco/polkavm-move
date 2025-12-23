extern crate alloc;
use alloc::boxed::Box;

// PolkaVM will call this function to execute the program.
// We need to load the call data and pass it to the selector function.
#[polkavm_derive::polkavm_export]
unsafe extern "C" fn call() {
    // 4 bytes for selector, 20 bytes for origin, rest padding
    let mut buf = Box::new_uninit_slice(36).assume_init();
    // a buffer for the origin
    let out_ptr = buf.as_mut_ptr();
    call_data_copy(out_ptr, 4, 0);
    let signer_ptr = unsafe { out_ptr.add(4) }; // Skip first 4 bytes
    origin(signer_ptr);
    call_selector(out_ptr, 36);
}

#[polkavm_derive::polkavm_export]
unsafe extern "C" fn deploy() {}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn call_data_copy(out_ptr: *mut u8, out_len: u32, offset: u64);
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn call_data_size() -> u64;
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn origin(buf: *mut u8);
}

// The call_selector is generated during translation
extern "C" {
    pub(crate) fn call_selector(buf: *mut u8, size: u64);
}
