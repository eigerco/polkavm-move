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
    pub fn guest_alloc(size: u64, align: u64) -> u32;
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn hash_sha2_256(t: *const MoveType, v: *const MoveByteVector) -> u32;
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn hash_sha3_256(t: *const MoveType, v: *const MoveByteVector) -> u32;
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn get_vec() -> u32;
}
