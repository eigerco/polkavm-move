use crate::{
    types::{
        AnyValue, MoveAddress, MoveAsciiString, MoveByteVector, MoveSigner, MoveType,
        MoveUntypedReference, MoveUntypedVector, TypeDesc, ACCOUNT_ADDRESS_LENGTH, U256,
    },
    vector::{
        MoveBorrowedRustVecMut, MoveBorrowedRustVecOfStructMut, TypedMoveBorrowedRustVec,
        TypedMoveBorrowedRustVecMut,
    },
};
extern crate alloc;
use core::{ptr, str};

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
    // Write full u64 abort code (little-endian) into first 8 bytes
    let bytes = code.to_le_bytes();
    beneficiary[..8].copy_from_slice(&bytes);
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

#[export_name = "move_native_sha2_512_internal"]
unsafe extern "C" fn move_native_sha2_512_internal(bytes: *const MoveByteVector) -> MoveByteVector {
    let address = imports::hash_sha3_256(bytes);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

#[export_name = "move_native_sha3_512_internal"]
unsafe extern "C" fn move_native_sha3_512_internal(bytes: *const MoveByteVector) -> MoveByteVector {
    let address = imports::hash_sha3_256(bytes);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

#[export_name = "move_native_sip_hash"]
unsafe extern "C" fn move_native_sip_hash(bytes: *const MoveByteVector) -> MoveByteVector {
    let address = imports::sip_hash(bytes);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

#[export_name = "move_native_ripemd160_internal"]
unsafe extern "C" fn move_native_ripemd160_internal(
    bytes: *const MoveByteVector,
) -> MoveByteVector {
    let address = imports::ripemd160_internal(bytes);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

#[export_name = "move_native_blake2b_256_internal"]
unsafe extern "C" fn move_native_blake2b_256_internal(
    bytes: *const MoveByteVector,
) -> MoveByteVector {
    let address = imports::blake2b_256_internal(bytes);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

#[export_name = "move_native_keccak256"]
unsafe extern "C" fn move_native_keccak256(bytes: *const MoveByteVector) -> MoveByteVector {
    let address = imports::keccak256(bytes);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

// Aliases for aptos_std::aptos_hash native functions.
// The compiler generates names prefixed with the module path.
#[export_name = "move_native_aptos_hash_sha2_512_internal"]
unsafe extern "C" fn aptos_hash_sha2_512(bytes: *const MoveByteVector) -> MoveByteVector {
    move_native_sha2_512_internal(bytes)
}

#[export_name = "move_native_aptos_hash_sha3_512_internal"]
unsafe extern "C" fn aptos_hash_sha3_512(bytes: *const MoveByteVector) -> MoveByteVector {
    move_native_sha3_512_internal(bytes)
}

#[export_name = "move_native_aptos_hash_blake2b_256_internal"]
unsafe extern "C" fn aptos_hash_blake2b_256(bytes: *const MoveByteVector) -> MoveByteVector {
    move_native_blake2b_256_internal(bytes)
}

#[export_name = "move_native_aptos_hash_ripemd160_internal"]
unsafe extern "C" fn aptos_hash_ripemd160(bytes: *const MoveByteVector) -> MoveByteVector {
    move_native_ripemd160_internal(bytes)
}

#[export_name = "move_native_aptos_hash_keccak256"]
unsafe extern "C" fn aptos_hash_keccak256(bytes: *const MoveByteVector) -> MoveByteVector {
    move_native_keccak256(bytes)
}

#[export_name = "move_native_aptos_hash_sip_hash"]
unsafe extern "C" fn aptos_hash_sip_hash(bytes: *const MoveByteVector) -> MoveByteVector {
    move_native_sip_hash(bytes)
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

#[export_name = "move_native_vector_move_range"]
unsafe extern "C" fn vector_move_range(
    type_ve: &MoveType,
    from: &mut MoveUntypedVector,
    removal_position: u64,
    length: u64,
    to: &mut MoveUntypedVector,
    insert_position: u64,
) {
    if length == 0 {
        return;
    }

    macro_rules! typed_move_range {
        ($t:ty) => {{
            let rp = removal_position as usize;
            let count = length as usize;
            let ip = insert_position as usize;
            let mut from_vec = MoveBorrowedRustVecMut::<$t>::new(from);
            let drained: alloc::vec::Vec<$t> = from_vec.drain(rp..rp + count).collect();
            drop(from_vec);
            let mut to_vec = MoveBorrowedRustVecMut::<$t>::new(to);
            to_vec.splice(ip..ip, drained);
            drop(to_vec);
        }};
    }

    match type_ve.type_desc {
        TypeDesc::Bool => typed_move_range!(bool),
        TypeDesc::U8 => typed_move_range!(u8),
        TypeDesc::U16 => typed_move_range!(u16),
        TypeDesc::U32 => typed_move_range!(u32),
        TypeDesc::U64 => typed_move_range!(u64),
        TypeDesc::U128 => typed_move_range!(u128),
        TypeDesc::U256 => typed_move_range!(U256),
        TypeDesc::Address => typed_move_range!(MoveAddress),
        TypeDesc::Signer => typed_move_range!(MoveSigner),
        TypeDesc::Vector => typed_move_range!(MoveUntypedVector),
        TypeDesc::Reference => typed_move_range!(MoveUntypedReference),
        TypeDesc::Struct => {
            struct_move_range(type_ve, from, removal_position, length, to, insert_position);
        }
    }
}

unsafe fn struct_move_range(
    type_ve: &MoveType,
    from: &mut MoveUntypedVector,
    removal_position: u64,
    count_u64: u64,
    to: &mut MoveUntypedVector,
    insert_position: u64,
) {
    let elem_size = (*type_ve.type_info).struct_.size as usize;
    let rp = removal_position as usize;
    let count = count_u64 as usize;
    let ip = insert_position as usize;
    let from_len = from.length as usize;
    let to_len = to.length as usize;
    let byte_count = count * elem_size;

    assert!(rp + count <= from_len, "removal range out of bounds");
    assert!(ip <= to_len, "insert position out of bounds");

    // 1. Copy elements from `from` into temp buffer
    let mut temp = alloc::vec![0u8; byte_count];
    ptr::copy_nonoverlapping(from.ptr.add(rp * elem_size), temp.as_mut_ptr(), byte_count);

    // 2. Close gap in `from`
    let remaining = from_len - (rp + count);
    if remaining > 0 {
        ptr::copy(
            from.ptr.add((rp + count) * elem_size),
            from.ptr.add(rp * elem_size),
            remaining * elem_size,
        );
    }
    from.length -= count_u64;

    // 3. Ensure `to` has capacity
    let new_to_len = to_len + count;
    if new_to_len > to.capacity as usize {
        let new_cap = new_to_len.next_power_of_two();
        let mut to_struct = MoveBorrowedRustVecOfStructMut::new(type_ve, to);
        to_struct.reserve_exact(new_cap);
    }

    // 4. Shift elements in `to` to make space at insert_position
    let remaining_after = to_len - ip;
    if remaining_after > 0 {
        ptr::copy(
            to.ptr.add(ip * elem_size),
            to.ptr.add((ip + count) * elem_size),
            remaining_after * elem_size,
        );
    }

    // 5. Copy temp buffer into `to`
    ptr::copy_nonoverlapping(temp.as_ptr(), to.ptr.add(ip * elem_size), byte_count);
    to.length += count_u64;
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

#[export_name = "move_native_bcs_serialized_size"]
pub unsafe extern "C" fn bcs_serialized_size(type_v: &MoveType, v: &AnyValue) -> u64 {
    let bytes = crate::serialization::serialize(type_v, v);
    bytes.length
}

/// Return type for constant_serialized_size matching Move's Option<u64> enum layout:
/// { i64 discriminant (0=None, 1=Some), u64 value }
#[repr(C)]
pub struct MoveOptionU64 {
    discriminant: i64,
    value: u64,
}

/// Compute the constant BCS serialized size of a type (no value needed).
/// Returns Option<u64>: Some(size) if constant, None if variable.
unsafe fn constant_size_for_type(type_v: &MoveType) -> Option<usize> {
    match type_v.type_desc {
        TypeDesc::Bool | TypeDesc::U8 => Some(1),
        TypeDesc::U16 => Some(2),
        TypeDesc::U32 => Some(4),
        TypeDesc::U64 => Some(8),
        TypeDesc::U128 => Some(16),
        TypeDesc::U256 => Some(32),
        TypeDesc::Address => Some(32),
        TypeDesc::Signer => None,
        TypeDesc::Vector => None,
        TypeDesc::Reference => None,
        TypeDesc::Struct => {
            let structinfo = &(*(type_v.type_info)).struct_;
            let field_count = structinfo.field_array_len as usize;
            if field_count == 0 {
                return Some(0);
            }
            let fields = core::slice::from_raw_parts(structinfo.field_array_ptr, field_count);
            // Detect enums: first field offset > 0 means there's a discriminant tag
            if fields[0].offset > 0 {
                return None;
            }
            let mut total = 0;
            for field in fields {
                match constant_size_for_type(&field.type_) {
                    Some(sz) => total += sz,
                    None => return None,
                }
            }
            Some(total)
        }
    }
}

#[export_name = "move_native_bcs_constant_serialized_size"]
pub unsafe extern "C" fn bcs_constant_serialized_size(type_v: &MoveType) -> MoveOptionU64 {
    match constant_size_for_type(type_v) {
        Some(size) => MoveOptionU64 {
            discriminant: 1,
            value: size as u64,
        },
        None => MoveOptionU64 {
            discriminant: 0,
            value: 0,
        },
    }
}

// --- ed25519 native functions ---

#[export_name = "move_native_ed25519_public_key_validate_internal"]
unsafe extern "C" fn ed25519_public_key_validate_internal(bytes: *const MoveByteVector) -> bool {
    imports::ed25519_public_key_validate(bytes) != 0
}

#[export_name = "move_native_ed25519_signature_verify_strict_internal"]
unsafe extern "C" fn ed25519_signature_verify_strict_internal(
    sig: *const MoveByteVector,
    pk: *const MoveByteVector,
    msg: *const MoveByteVector,
) -> bool {
    imports::ed25519_signature_verify_strict(sig, pk, msg) != 0
}

#[export_name = "move_native_ed25519_generate_keys_internal"]
unsafe extern "C" fn ed25519_generate_keys_internal(out_sk: *mut MoveByteVector) -> MoveByteVector {
    let address = imports::ed25519_generate_keys();
    // Write first vector (sk) to caller's alloca via pointer
    let sk_src = address as *const MoveByteVector;
    ptr::write(out_sk, ptr::read(sk_src));
    // Return second vector (pk) in register (via sret, like sip_hash)
    let pk_src = (address as *const u8).add(24) as *const MoveByteVector;
    ptr::read(pk_src)
}

#[export_name = "move_native_ed25519_sign_internal"]
unsafe extern "C" fn ed25519_sign_internal(
    sk: *const MoveByteVector,
    msg: *const MoveByteVector,
) -> MoveByteVector {
    let address = imports::ed25519_sign(sk, msg);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

// --- multi_ed25519 native functions ---

#[export_name = "move_native_multi_ed25519_public_key_validate_internal"]
unsafe extern "C" fn multi_ed25519_public_key_validate_internal(
    bytes: *const MoveByteVector,
) -> bool {
    imports::multi_ed25519_public_key_validate(bytes) != 0
}

#[export_name = "move_native_multi_ed25519_public_key_validate_v2_internal"]
unsafe extern "C" fn multi_ed25519_public_key_validate_v2_internal(
    bytes: *const MoveByteVector,
) -> bool {
    imports::multi_ed25519_public_key_validate_v2(bytes) != 0
}

#[export_name = "move_native_multi_ed25519_signature_verify_strict_internal"]
unsafe extern "C" fn multi_ed25519_signature_verify_strict_internal(
    sig: *const MoveByteVector,
    pk: *const MoveByteVector,
    msg: *const MoveByteVector,
) -> bool {
    imports::multi_ed25519_signature_verify_strict(sig, pk, msg) != 0
}

#[export_name = "move_native_multi_ed25519_sign_internal"]
unsafe extern "C" fn multi_ed25519_sign_internal(
    sk: *const MoveByteVector,
    msg: *const MoveByteVector,
) -> MoveByteVector {
    let address = imports::multi_ed25519_sign(sk, msg);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

// --- bls12381 native functions ---

#[export_name = "move_native_bls12381_validate_pubkey_internal"]
unsafe extern "C" fn bls12381_validate_pubkey_internal(bytes: *const MoveByteVector) -> bool {
    imports::bls12381_validate_pubkey(bytes) != 0
}

#[export_name = "move_native_bls12381_signature_subgroup_check_internal"]
unsafe extern "C" fn bls12381_signature_subgroup_check_internal(
    bytes: *const MoveByteVector,
) -> bool {
    imports::bls12381_signature_subgroup_check(bytes) != 0
}

#[export_name = "move_native_bls12381_verify_normal_signature_internal"]
unsafe extern "C" fn bls12381_verify_normal_signature_internal(
    sig: *const MoveByteVector,
    pk: *const MoveByteVector,
    msg: *const MoveByteVector,
) -> bool {
    imports::bls12381_verify_normal_signature(sig, pk, msg) != 0
}

#[export_name = "move_native_bls12381_verify_multisignature_internal"]
unsafe extern "C" fn bls12381_verify_multisignature_internal(
    sig: *const MoveByteVector,
    pk: *const MoveByteVector,
    msg: *const MoveByteVector,
) -> bool {
    imports::bls12381_verify_multisignature(sig, pk, msg) != 0
}

#[export_name = "move_native_bls12381_verify_proof_of_possession_internal"]
unsafe extern "C" fn bls12381_verify_proof_of_possession_internal(
    pk: *const MoveByteVector,
    pop: *const MoveByteVector,
) -> bool {
    imports::bls12381_verify_proof_of_possession(pk, pop) != 0
}

#[export_name = "move_native_bls12381_verify_signature_share_internal"]
unsafe extern "C" fn bls12381_verify_signature_share_internal(
    sig: *const MoveByteVector,
    pk: *const MoveByteVector,
    msg: *const MoveByteVector,
) -> bool {
    imports::bls12381_verify_signature_share(sig, pk, msg) != 0
}

#[export_name = "move_native_bls12381_sign_internal"]
unsafe extern "C" fn bls12381_sign_internal(
    sk: *const MoveByteVector,
    msg: *const MoveByteVector,
) -> MoveByteVector {
    let address = imports::bls12381_sign(sk, msg);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

#[export_name = "move_native_bls12381_generate_proof_of_possession_internal"]
unsafe extern "C" fn bls12381_generate_proof_of_possession_internal(
    sk: *const MoveByteVector,
) -> MoveByteVector {
    let address = imports::bls12381_generate_proof_of_possession(sk);
    let mv_ptr = address as *const MoveByteVector;
    *mv_ptr
}

// --- bls12381 aggregate native functions ---

#[export_name = "move_native_bls12381_aggregate_pubkeys_internal"]
unsafe extern "C" fn bls12381_aggregate_pubkeys_internal(
    pks: *const MoveByteVector,
    out_vec: *mut MoveByteVector,
) -> bool {
    let result_addr = imports::bls12381_aggregate_pubkeys(pks);
    let result_ptr = result_addr as *const u8;
    let vec = *(result_ptr as *const MoveByteVector);
    ptr::write(out_vec, vec);
    let success = *result_ptr.add(24);
    success != 0
}

#[export_name = "move_native_bls12381_aggregate_signatures_internal"]
unsafe extern "C" fn bls12381_aggregate_signatures_internal(
    sigs: *const MoveByteVector,
    out_vec: *mut MoveByteVector,
) -> bool {
    let result_addr = imports::bls12381_aggregate_signatures(sigs);
    let result_ptr = result_addr as *const u8;
    let vec = *(result_ptr as *const MoveByteVector);
    ptr::write(out_vec, vec);
    let success = *result_ptr.add(24);
    success != 0
}

#[export_name = "move_native_bls12381_verify_aggregate_signature_internal"]
unsafe extern "C" fn bls12381_verify_aggregate_signature_internal(
    sig: *const MoveByteVector,
    pks: *const MoveByteVector,
    msgs: *const MoveByteVector,
) -> bool {
    imports::bls12381_verify_aggregate_signature(sig, pks, msgs) != 0
}

#[export_name = "move_native_bls12381_generate_keys_internal"]
unsafe extern "C" fn bls12381_generate_keys_internal(
    out_sk: *mut MoveByteVector,
) -> MoveByteVector {
    let address = imports::bls12381_generate_keys();
    // Write first vector (sk) to caller's alloca via pointer
    let sk_src = address as *const MoveByteVector;
    ptr::write(out_sk, ptr::read(sk_src));
    // Return second vector (pk_with_pop) in register
    let pk_src = (address as *const u8).add(24) as *const MoveByteVector;
    ptr::read(pk_src)
}

// --- cmp native functions ---

#[export_name = "move_native_cmp_compare"]
pub unsafe extern "C" fn cmp_compare(
    type_t: &MoveType,
    first: &AnyValue,
    second: &AnyValue,
) -> i64 {
    // Move Ordering: Less=0, Equal=1, Greater=2
    // Rust Ordering: Less=-1, Equal=0, Greater=1
    let ord = crate::comparison::compare(type_t, first, second);
    ord as i64 + 1
}

// --- mem native functions ---

/// Returns the size in bytes of a Move type.
unsafe fn size_of_move_type(type_ve: &MoveType) -> usize {
    match type_ve.type_desc {
        TypeDesc::Bool | TypeDesc::U8 => 1,
        TypeDesc::U16 => 2,
        TypeDesc::U32 => 4,
        TypeDesc::U64 => 8,
        TypeDesc::U128 => 16,
        TypeDesc::U256 => core::mem::size_of::<U256>(),
        TypeDesc::Address => core::mem::size_of::<MoveAddress>(),
        TypeDesc::Signer => core::mem::size_of::<MoveSigner>(),
        TypeDesc::Vector => core::mem::size_of::<MoveUntypedVector>(),
        TypeDesc::Reference => core::mem::size_of::<MoveUntypedReference>(),
        TypeDesc::Struct => (*type_ve.type_info).struct_.size as usize,
    }
}

#[export_name = "move_native_mem_swap"]
unsafe extern "C" fn mem_swap(type_ve: &MoveType, left: *mut AnyValue, right: *mut AnyValue) {
    let size = size_of_move_type(type_ve);
    let left = left as *mut u8;
    let right = right as *mut u8;
    // Use a stack buffer for small types, heap for large
    if size <= 256 {
        let mut tmp = [0u8; 256];
        ptr::copy_nonoverlapping(left, tmp.as_mut_ptr(), size);
        ptr::copy_nonoverlapping(right, left, size);
        ptr::copy_nonoverlapping(tmp.as_ptr(), right, size);
    } else {
        let mut tmp = alloc::vec![0u8; size];
        ptr::copy_nonoverlapping(left, tmp.as_mut_ptr(), size);
        ptr::copy_nonoverlapping(right, left, size);
        ptr::copy_nonoverlapping(tmp.as_ptr(), right, size);
    }
}

// --- from_bcs native functions ---

#[export_name = "move_native_from_bcs_from_bytes"]
pub unsafe extern "C" fn from_bcs_from_bytes(
    type_t: &MoveType,
    bytes: &MoveByteVector,
    out: *mut AnyValue,
) {
    crate::serialization::deserialize(type_t, bytes, out);
}

// --- type_info native functions ---

#[repr(C)]
pub struct MoveTypeInfoReturn {
    account_address: MoveAddress,
    module_name: MoveByteVector,
    struct_name: MoveByteVector,
}

/// Parse a hex address string (e.g. "0x1") into a 32-byte little-endian MoveAddress.
unsafe fn parse_hex_address(addr_str: &[u8]) -> MoveAddress {
    // Strip "0x" prefix if present
    let hex_str = if addr_str.len() >= 2 && addr_str[0] == b'0' && addr_str[1] == b'x' {
        &addr_str[2..]
    } else {
        addr_str
    };

    let mut bytes = [0u8; ACCOUNT_ADDRESS_LENGTH];

    // Parse hex string into big-endian bytes, right-aligned
    let hex_len = hex_str.len();
    let byte_count = (hex_len + 1) / 2;
    let start = ACCOUNT_ADDRESS_LENGTH - byte_count;

    for i in 0..hex_len {
        let nibble = match hex_str[i] {
            b'0'..=b'9' => hex_str[i] - b'0',
            b'a'..=b'f' => hex_str[i] - b'a' + 10,
            b'A'..=b'F' => hex_str[i] - b'A' + 10,
            _ => 0,
        };
        let byte_idx = start + (i / 2);
        // If hex_len is odd, the first nibble is the low nibble of the first byte
        if hex_len % 2 == 1 && i == 0 {
            bytes[byte_idx] |= nibble;
        } else if (hex_len % 2 == 1 && i % 2 == 1) || (hex_len % 2 == 0 && i % 2 == 0) {
            bytes[byte_idx] |= nibble << 4;
        } else {
            bytes[byte_idx] |= nibble;
        }
    }

    // Reverse to little-endian (MoveAddress stores LE)
    bytes.reverse();
    MoveAddress(bytes)
}

#[export_name = "move_native_type_info_type_name"]
pub unsafe extern "C" fn type_info_type_name(type_t: &MoveType) -> MoveAsciiString {
    let name_bytes = core::slice::from_raw_parts(type_t.name.ptr, type_t.name.len as usize);
    MoveAsciiString {
        bytes: MoveByteVector::from_rust_vec(name_bytes.to_vec()),
    }
}

#[export_name = "move_native_type_info_type_of"]
pub unsafe extern "C" fn type_info_type_of(type_t: &MoveType) -> MoveTypeInfoReturn {
    let name_bytes = core::slice::from_raw_parts(type_t.name.ptr, type_t.name.len as usize);

    // Find first "::" — separates address from module
    let mut first_sep = None;
    for i in 0..name_bytes.len().saturating_sub(1) {
        if name_bytes[i] == b':' && name_bytes[i + 1] == b':' {
            first_sep = Some(i);
            break;
        }
    }

    if let Some(fs) = first_sep {
        let addr_str = &name_bytes[..fs];
        let after_addr = &name_bytes[fs + 2..];

        // Find second "::" — separates module from struct name
        let mut second_sep = None;
        for i in 0..after_addr.len().saturating_sub(1) {
            if after_addr[i] == b':' && after_addr[i + 1] == b':' {
                second_sep = Some(i);
                break;
            }
        }

        if let Some(ss) = second_sep {
            let module_str = &after_addr[..ss];
            let struct_str = &after_addr[ss + 2..];

            return MoveTypeInfoReturn {
                account_address: parse_hex_address(addr_str),
                module_name: MoveByteVector::from_rust_vec(module_str.to_vec()),
                struct_name: MoveByteVector::from_rust_vec(struct_str.to_vec()),
            };
        }
    }

    // For primitives or types without "::", return zeroed address and empty strings
    MoveTypeInfoReturn {
        account_address: MoveAddress([0u8; ACCOUNT_ADDRESS_LENGTH]),
        module_name: MoveByteVector::from_rust_vec(alloc::vec::Vec::new()),
        struct_name: MoveByteVector::from_rust_vec(name_bytes.to_vec()),
    }
}

// --- unit_test native functions ---

#[export_name = "move_native_unit_test_create_signers_for_testing"]
unsafe extern "C" fn unit_test_create_signers_for_testing(num_signers: u64) -> MoveUntypedVector {
    let signers: alloc::vec::Vec<MoveSigner> = (0..num_signers)
        .map(|i| {
            let mut addr = [0u8; ACCOUNT_ADDRESS_LENGTH];
            addr[..8].copy_from_slice(&i.to_le_bytes());
            MoveSigner(MoveAddress(addr))
        })
        .collect();
    MoveUntypedVector::from_rust_vec(signers)
}

// --- table native functions ---

struct TableEntry {
    key_bytes: alloc::vec::Vec<u8>,
    value_ptr: *mut u8,
    value_size: usize,
}

static mut TABLE_STORE: Option<alloc::vec::Vec<(u32, alloc::vec::Vec<TableEntry>)>> = None;
static mut NEXT_TABLE_HANDLE: u32 = 1;

const TABLE_VALUE_ALIGN: usize = 8;

#[inline]
unsafe fn table_store() -> &'static mut alloc::vec::Vec<(u32, alloc::vec::Vec<TableEntry>)> {
    let ptr = core::ptr::addr_of_mut!(TABLE_STORE);
    (*ptr).get_or_insert_with(alloc::vec::Vec::new)
}

unsafe fn extract_handle_id(table_ptr: *const AnyValue) -> u32 {
    ptr::read_unaligned(table_ptr as *const u32)
}

unsafe fn serialize_key_to_vec(type_k: &MoveType, key: *const AnyValue) -> alloc::vec::Vec<u8> {
    let mbv = crate::serialization::serialize(type_k, &*key);
    mbv.into_rust_vec()
}

#[export_name = "move_native_table_new_table_handle"]
unsafe extern "C" fn table_new_table_handle(_type_k: &MoveType, _type_v: &MoveType) -> MoveAddress {
    let handle_ptr = core::ptr::addr_of_mut!(NEXT_TABLE_HANDLE);
    let handle_id = *handle_ptr;
    *handle_ptr += 1;
    table_store().push((handle_id, alloc::vec::Vec::new()));
    let mut addr = [0u8; ACCOUNT_ADDRESS_LENGTH];
    addr[..4].copy_from_slice(&handle_id.to_le_bytes());
    MoveAddress(addr)
}

#[export_name = "move_native_table_add_box"]
unsafe extern "C" fn table_add_box(
    type_k: &MoveType,
    _type_v: &MoveType,
    type_b: &MoveType,
    table: *mut AnyValue,
    key: *const AnyValue,
    val: *const AnyValue,
) {
    let handle_id = extract_handle_id(table);
    let key_bytes = serialize_key_to_vec(type_k, key);

    let (_, entries) = match table_store().iter_mut().find(|(id, _)| *id == handle_id) {
        Some(x) => x,
        None => {
            move_rt_abort(200); // add_box: table not found
            return;
        }
    };

    if entries.iter().any(|e| e.key_bytes == key_bytes) {
        move_rt_abort(100); // EALREADY_EXISTS
        return;
    }

    let value_size = size_of_move_type(type_b);
    let layout = alloc::alloc::Layout::from_size_align(value_size, TABLE_VALUE_ALIGN)
        .unwrap_or_else(|_| {
            move_rt_abort(202);
            core::hint::unreachable_unchecked()
        });
    let value_ptr = alloc::alloc::alloc(layout);
    ptr::copy_nonoverlapping(val as *const u8, value_ptr, value_size);

    entries.push(TableEntry {
        key_bytes,
        value_ptr,
        value_size,
    });
}

#[export_name = "move_native_table_borrow_box"]
unsafe extern "C" fn table_borrow_box(
    type_k: &MoveType,
    _type_v: &MoveType,
    _type_b: &MoveType,
    table: *const AnyValue,
    key: *const AnyValue,
) -> *const AnyValue {
    let handle_id = extract_handle_id(table);
    let key_bytes = serialize_key_to_vec(type_k, key);

    let (_, entries) = table_store()
        .iter()
        .find(|(id, _)| *id == handle_id)
        .unwrap_or_else(|| {
            move_rt_abort(201);
            core::hint::unreachable_unchecked()
        });

    for entry in entries.iter() {
        if entry.key_bytes == key_bytes {
            return entry.value_ptr as *const AnyValue;
        }
    }

    move_rt_abort(101); // ENOT_FOUND
    core::hint::unreachable_unchecked()
}

#[export_name = "move_native_table_borrow_box_mut"]
unsafe extern "C" fn table_borrow_box_mut(
    type_k: &MoveType,
    _type_v: &MoveType,
    _type_b: &MoveType,
    table: *mut AnyValue,
    key: *const AnyValue,
) -> *mut AnyValue {
    let handle_id = extract_handle_id(table);
    let key_bytes = serialize_key_to_vec(type_k, key);

    let (_, entries) = table_store()
        .iter()
        .find(|(id, _)| *id == handle_id)
        .unwrap_or_else(|| {
            move_rt_abort(201);
            core::hint::unreachable_unchecked()
        });

    for entry in entries.iter() {
        if entry.key_bytes == key_bytes {
            return entry.value_ptr as *mut AnyValue;
        }
    }

    move_rt_abort(101); // ENOT_FOUND
    core::hint::unreachable_unchecked()
}

#[export_name = "move_native_table_contains_box"]
unsafe extern "C" fn table_contains_box(
    type_k: &MoveType,
    _type_v: &MoveType,
    _type_b: &MoveType,
    table: *const AnyValue,
    key: *const AnyValue,
) -> bool {
    let handle_id = extract_handle_id(table);
    let key_bytes = serialize_key_to_vec(type_k, key);

    let (_, entries) = table_store()
        .iter()
        .find(|(id, _)| *id == handle_id)
        .unwrap_or_else(|| {
            move_rt_abort(201);
            core::hint::unreachable_unchecked()
        });

    entries.iter().any(|e| e.key_bytes == key_bytes)
}

#[export_name = "move_native_table_remove_box"]
unsafe extern "C" fn table_remove_box(
    type_k: &MoveType,
    _type_v: &MoveType,
    _type_b: &MoveType,
    table: *mut AnyValue,
    key: *const AnyValue,
    out: *mut AnyValue,
) {
    let handle_id = extract_handle_id(table);
    let key_bytes = serialize_key_to_vec(type_k, key);

    let (_, entries) = table_store()
        .iter_mut()
        .find(|(id, _)| *id == handle_id)
        .unwrap_or_else(|| {
            move_rt_abort(201);
            core::hint::unreachable_unchecked()
        });

    let pos = entries.iter().position(|e| e.key_bytes == key_bytes);
    match pos {
        Some(i) => {
            let entry = entries.remove(i);
            ptr::copy_nonoverlapping(entry.value_ptr, out as *mut u8, entry.value_size);
            let layout = alloc::alloc::Layout::from_size_align(entry.value_size, TABLE_VALUE_ALIGN)
                .unwrap_or_else(|_| {
                    move_rt_abort(202);
                    core::hint::unreachable_unchecked()
                });
            alloc::alloc::dealloc(entry.value_ptr, layout);
        }
        None => {
            move_rt_abort(101); // ENOT_FOUND
        }
    }
}

#[export_name = "move_native_table_destroy_empty_box"]
unsafe extern "C" fn table_destroy_empty_box(
    _type_k: &MoveType,
    _type_v: &MoveType,
    _type_b: &MoveType,
    table: *const AnyValue,
) {
    let handle_id = extract_handle_id(table);

    let store = table_store();
    if let Some((_, entries)) = store.iter().find(|(id, _)| *id == handle_id) {
        if !entries.is_empty() {
            move_rt_abort(102); // ENOT_EMPTY
        }
    }
}

#[export_name = "move_native_table_drop_unchecked_box"]
unsafe extern "C" fn table_drop_unchecked_box(
    _type_k: &MoveType,
    _type_v: &MoveType,
    _type_b: &MoveType,
    table: *const AnyValue,
) {
    let handle_id = extract_handle_id(table);
    let store = table_store();
    if let Some(pos) = store.iter().position(|(id, _)| *id == handle_id) {
        let (_, entries) = store.remove(pos);
        for entry in entries {
            if entry.value_size > 0 {
                let layout =
                    alloc::alloc::Layout::from_size_align(entry.value_size, TABLE_VALUE_ALIGN)
                        .unwrap_or_else(|_| {
                            move_rt_abort(202);
                            core::hint::unreachable_unchecked()
                        });
                alloc::alloc::dealloc(entry.value_ptr, layout);
            }
        }
    }
}

// --- debug native functions ---

#[export_name = "move_native_debug_test_debug_return_true"]
unsafe extern "C" fn debug_return_true() -> bool {
    true
}

#[export_name = "move_native_debug_test_debug_return_tuple"]
unsafe extern "C" fn debug_return_tuple(out_val: *mut u64) -> bool {
    ptr::write(out_val, 42);
    true
}

#[export_name = "move_native_debug_test_debug_return_vec_bool"]
unsafe extern "C" fn debug_return_vec_bool(out_vec: *mut MoveByteVector) -> bool {
    ptr::write(
        out_vec,
        MoveByteVector {
            ptr: ptr::null_mut(),
            capacity: 0,
            length: 0,
        },
    );
    true
}

#[export_name = "move_native_debug_test_debug_vec_args_tuple"]
unsafe extern "C" fn debug_vec_args_tuple(
    _msg: *const MoveByteVector,
    _id: u8,
    _sig: *const MoveByteVector,
    out_vec: *mut MoveByteVector,
) -> bool {
    ptr::write(
        out_vec,
        MoveByteVector {
            ptr: ptr::null_mut(),
            capacity: 0,
            length: 0,
        },
    );
    true
}

// --- secp256k1 native functions ---

#[export_name = "move_native_secp256k1_ecdsa_recover_internal"]
unsafe extern "C" fn secp256k1_ecdsa_recover_internal(
    msg: *const MoveByteVector,
    recovery_id: u8,
    sig: *const MoveByteVector,
    out_pk: *mut MoveByteVector,
) -> bool {
    let result_addr = imports::secp256k1_ecdsa_recover(msg, recovery_id as u32, sig);
    // Result struct on heap: { pk: MoveByteVector (24 bytes), success: u8 }
    let result_ptr = result_addr as *const u8;
    let pk_vec = *(result_ptr as *const MoveByteVector);
    ptr::write(out_pk, pk_vec);
    let success = *result_ptr.add(24);
    success != 0
}

// --- type_info native functions ---

#[export_name = "move_native_type_info_chain_id_internal"]
unsafe extern "C" fn chain_id_internal() -> u8 {
    imports::chain_id_internal() as u8
}

// --- ristretto255 native functions ---

// Scalar operations

#[export_name = "move_native_ristretto255_scalar_is_canonical_internal"]
unsafe extern "C" fn ristretto255_scalar_is_canonical_internal(
    bytes: *const MoveByteVector,
) -> bool {
    imports::ristretto255_scalar_is_canonical_internal(bytes) != 0
}

#[export_name = "move_native_ristretto255_scalar_from_u64_internal"]
unsafe extern "C" fn ristretto255_scalar_from_u64_internal(v: u64) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_from_u64_internal(v);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_from_u128_internal"]
unsafe extern "C" fn ristretto255_scalar_from_u128_internal(lo: u64, hi: u64) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_from_u128_internal(lo, hi);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_reduced_from_32_bytes_internal"]
unsafe extern "C" fn ristretto255_scalar_reduced_from_32_bytes_internal(
    bytes: *const MoveByteVector,
) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_reduced_from_32_bytes_internal(bytes);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_uniform_from_64_bytes_internal"]
unsafe extern "C" fn ristretto255_scalar_uniform_from_64_bytes_internal(
    bytes: *const MoveByteVector,
) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_uniform_from_64_bytes_internal(bytes);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_from_sha512_internal"]
unsafe extern "C" fn ristretto255_scalar_from_sha512_internal(
    bytes: *const MoveByteVector,
) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_from_sha512_internal(bytes);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_invert_internal"]
unsafe extern "C" fn ristretto255_scalar_invert_internal(
    bytes: *const MoveByteVector,
) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_invert_internal(bytes);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_mul_internal"]
unsafe extern "C" fn ristretto255_scalar_mul_internal(
    a: *const MoveByteVector,
    b: *const MoveByteVector,
) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_mul_internal(a, b);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_add_internal"]
unsafe extern "C" fn ristretto255_scalar_add_internal(
    a: *const MoveByteVector,
    b: *const MoveByteVector,
) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_add_internal(a, b);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_sub_internal"]
unsafe extern "C" fn ristretto255_scalar_sub_internal(
    a: *const MoveByteVector,
    b: *const MoveByteVector,
) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_sub_internal(a, b);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_scalar_neg_internal"]
unsafe extern "C" fn ristretto255_scalar_neg_internal(a: *const MoveByteVector) -> MoveByteVector {
    let addr = imports::ristretto255_scalar_neg_internal(a);
    *(addr as *const MoveByteVector)
}

// Point operations

#[export_name = "move_native_ristretto255_point_identity_internal"]
unsafe extern "C" fn ristretto255_point_identity_internal() -> u64 {
    imports::ristretto255_point_identity_internal()
}

#[export_name = "move_native_ristretto255_point_is_canonical_internal"]
unsafe extern "C" fn ristretto255_point_is_canonical_internal(
    bytes: *const MoveByteVector,
) -> bool {
    imports::ristretto255_point_is_canonical_internal(bytes) != 0
}

#[export_name = "move_native_ristretto255_point_decompress_internal"]
unsafe extern "C" fn ristretto255_point_decompress_internal(
    bytes: *const MoveByteVector,
    out_handle: *mut u64,
) -> bool {
    let result_addr = imports::ristretto255_point_decompress_internal(bytes);
    // Result struct: { handle: u64, ok: u32 }
    let result_ptr = result_addr as *const u8;
    let handle = *(result_ptr as *const u64);
    let ok = *(result_ptr.add(8) as *const u32);
    ptr::write(out_handle, handle);
    ok != 0
}

#[export_name = "move_native_ristretto255_point_clone_internal"]
unsafe extern "C" fn ristretto255_point_clone_internal(handle: u64) -> u64 {
    imports::ristretto255_point_clone_internal(handle)
}

#[export_name = "move_native_ristretto255_point_compress_internal"]
unsafe extern "C" fn ristretto255_point_compress_internal(point_ref: *const u64) -> MoveByteVector {
    let handle = *point_ref;
    let addr = imports::ristretto255_point_compress_internal(handle);
    *(addr as *const MoveByteVector)
}

#[export_name = "move_native_ristretto255_point_mul_internal"]
unsafe extern "C" fn ristretto255_point_mul_internal(
    point_ref: *const u64,
    scalar: *const MoveByteVector,
    in_place: bool,
) -> u64 {
    let handle = *point_ref;
    imports::ristretto255_point_mul_internal(handle, scalar, in_place as u32)
}

#[export_name = "move_native_ristretto255_point_add_internal"]
unsafe extern "C" fn ristretto255_point_add_internal(
    a_ref: *const u64,
    b_ref: *const u64,
    in_place: bool,
) -> u64 {
    let h1 = *a_ref;
    let h2 = *b_ref;
    imports::ristretto255_point_add_internal(h1, h2, in_place as u32)
}

#[export_name = "move_native_ristretto255_point_sub_internal"]
unsafe extern "C" fn ristretto255_point_sub_internal(
    a_ref: *const u64,
    b_ref: *const u64,
    in_place: bool,
) -> u64 {
    let h1 = *a_ref;
    let h2 = *b_ref;
    imports::ristretto255_point_sub_internal(h1, h2, in_place as u32)
}

#[export_name = "move_native_ristretto255_point_neg_internal"]
unsafe extern "C" fn ristretto255_point_neg_internal(a_ref: *const u64, in_place: bool) -> u64 {
    let handle = *a_ref;
    imports::ristretto255_point_neg_internal(handle, in_place as u32)
}

#[export_name = "move_native_ristretto255_point_equals"]
unsafe extern "C" fn ristretto255_point_equals(g_ref: *const u64, h_ref: *const u64) -> bool {
    let h1 = *g_ref;
    let h2 = *h_ref;
    imports::ristretto255_point_equals(h1, h2) != 0
}

#[export_name = "move_native_ristretto255_basepoint_mul_internal"]
unsafe extern "C" fn ristretto255_basepoint_mul_internal(scalar: *const MoveByteVector) -> u64 {
    imports::ristretto255_basepoint_mul_internal(scalar)
}

#[export_name = "move_native_ristretto255_basepoint_double_mul_internal"]
unsafe extern "C" fn ristretto255_basepoint_double_mul_internal(
    a: *const MoveByteVector,
    point_ref: *const u64,
    b: *const MoveByteVector,
) -> u64 {
    let handle = *point_ref;
    imports::ristretto255_basepoint_double_mul_internal(a, handle, b)
}

#[export_name = "move_native_ristretto255_double_scalar_mul_internal"]
unsafe extern "C" fn ristretto255_double_scalar_mul_internal(
    h1: u64,
    h2: u64,
    s1: *const MoveByteVector,
    s2: *const MoveByteVector,
) -> u64 {
    imports::ristretto255_double_scalar_mul_internal(h1, h2, s1, s2)
}

#[export_name = "move_native_ristretto255_new_point_from_sha512_internal"]
unsafe extern "C" fn ristretto255_new_point_from_sha512_internal(
    bytes: *const MoveByteVector,
) -> u64 {
    imports::ristretto255_new_point_from_sha512_internal(bytes)
}

#[export_name = "move_native_ristretto255_new_point_from_64_uniform_bytes_internal"]
unsafe extern "C" fn ristretto255_new_point_from_64_uniform_bytes_internal(
    bytes: *const MoveByteVector,
) -> u64 {
    imports::ristretto255_new_point_from_64_uniform_bytes_internal(bytes)
}

#[export_name = "move_native_ristretto255_multi_scalar_mul_internal"]
unsafe extern "C" fn ristretto255_multi_scalar_mul_internal(
    _type_desc_p: *const u8, // type descriptor for P (RistrettoPoint)
    _type_desc_s: *const u8, // type descriptor for S (Scalar)
    points: *const MoveByteVector,
    scalars: *const MoveByteVector,
) -> u64 {
    imports::ristretto255_multi_scalar_mul_internal(points, scalars)
}

// Bulletproofs

#[export_name = "move_native_ristretto255_bulletproofs_verify_range_proof_internal"]
unsafe extern "C" fn ristretto255_bulletproofs_verify_range_proof_internal(
    com: *const MoveByteVector,
    val_base_ref: *const u64,
    rand_base_ref: *const u64,
    proof: *const MoveByteVector,
    num_bits: u64,
    dst: *const MoveByteVector,
) -> bool {
    let val_base_handle = *val_base_ref;
    let rand_base_handle = *rand_base_ref;
    imports::ristretto255_bulletproofs_verify_range_proof_internal(
        com,
        val_base_handle,
        rand_base_handle,
        proof,
        num_bits,
        dst,
    ) != 0
}

#[export_name = "move_native_ristretto255_bulletproofs_verify_batch_range_proof_internal"]
unsafe extern "C" fn ristretto255_bulletproofs_verify_batch_range_proof_internal(
    coms: *const MoveByteVector,
    val_base_ref: *const u64,
    rand_base_ref: *const u64,
    proof: *const MoveByteVector,
    num_bits: u64,
    dst: *const MoveByteVector,
) -> bool {
    let val_base_handle = *val_base_ref;
    let rand_base_handle = *rand_base_ref;
    imports::ristretto255_bulletproofs_verify_batch_range_proof_internal(
        coms,
        val_base_handle,
        rand_base_handle,
        proof,
        num_bits,
        dst,
    ) != 0
}

#[export_name = "move_native_ristretto255_bulletproofs_prove_range_internal"]
unsafe extern "C" fn ristretto255_bulletproofs_prove_range_internal(
    val: *const MoveByteVector,
    r: *const MoveByteVector,
    num_bits: u64,
    dst: *const MoveByteVector,
    val_base_ref: *const u64,
    rand_base_ref: *const u64,
    out_proof: *mut MoveByteVector,
) -> MoveByteVector {
    let val_base_handle = *val_base_ref;
    let rand_base_handle = *rand_base_ref;
    let result_addr = imports::ristretto255_bulletproofs_prove_range_internal(
        val,
        r,
        num_bits,
        dst,
        val_base_handle,
        rand_base_handle,
    );
    // Result struct: { proof: MoveByteVector (24 bytes), com: MoveByteVector (24 bytes) }
    let result_ptr = result_addr as *const u8;
    let proof_vec = *(result_ptr as *const MoveByteVector);
    ptr::write(out_proof, proof_vec);
    let com_vec = *(result_ptr.add(24) as *const MoveByteVector);
    com_vec
}

#[export_name = "move_native_ristretto255_bulletproofs_prove_batch_range_internal"]
unsafe extern "C" fn ristretto255_bulletproofs_prove_batch_range_internal(
    vals: *const MoveByteVector,
    rs: *const MoveByteVector,
    num_bits: u64,
    dst: *const MoveByteVector,
    val_base_ref: *const u64,
    rand_base_ref: *const u64,
    out_proof: *mut MoveByteVector,
) -> MoveByteVector {
    let val_base_handle = *val_base_ref;
    let rand_base_handle = *rand_base_ref;
    let result_addr = imports::ristretto255_bulletproofs_prove_batch_range_internal(
        vals,
        rs,
        num_bits,
        dst,
        val_base_handle,
        rand_base_handle,
    );
    // Result struct: { proof: MoveByteVector (24 bytes), coms: MoveByteVector (24 bytes) }
    let result_ptr = result_addr as *const u8;
    let proof_vec = *(result_ptr as *const MoveByteVector);
    ptr::write(out_proof, proof_vec);
    let coms_vec = *(result_ptr.add(24) as *const MoveByteVector);
    coms_vec
}

// --- event native functions (no-ops) ---

#[export_name = "move_native_event_write_module_event_to_store"]
unsafe extern "C" fn event_write_module_event_to_store(_type_desc: *const u8, _msg: *const u8) {
    // No-op: events are fire-and-forget in our PolkaVM context
}

#[export_name = "move_native_event_write_to_event_store"]
unsafe extern "C" fn event_write_to_event_store(
    _type_desc: *const u8,
    _guid: *const MoveByteVector,
    _count: u64,
    _msg: *const u8,
) {
    // No-op: deprecated event function
}

// --- transaction_context native functions ---

#[export_name = "move_native_transaction_context_chain_id_internal"]
unsafe extern "C" fn transaction_context_chain_id_internal() -> u8 {
    imports::chain_id_internal() as u8
}

#[export_name = "move_native_transaction_context_get_txn_hash"]
unsafe extern "C" fn transaction_context_get_txn_hash() -> MoveByteVector {
    let ptr = imports::txn_hash();
    *(ptr as *const MoveByteVector)
}

#[export_name = "move_native_transaction_context_get_script_hash"]
unsafe extern "C" fn transaction_context_get_script_hash() -> MoveByteVector {
    MoveByteVector {
        ptr: core::ptr::null_mut(),
        capacity: 0,
        length: 0,
    }
}

#[export_name = "move_native_transaction_context_max_gas_amount_internal"]
unsafe extern "C" fn transaction_context_max_gas_amount_internal() -> u64 {
    imports::max_gas_amount()
}

#[export_name = "move_native_transaction_context_gas_unit_price_internal"]
unsafe extern "C" fn transaction_context_gas_unit_price_internal() -> u64 {
    imports::gas_unit_price()
}

#[export_name = "move_native_transaction_context_sender_internal"]
unsafe extern "C" fn transaction_context_sender_internal(out: *mut u8) {
    imports::sender_address(out);
}

#[export_name = "move_native_transaction_context_gas_payer_internal"]
unsafe extern "C" fn transaction_context_gas_payer_internal(out: *mut u8) {
    // Gas payer is same as sender in our test environment
    imports::sender_address(out);
}

#[export_name = "move_native_transaction_context_generate_unique_address"]
unsafe extern "C" fn transaction_context_generate_unique_address(out: *mut u8) {
    imports::generate_unique_addr(out);
}

#[export_name = "move_native_transaction_context_secondary_signers_internal"]
unsafe extern "C" fn transaction_context_secondary_signers_internal() -> MoveByteVector {
    // No secondary signers in test environment — return empty vector
    MoveByteVector {
        ptr: core::ptr::null_mut(),
        capacity: 0,
        length: 0,
    }
}

// Note: entry_function_payload_internal and multisig_payload_internal
// are intercepted in the translator (return None inline), no guest export needed.

#[export_name = "move_native_transaction_context_monotonically_increasing_counter_internal"]
unsafe extern "C" fn transaction_context_monotonically_increasing_counter_internal(
    _timestamp_us: u64,
) -> u128 {
    // Stub: return 0 in test environment
    0
}

#[export_name = "move_native_transaction_context_monotonically_increasing_counter_internal_for_test_only"]
unsafe extern "C" fn transaction_context_monotonically_increasing_counter_internal_for_test_only(
) -> u128 {
    0
}

// --- permissioned_signer native functions ---

#[export_name = "move_native_permissioned_signer_is_permissioned_signer_impl"]
unsafe extern "C" fn permissioned_signer_is_permissioned_signer_impl(_s: *const u8) -> bool {
    // No permissioned signers in our environment
    false
}

#[export_name = "move_native_permissioned_signer_permission_address"]
unsafe extern "C" fn permissioned_signer_permission_address(_s: *const u8, out: *mut u8) {
    // Return zero address
    core::ptr::write_bytes(out, 0, 32);
}

// --- misc framework native stubs ---

#[export_name = "move_native_object_exists_at"]
unsafe extern "C" fn object_exists_at_native(_td: *const u8, _addr: *const u8) -> bool {
    false
}

#[export_name = "move_native_randomness_is_unbiasable"]
unsafe extern "C" fn randomness_is_unbiasable() -> bool {
    false
}

#[export_name = "move_native_randomness_fetch_and_increment_txn_counter"]
unsafe extern "C" fn randomness_fetch_and_increment_txn_counter() -> MoveByteVector {
    MoveByteVector {
        ptr: core::ptr::null_mut(),
        capacity: 0,
        length: 0,
    }
}

#[export_name = "move_native_function_info_check_dispatch_type_compatibility_impl"]
unsafe extern "C" fn function_info_check_dispatch_type_compatibility_impl(
    _lhs: *const u8,
    _rhs: *const u8,
) -> bool {
    false
}

#[export_name = "move_native_function_info_is_identifier"]
unsafe extern "C" fn function_info_is_identifier(_s: *const u8) -> bool {
    true
}

#[export_name = "move_native_function_info_load_function_impl"]
unsafe extern "C" fn function_info_load_function_impl(_f: *const u8) {
    // No-op
}

#[export_name = "move_native_state_storage_get_state_storage_usage_only_at_epoch_beginning"]
unsafe extern "C" fn state_storage_get_usage(out: *mut u8) {
    // Return zeroed Usage struct
    core::ptr::write_bytes(out, 0, 16);
}

#[export_name = "move_native_code_request_publish"]
unsafe extern "C" fn code_request_publish(
    _owner: *const u8,
    _expected: *const u8,
    _bundle: *const u8,
    _policy: u64,
) {
    // No-op
}

#[export_name = "move_native_code_request_publish_with_allowed_deps"]
unsafe extern "C" fn code_request_publish_with_allowed_deps(
    _owner: *const u8,
    _expected: *const u8,
    _bundle: *const u8,
    _allowed: *const u8,
    _policy: u64,
) {
    // No-op
}

#[export_name = "move_native_transaction_context_validator_txn_enabled_internal"]
unsafe extern "C" fn transaction_context_validator_txn_enabled_internal(
    _config_bytes: *const u8,
) -> bool {
    false
}

#[export_name = "move_native_consensus_config_validator_txn_enabled_internal"]
unsafe extern "C" fn consensus_config_validator_txn_enabled_internal(
    _config_bytes: *const u8,
) -> bool {
    false
}

// --- aggregator_v2 native functions ---

// Helper: read u64 or u128 value from a pointer based on type descriptor
unsafe fn aggregator_read_int(ptr: *const u8, td: *const u8) -> u128 {
    let move_type = &*(td as *const MoveType);
    match move_type.type_desc {
        TypeDesc::U64 => *(ptr as *const u64) as u128,
        TypeDesc::U128 => *(ptr as *const u128),
        _ => {
            move_rt_abort(0xdead);
            core::hint::unreachable_unchecked()
        }
    }
}

// Helper: write u64 or u128 value to a pointer based on type descriptor
unsafe fn aggregator_write_int(ptr: *mut u8, td: *const u8, val: u128) {
    let move_type = &*(td as *const MoveType);
    match move_type.type_desc {
        TypeDesc::U64 => *(ptr as *mut u64) = val as u64,
        TypeDesc::U128 => *(ptr as *mut u128) = val,
        _ => {
            move_rt_abort(0xdead);
            core::hint::unreachable_unchecked()
        }
    }
}

// Helper: get the byte size of an int element from type descriptor
unsafe fn aggregator_int_size(td: *const u8) -> usize {
    let move_type = &*(td as *const MoveType);
    match move_type.type_desc {
        TypeDesc::U64 => 8,
        TypeDesc::U128 => 16,
        _ => {
            move_rt_abort(0xdead);
            core::hint::unreachable_unchecked()
        }
    }
}

// Helper: get max value for type
unsafe fn aggregator_max_for_type(td: *const u8) -> u128 {
    let move_type = &*(td as *const MoveType);
    match move_type.type_desc {
        TypeDesc::U64 => u64::MAX as u128,
        TypeDesc::U128 => u128::MAX,
        _ => {
            move_rt_abort(0xdead);
            core::hint::unreachable_unchecked()
        }
    }
}

/// create_aggregator<I>(max_value: I) -> Aggregator<I>
/// Aggregator struct: { value: I, max_value: I }
/// Generic return → output pointer is last arg
#[export_name = "move_native_aggregator_v2_create_aggregator"]
unsafe extern "C" fn aggregator_v2_create_aggregator(
    type_desc: *const u8,
    max_value: *const u8,
    out: *mut u8,
) {
    let int_size = aggregator_int_size(type_desc);
    // value = 0
    core::ptr::write_bytes(out, 0, int_size);
    // max_value = max_value arg
    core::ptr::copy_nonoverlapping(max_value, out.add(int_size), int_size);
}

/// create_unbounded_aggregator<I>() -> Aggregator<I>
#[export_name = "move_native_aggregator_v2_create_unbounded_aggregator"]
unsafe extern "C" fn aggregator_v2_create_unbounded_aggregator(type_desc: *const u8, out: *mut u8) {
    let int_size = aggregator_int_size(type_desc);
    let max_val = aggregator_max_for_type(type_desc);
    // value = 0
    core::ptr::write_bytes(out, 0, int_size);
    // max_value = MAX
    aggregator_write_int(out.add(int_size), type_desc, max_val);
}

/// try_add<I>(&mut self, value: I) -> bool
/// self is &mut Aggregator<I> = pointer to {value: I, max_value: I}
/// For generic `value` param: passed as pointer
#[export_name = "move_native_aggregator_v2_try_add"]
unsafe extern "C" fn aggregator_v2_try_add(
    type_desc: *const u8,
    self_ptr: *mut u8,
    value_ptr: *const u8,
) -> u32 {
    let int_size = aggregator_int_size(type_desc);
    let current = aggregator_read_int(self_ptr, type_desc);
    let add_val = aggregator_read_int(value_ptr, type_desc);
    let max_val = aggregator_read_int(self_ptr.add(int_size), type_desc);
    if current + add_val <= max_val {
        aggregator_write_int(self_ptr, type_desc, current + add_val);
        1 // true
    } else {
        0 // false
    }
}

/// try_sub<I>(&mut self, value: I) -> bool
#[export_name = "move_native_aggregator_v2_try_sub"]
unsafe extern "C" fn aggregator_v2_try_sub(
    type_desc: *const u8,
    self_ptr: *mut u8,
    value_ptr: *const u8,
) -> u32 {
    let current = aggregator_read_int(self_ptr, type_desc);
    let sub_val = aggregator_read_int(value_ptr, type_desc);
    if current >= sub_val {
        aggregator_write_int(self_ptr, type_desc, current - sub_val);
        1 // true
    } else {
        0 // false
    }
}

/// is_at_least_impl<I>(&self, min_amount: I) -> bool
#[export_name = "move_native_aggregator_v2_is_at_least_impl"]
unsafe extern "C" fn aggregator_v2_is_at_least_impl(
    type_desc: *const u8,
    self_ptr: *const u8,
    min_ptr: *const u8,
) -> u32 {
    let current = aggregator_read_int(self_ptr, type_desc);
    let min_val = aggregator_read_int(min_ptr, type_desc);
    if current >= min_val {
        1
    } else {
        0
    }
}

/// read<I>(&self) -> I
/// Return type is TypeParameter → generic return via output pointer
#[export_name = "move_native_aggregator_v2_read"]
unsafe extern "C" fn aggregator_v2_read(type_desc: *const u8, self_ptr: *const u8, out: *mut u8) {
    let int_size = aggregator_int_size(type_desc);
    core::ptr::copy_nonoverlapping(self_ptr, out, int_size);
}

/// snapshot<I>(&self) -> AggregatorSnapshot<I>
/// AggregatorSnapshot has one field: value: I
/// Generic return → output pointer
#[export_name = "move_native_aggregator_v2_snapshot"]
unsafe extern "C" fn aggregator_v2_snapshot(
    type_desc: *const u8,
    self_ptr: *const u8,
    out: *mut u8,
) {
    let int_size = aggregator_int_size(type_desc);
    // Copy just the value field (first field of Aggregator)
    core::ptr::copy_nonoverlapping(self_ptr, out, int_size);
}

/// create_snapshot<I>(value: I) -> AggregatorSnapshot<I>
/// Generic return → output pointer
#[export_name = "move_native_aggregator_v2_create_snapshot"]
unsafe extern "C" fn aggregator_v2_create_snapshot(
    type_desc: *const u8,
    value_ptr: *const u8,
    out: *mut u8,
) {
    let int_size = aggregator_int_size(type_desc);
    core::ptr::copy_nonoverlapping(value_ptr, out, int_size);
}

/// read_snapshot<I>(&self) -> I
/// Generic return → output pointer
#[export_name = "move_native_aggregator_v2_read_snapshot"]
unsafe extern "C" fn aggregator_v2_read_snapshot(
    type_desc: *const u8,
    self_ptr: *const u8,
    out: *mut u8,
) {
    let int_size = aggregator_int_size(type_desc);
    core::ptr::copy_nonoverlapping(self_ptr, out, int_size);
}

// --- Aggregator V1 stubs (deprecated but still referenced by framework modules) ---

#[no_mangle]
#[export_name = "move_native_aggregator_factory_new_aggregator"]
unsafe extern "C" fn aggregator_factory_new_aggregator(
    _table_handle: u128,
    _key: u128,
    _limit: u128,
    out: *mut u128,
) {
    // Return a zero aggregator handle
    *out = 0;
}

#[no_mangle]
#[export_name = "move_native_aggregator_add"]
unsafe extern "C" fn aggregator_add(_agg: *mut u8, _value: u128) {
    // no-op stub
}

#[no_mangle]
#[export_name = "move_native_aggregator_sub"]
unsafe extern "C" fn aggregator_sub(_agg: *mut u8, _value: u128) {
    // no-op stub
}

#[no_mangle]
#[export_name = "move_native_aggregator_read"]
unsafe extern "C" fn aggregator_read(_agg: *const u8) -> u128 {
    0
}

#[no_mangle]
#[export_name = "move_native_aggregator_destroy"]
unsafe extern "C" fn aggregator_destroy(_agg: *mut u8) {
    // no-op stub
}

// --- Additional aggregator_v2 stubs ---

#[no_mangle]
#[export_name = "move_native_aggregator_v2_copy_snapshot"]
unsafe extern "C" fn aggregator_v2_copy_snapshot(
    type_desc: *const u8,
    self_ptr: *const u8,
    out: *mut u8,
) {
    let int_size = aggregator_int_size(type_desc);
    core::ptr::copy_nonoverlapping(self_ptr, out, int_size);
}

#[no_mangle]
#[export_name = "move_native_aggregator_v2_create_derived_string"]
unsafe extern "C" fn aggregator_v2_create_derived_string(
    _value_ptr: *const MoveByteVector,
    out: *mut MoveByteVector,
) {
    // Return empty derived string
    *out = MoveByteVector::from_rust_vec(alloc::vec![]);
}

#[no_mangle]
#[export_name = "move_native_aggregator_v2_read_derived_string"]
unsafe extern "C" fn aggregator_v2_read_derived_string(
    _self_ptr: *const u8,
    out: *mut MoveByteVector,
) {
    *out = MoveByteVector::from_rust_vec(alloc::vec![]);
}

#[no_mangle]
#[export_name = "move_native_aggregator_v2_derive_string_concat"]
unsafe extern "C" fn aggregator_v2_derive_string_concat(
    _prefix: *const MoveByteVector,
    _snapshot: *const u8,
    _suffix: *const MoveByteVector,
    out: *mut u8,
) {
    // Zero-init the output (DerivedStringSnapshot stub)
    core::ptr::write_bytes(out, 0, 64);
}

#[no_mangle]
#[export_name = "move_native_aggregator_v2_string_concat"]
unsafe extern "C" fn aggregator_v2_string_concat(
    _before: *const MoveByteVector,
    _snapshot: *const u8,
    _after: *const MoveByteVector,
    out: *mut u8,
) {
    core::ptr::write_bytes(out, 0, 64);
}

// --- crypto_algebra stubs (abstract algebra for cryptographic groups) ---

#[no_mangle]
#[export_name = "move_native_crypto_algebra_abort_unless_cryptography_algebra_natives_enabled"]
unsafe extern "C" fn crypto_algebra_abort_unless_enabled() {
    // no-op: allow algebra operations
}

macro_rules! crypto_algebra_stub {
    ($export:literal, $name:ident $(, $arg:ident : $ty:ty)*) => {
        #[no_mangle]
        #[export_name = $export]
        unsafe extern "C" fn $name($(_: $ty),*) {
            // Stub: crypto_algebra not yet implemented
            move_rt_abort(0xFF);
        }
    };
    ($export:literal, $name:ident, ret $ret_ty:ty $(, $arg:ident : $ty:ty)*) => {
        #[no_mangle]
        #[export_name = $export]
        unsafe extern "C" fn $name($(_: $ty),*) -> $ret_ty {
            move_rt_abort(0xFF);
            core::hint::unreachable_unchecked()
        }
    };
}

crypto_algebra_stub!("move_native_crypto_algebra_add_internal", crypto_algebra_add, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_sub_internal", crypto_algebra_sub, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_mul_internal", crypto_algebra_mul, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_div_internal", crypto_algebra_div, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_neg_internal", crypto_algebra_neg, a: *const u8, b: *const u8, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_inv_internal", crypto_algebra_inv, a: *const u8, b: *const u8, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_sqr_internal", crypto_algebra_sqr, a: *const u8, b: *const u8, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_double_internal", crypto_algebra_double, a: *const u8, b: *const u8, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_eq_internal", crypto_algebra_eq, ret bool, a: *const u8, b: *const u8, c: *const u8);
crypto_algebra_stub!("move_native_crypto_algebra_from_u64_internal", crypto_algebra_from_u64, a: *const u8, b: u64, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_zero_internal", crypto_algebra_zero, a: *const u8, b: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_one_internal", crypto_algebra_one, a: *const u8, b: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_serialize_internal", crypto_algebra_serialize, a: *const u8, b: *const u8, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_deserialize_internal", crypto_algebra_deserialize, a: *const u8, b: *const u8, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_order_internal", crypto_algebra_order, a: *const u8, b: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_upcast_internal", crypto_algebra_upcast, a: *const u8, b: *const u8, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_downcast_internal", crypto_algebra_downcast, a: *const u8, b: *const u8, c: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_hash_to_internal", crypto_algebra_hash_to, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_pairing_internal", crypto_algebra_pairing, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_multi_pairing_internal", crypto_algebra_multi_pairing, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_multi_scalar_mul_internal", crypto_algebra_multi_scalar_mul, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_scalar_mul_internal", crypto_algebra_scalar_mul, a: *const u8, b: *const u8, c: *const u8, d: *mut u8);
crypto_algebra_stub!("move_native_crypto_algebra_rand_insecure_internal", crypto_algebra_rand_insecure, a: *const u8, b: *mut u8);

// --- debug stubs ---

#[no_mangle]
#[export_name = "move_native_debug_native_print"]
unsafe extern "C" fn debug_native_print(_type_desc: *const u8, _val: *const u8) {
    // no-op stub for native_print
}

#[no_mangle]
#[export_name = "move_native_debug_native_stack_trace"]
unsafe extern "C" fn debug_native_stack_trace(out: *mut MoveByteVector) {
    *out = MoveByteVector::from_rust_vec(b"<stack trace unavailable>".to_vec());
}

// --- string_utils stubs ---

#[no_mangle]
#[export_name = "move_native_string_utils_native_format"]
unsafe extern "C" fn string_utils_native_format(
    _type_desc: *const u8,
    _val: *const u8,
    _type_tag: bool,
    _canonicalize: bool,
    _single_line: bool,
    _include_int_types: bool,
    out: *mut MoveByteVector,
) {
    *out = MoveByteVector::from_rust_vec(b"<format unavailable>".to_vec());
}

#[no_mangle]
#[export_name = "move_native_string_utils_native_format_list"]
unsafe extern "C" fn string_utils_native_format_list(
    _fmt: *const MoveByteVector,
    _val: *const u8,
    out: *mut MoveByteVector,
) {
    *out = MoveByteVector::from_rust_vec(b"<format_list unavailable>".to_vec());
}

// --- util stubs ---

#[no_mangle]
#[export_name = "move_native_util_from_bytes"]
unsafe extern "C" fn util_from_bytes(
    _type_desc: *const u8,
    _bytes: *const MoveByteVector,
    out: *mut u8,
) {
    // Zero-initialize output; real deserialization not yet implemented
    core::ptr::write_bytes(out, 0, 64);
}

// --- object stubs ---

#[no_mangle]
#[export_name = "move_native_object_create_user_derived_object_address_impl"]
unsafe extern "C" fn object_create_user_derived_object_address_impl(
    _source: *const u8,
    _derive_from: *const u8,
    out: *mut u8,
) {
    // Return zero address
    core::ptr::write_bytes(out, 0, 32);
}

// --- event stubs ---

#[no_mangle]
#[export_name = "move_native_event_emitted_events"]
unsafe extern "C" fn event_emitted_events(_type_desc: *const u8, out: *mut MoveByteVector) {
    // Return empty vector
    *out = MoveByteVector::from_rust_vec(alloc::vec![]);
}

#[no_mangle]
#[export_name = "move_native_event_emitted_events_by_handle"]
unsafe extern "C" fn event_emitted_events_by_handle(
    _type_desc: *const u8,
    _handle: u64,
    out: *mut MoveByteVector,
) {
    *out = MoveByteVector::from_rust_vec(alloc::vec![]);
}

// --- ristretto255 missing stub ---

#[no_mangle]
#[export_name = "move_native_ristretto255_random_scalar_internal"]
unsafe extern "C" fn ristretto255_random_scalar_internal(out: *mut u8) {
    // Return zero scalar (stub)
    core::ptr::write_bytes(out, 0, 32);
}

// --- permissioned_signer stubs ---

#[no_mangle]
#[export_name = "move_native_permissioned_signer_signer_from_permissioned_handle_impl"]
unsafe extern "C" fn permissioned_signer_signer_from_permissioned_handle_impl(
    _master: *const u8,
    _permissions_storage_addr: *const u8,
    _permission_addr: *const u8,
    out: *mut u8,
) {
    // Return zero signer (stub)
    core::ptr::write_bytes(out, 0, 32);
}

// --- consensus_config stubs (may alias existing) ---

// validator_txn_enabled_internal is already defined above

// --- dispatchable_fungible_asset stubs ---
// These are intercepted at the translator level (translate.rs),
// but in case they're referenced as symbols, provide empty stubs.

#[no_mangle]
#[export_name = "move_native_dispatchable_fungible_asset_dispatchable_withdraw"]
unsafe extern "C" fn dispatchable_fa_withdraw() {
    move_rt_abort(0xFF);
}

#[no_mangle]
#[export_name = "move_native_dispatchable_fungible_asset_dispatchable_deposit"]
unsafe extern "C" fn dispatchable_fa_deposit() {
    move_rt_abort(0xFF);
}

#[no_mangle]
#[export_name = "move_native_dispatchable_fungible_asset_dispatchable_derived_balance"]
unsafe extern "C" fn dispatchable_fa_derived_balance() {
    move_rt_abort(0xFF);
}

#[no_mangle]
#[export_name = "move_native_dispatchable_fungible_asset_dispatchable_derived_supply"]
unsafe extern "C" fn dispatchable_fa_derived_supply() {
    move_rt_abort(0xFF);
}

// --- account_abstraction stub ---

#[no_mangle]
#[export_name = "move_native_account_abstraction_dispatchable_authenticate"]
unsafe extern "C" fn account_abstraction_dispatchable_authenticate() {
    move_rt_abort(0xFF);
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
