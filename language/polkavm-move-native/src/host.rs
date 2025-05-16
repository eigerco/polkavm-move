use crate::types::{AnyValue, MoveType};

pub(crate) const PANIC_CODE: u64 = 0xca11;
pub(crate) const ALLOC_CODE: u64 = 0xdead;

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(super) fn abort(code: u64);
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(super) fn debug_print(t: *const MoveType, v: *const AnyValue);
}
