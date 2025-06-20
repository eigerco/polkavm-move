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
    pub(crate) fn hash_sha2_256(v: *const MoveByteVector) -> u32;
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn hash_sha3_256(v: *const MoveByteVector) -> u32;
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn move_to(
        type_ve: *const MoveType,
        signer_ref: *const AnyValue,
        struct_ref: *const MoveByteVector,
        tag: *const AnyValue,
    );
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn move_from(
        type_ve: *const MoveType,
        s1: *const AnyValue,
        remove: u32,
        tag: *const AnyValue,
        is_mut: u32,
    ) -> u32;
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn exists(
        type_ve: *const MoveType,
        s1: *const AnyValue,
        tag: *const AnyValue,
    ) -> u32;
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn hex_dump();
}
