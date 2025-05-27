use crate::types::{AnyValue, MoveByteVector, MoveType};

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn abort(code: u64);
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn debug_print(t: *const MoveType, v: *const AnyValue);
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn hash_sha2_256(t: *const MoveByteVector);
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn hash_sha3_256(t: *const MoveByteVector);
}
