use crate::types::{AnyValue, MoveAddress, MoveSigner, MoveType};

mod allocator;
mod imports;
mod panic;

// FIXME(tadas) LLVM code generation shouldn't mangle native function libs
#[export_name = "_ZN6native13move_rt_abort17h0d65d6cb873e6403E"]
unsafe extern "C" fn move_rt_abort(code: u64) {
    imports::abort(code);
}

#[export_name = "_ZN6native23move_native_debug_print17hf268c585c096f4dcE"]
unsafe extern "C" fn print(type_x: *const MoveType, x: *const AnyValue) {
    imports::debug_print(type_x, x);
}

#[export_name = "_ZN6native33move_native_signer_borrow_address17ha48b6bb6485a828bE"]
extern "C" fn borrow_address(s: &MoveSigner) -> &MoveAddress {
    &s.0
}
