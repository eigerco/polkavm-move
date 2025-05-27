use crate::types::{AnyValue, MoveAddress, MoveSigner, MoveType};

mod allocator;
mod imports;
mod panic;

#[export_name = "move_rt_abort"]
unsafe extern "C" fn move_rt_abort(code: u64) {
    imports::abort(code);
}

#[export_name = "move_rt_vec_empty"]
unsafe extern "C" fn move_rt_vec_empty() {}

#[export_name = "move_rt_vec_copy"]
unsafe extern "C" fn move_rt_vec_copy() {}

#[export_name = "move_native_debug_print"]
unsafe extern "C" fn print(type_x: *const MoveType, x: *const AnyValue) {
    imports::debug_print(type_x, x);
}

#[export_name = "move_native_signer_borrow_address"]
extern "C" fn borrow_address(s: &MoveSigner) -> &MoveAddress {
    &s.0
}
