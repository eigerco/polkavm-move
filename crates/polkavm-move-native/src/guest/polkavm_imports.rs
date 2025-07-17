extern crate alloc;
use alloc::boxed::Box;

// PolkaVM will call this function to execute the program.
// We need to load the call data and pass it to the selector function.
#[polkavm_derive::polkavm_export]
unsafe extern "C" fn call() {
    let size = call_data_size();
    let mut buf = Box::new_uninit_slice(size as usize).assume_init();
    let out_ptr = buf.as_mut_ptr();
    call_data_copy(out_ptr, size as u32, 0);
    call_selector(out_ptr, size);
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

// The call_selector is generated during translation
extern "C" {
    pub(crate) fn call_selector(buf: *mut u8, size: u64);
}
