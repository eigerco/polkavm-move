use crate::{
    types::{AnyValue, MoveAddress, MoveSigner, MoveType, MoveUntypedVector},
    vector::{TypedMoveBorrowedRustVec, TypedMoveBorrowedRustVecMut},
};

mod allocator;
mod imports;
mod panic;

#[export_name = "move_rt_abort"]
unsafe extern "C" fn move_rt_abort(code: u64) {
    imports::abort(code);
}

#[export_name = "move_native_debug_print"]
unsafe extern "C" fn print(type_x: *const MoveType, x: *const AnyValue) {
    imports::debug_print(type_x, x);
}

#[export_name = "move_native_signer_borrow_address"]
extern "C" fn borrow_address(s: &MoveSigner) -> &MoveAddress {
    &s.0
}

#[export_name = "move_rt_vec_destroy"]
unsafe extern "C" fn vec_destroy(type_ve: &MoveType, v: MoveUntypedVector) {
    v.destroy(type_ve);
}

#[export_name = "move_rt_vec_empty"]
unsafe extern "C" fn vec_empty(type_ve: &MoveType) -> MoveUntypedVector {
    MoveUntypedVector::empty(type_ve)
}

#[export_name = "move_rt_vec_copy"]
unsafe extern "C" fn vec_copy(
    type_ve: &MoveType,
    dstv: &mut MoveUntypedVector,
    srcv: &MoveUntypedVector,
) {
    let mut dstv = TypedMoveBorrowedRustVecMut::new(type_ve, dstv);
    let srcv = TypedMoveBorrowedRustVec::new(type_ve, srcv);
    dstv.copy_from(&srcv)
}

#[export_name = "move_rt_vec_cmp_eq"]
unsafe extern "C" fn vec_cmp_eq(
    type_ve: &MoveType,
    v1: &MoveUntypedVector,
    v2: &MoveUntypedVector,
) -> bool {
    let v1 = TypedMoveBorrowedRustVec::new(type_ve, v1);
    let v2 = TypedMoveBorrowedRustVec::new(type_ve, v2);
    v1.cmp_eq(&v2)
}

#[export_name = "move_rt_str_cmp_eq"]
unsafe extern "C" fn str_cmp_eq(
    s1_ptr: *const u8,
    s1_len: u64,
    s2_ptr: *const u8,
    s2_len: u64,
) -> bool {
    let s1 = core::slice::from_raw_parts(s1_ptr, usize::try_from(s1_len).expect("usize"));
    let s1 = core::str::from_utf8_unchecked(s1); // assume utf8
    let s2 = core::slice::from_raw_parts(s2_ptr, usize::try_from(s2_len).expect("usize"));
    let s2 = core::str::from_utf8_unchecked(s2); // assume utf8
    s1 == s2
}

#[export_name = "move_rt_struct_cmp_eq"]
unsafe extern "C" fn struct_cmp_eq(type_ve: &MoveType, s1: &AnyValue, s2: &AnyValue) -> bool {
    crate::structs::cmp_eq(type_ve, s1, s2)
}
// Safety: Even empty Rust vectors have non-null buffer pointers,
// which must be correctly aligned. This function crates empty Rust vecs
// of the correct type and converts them to untyped move vecs.
#[export_name = "move_native_vector_empty"]
unsafe extern "C" fn empty(type_r: &MoveType) -> MoveUntypedVector {
    MoveUntypedVector::empty(type_r)
}

#[export_name = "move_native_vector_length"]
unsafe extern "C" fn length(type_ve: &MoveType, v: &MoveUntypedVector) -> u64 {
    TypedMoveBorrowedRustVec::new(type_ve, v).len()
}

#[export_name = "move_native_vector_borrow"]
unsafe extern "C" fn borrow<'v>(
    type_ve: &'v MoveType,
    v: &'v MoveUntypedVector,
    i: u64,
) -> &'v AnyValue {
    TypedMoveBorrowedRustVec::new(type_ve, v).borrow(i)
}

#[export_name = "move_native_vector_push_back"]
unsafe extern "C" fn push_back(type_ve: &MoveType, v: &mut MoveUntypedVector, e: *mut AnyValue) {
    TypedMoveBorrowedRustVecMut::new(type_ve, v).push_back(e)
}

#[export_name = "move_native_vector_borrow_mut"]
unsafe extern "C" fn borrow_mut<'v>(
    type_ve: &'v MoveType,
    v: &'v mut MoveUntypedVector,
    i: u64,
) -> *mut AnyValue {
    TypedMoveBorrowedRustVecMut::new(type_ve, v).borrow_mut(i)
}

#[export_name = "move_native_vector_pop_back"]
unsafe extern "C" fn pop_back(type_ve: &MoveType, v: &mut MoveUntypedVector, r: *mut AnyValue) {
    TypedMoveBorrowedRustVecMut::new(type_ve, v).pop_back(r)
}

#[export_name = "move_native_vector_destroy_empty"]
unsafe extern "C" fn destroy_empty(type_ve: &MoveType, v: MoveUntypedVector) {
    v.destroy_empty(type_ve)
}

#[export_name = "move_native_vector_swap"]
unsafe extern "C" fn swap(type_ve: &MoveType, v: &mut MoveUntypedVector, i: u64, j: u64) {
    TypedMoveBorrowedRustVecMut::new(type_ve, v).swap(i, j)
}
