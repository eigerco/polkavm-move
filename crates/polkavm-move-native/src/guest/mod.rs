use crate::{
    types::{
        AnyValue, MoveAddress, MoveAsciiString, MoveByteVector, MoveSigner, MoveType,
        MoveUntypedVector,
    },
    vector::{TypedMoveBorrowedRustVec, TypedMoveBorrowedRustVecMut},
};
extern crate alloc;
use core::str;

mod allocator;
mod imports;
mod panic;
mod polkavm_imports;

#[macro_export]
macro_rules! heapless_format {
    ($($arg:tt)*) => {{
        use heapless::String;
        use core::fmt::Write;

        let mut s: String<256> = String::new();
        let _ = write!(&mut s, $($arg)*);
        s
    }};
}

#[export_name = "move_rt_abort"]
unsafe extern "C" fn move_rt_abort(code: u64) {
    let mut beneficiary = [0u8; 20];
    beneficiary[0] = code as u8;
    imports::terminate(beneficiary.as_ptr() as *const [u8; 20]);
}

#[export_name = "move_native_debug_print"]
unsafe extern "C" fn print(type_x: *const MoveType, x: *const AnyValue) {
    imports::debug_print(type_x, x);
}

#[export_name = "move_native_debug_hex_dump"]
unsafe extern "C" fn hex_dump() {
    imports::hex_dump();
}

#[export_name = "move_native_hash_sha2_256"]
unsafe extern "C" fn move_native_hash_sha2_256(bytes: *const MoveByteVector) -> MoveByteVector {
    let address = imports::hash_sha2_256(bytes);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

#[export_name = "move_native_hash_sha3_256"]
unsafe extern "C" fn move_native_hash_sha3_256(bytes: *const MoveByteVector) -> MoveByteVector {
    let address = imports::hash_sha3_256(bytes);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

#[export_name = "move_rt_move_to"]
unsafe extern "C" fn move_to(
    type_ve: &MoveType,
    signer_ref: &AnyValue,
    struct_ref: &AnyValue,
    tag: &AnyValue,
) {
    let bytes = crate::serialization::serialize(type_ve, struct_ref);
    imports::move_to(signer_ref, &bytes, tag);
}

#[export_name = "move_rt_move_from"]
unsafe extern "C" fn move_from(
    type_ve: &MoveType,
    s1: &AnyValue,
    out: *mut AnyValue,
    tag: &AnyValue,
) {
    let address = imports::move_from(s1, 1, tag, 0);
    let bytevec = &*(address as *const MoveByteVector);
    crate::serialization::deserialize(type_ve, bytevec, out);
}

#[export_name = "move_rt_borrow_global"]
unsafe extern "C" fn borrow_global(
    type_ve: &MoveType,
    s1: &AnyValue,
    out: *mut AnyValue,
    tag: &AnyValue,
    is_mut: u32,
) {
    let address = imports::move_from(s1, 0, tag, is_mut);
    let bytevec = &*(address as *const MoveByteVector);
    // allocate a boxed slice of 2*bytevec.length bytes (due to alignment)
    let boxed: alloc::boxed::Box<[u8]> =
        alloc::vec![0u8; (bytevec.length * 2) as usize].into_boxed_slice();
    let raw = alloc::boxed::Box::into_raw(boxed);
    // Deserialize into the boxed location
    crate::serialization::deserialize(type_ve, bytevec, raw as *mut AnyValue);
    let raw_addr_value = raw as *const u8 as u32;
    // Copy the address of the boxed value into the output pointer
    core::ptr::copy_nonoverlapping(&raw_addr_value as *const u32, out as *mut u32, 1);
}

#[export_name = "move_rt_exists"]
unsafe extern "C" fn exists(_type_ve: &MoveType, s: &AnyValue, tag: &AnyValue) -> u32 {
    imports::exists(s, tag)
}

#[export_name = "move_rt_release"]
unsafe extern "C" fn release(
    type_ve: &MoveType,
    s: &AnyValue,
    struct_ref: &AnyValue,
    tag: &AnyValue,
) {
    let bytes = crate::serialization::serialize(type_ve, struct_ref);
    imports::release(s, &bytes, tag);
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

#[export_name = "move_native_string_internal_check_utf8"]
pub unsafe extern "C" fn internal_check_utf8(v: &MoveByteVector) -> bool {
    let rust_vec = v.as_rust_vec();
    let res = str::from_utf8(&rust_vec);

    res.is_ok()
}

#[export_name = "move_native_string_internal_is_char_boundary"]
pub unsafe extern "C" fn internal_is_char_boundary(v: &MoveByteVector, i: u64) -> bool {
    let rust_vec = v.as_rust_vec();
    let i = usize::try_from(i).expect("usize");

    let rust_str = str::from_utf8(&rust_vec).expect("invalid utf8");
    rust_str.is_char_boundary(i)
}

#[export_name = "move_native_string_internal_sub_string"]
pub unsafe extern "C" fn internal_sub_string(
    s: &MoveAsciiString,
    i: u64,
    j: u64,
) -> MoveByteVector {
    let v = &s.bytes;
    let rust_vec = v.as_rust_vec();
    let i = usize::try_from(i).expect("usize");
    let j = usize::try_from(j).expect("usize");

    let rust_str = str::from_utf8(&rust_vec).expect("invalid utf8");
    let sub_rust_vec = &rust_str.as_bytes()[i..j];
    MoveByteVector::from_rust_vec(sub_rust_vec.into())
}

#[export_name = "move_native_string_internal_index_of"]
pub unsafe extern "C" fn internal_index_of(s: &MoveByteVector, r: &MoveByteVector) -> u64 {
    let s_rust_vec = s.as_rust_vec();
    let s_rust_str = str::from_utf8(&s_rust_vec).expect("invalid utf8");
    let r_rust_vec = r.as_rust_vec();
    let r_rust_str = str::from_utf8(&r_rust_vec).expect("invalid utf8");

    let res = s_rust_str.find(r_rust_str);

    u64::try_from(match res {
        Some(i) => i,
        None => s_rust_str.len(),
    })
    .expect("u64")
}

#[export_name = "move_native_bcs_to_bytes"]
pub unsafe extern "C" fn to_bytes(type_v: &MoveType, v: &AnyValue) -> MoveByteVector {
    crate::serialization::serialize(type_v, v)
}

#[allow(dead_code)]
unsafe fn print_vec(vec: &MoveByteVector) {
    let typ_string = MoveType::vec();
    imports::debug_print(&typ_string, vec as *const MoveByteVector as *const AnyValue);
}

#[allow(dead_code)]
pub unsafe fn print_str(info: &str) {
    let typ_string = MoveType::vec();
    let s = MoveAsciiString {
        bytes: MoveByteVector::from_rust_vec(info.as_bytes().to_vec()),
    };
    imports::debug_print(&typ_string, &s as *const MoveAsciiString as *const AnyValue);
}
