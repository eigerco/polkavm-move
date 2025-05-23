use crate::types::{AnyValue, MoveType};

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn abort(code: u64);
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn debug_print(t: *const MoveType, v: *const AnyValue);
}
