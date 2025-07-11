use crate::types::{AnyValue, MoveByteVector, MoveType};

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn terminate(beneficiary: *const [u8; 20]);
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn debug_print(t: *const MoveType, v: *const AnyValue);
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
    pub(crate) fn release(
        type_ve: *const MoveType,
        s1: *const AnyValue,
        struct_ref: *const MoveByteVector,
        tag: *const AnyValue,
    );
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub(crate) fn hex_dump();
}
