use crate::{
    conv::{borrow_move_value_as_rust_value, BorrowedTypedMoveValue as BTMV},
    types::{AnyValue, MoveType},
    vector::TypedMoveBorrowedRustVec,
};
use core::cmp::Ordering;

/// Recursively compare two Move values of the same type, returning
/// a lexicographic ordering.
///
/// # Safety
///
/// `type_v` must accurately describe the layout of both `a` and `b`.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn compare(type_v: &MoveType, a: &AnyValue, b: &AnyValue) -> Ordering {
    let va = borrow_move_value_as_rust_value(type_v, a);
    let vb = borrow_move_value_as_rust_value(type_v, b);

    match (va, vb) {
        (BTMV::Bool(a), BTMV::Bool(b)) => a.cmp(b),
        (BTMV::U8(a), BTMV::U8(b)) => a.cmp(b),
        (BTMV::U16(a), BTMV::U16(b)) => a.cmp(b),
        (BTMV::U32(a), BTMV::U32(b)) => a.cmp(b),
        (BTMV::U64(a), BTMV::U64(b)) => a.cmp(b),
        (BTMV::U128(a), BTMV::U128(b)) => a.cmp(b),
        (BTMV::U256(a), BTMV::U256(b)) => {
            // U256 is [u128; 2] stored little-endian: compare high word first
            let ord = a.0[1].cmp(&b.0[1]);
            if ord != Ordering::Equal {
                return ord;
            }
            a.0[0].cmp(&b.0[0])
        }
        (BTMV::Address(a), BTMV::Address(b)) => a.0.cmp(&b.0),
        (BTMV::Signer(a), BTMV::Signer(b)) => (a.0).0.cmp(&(b.0).0),
        (BTMV::Vector(t1, utv1), BTMV::Vector(t2, utv2)) => {
            assert_eq!(t1.type_desc, t2.type_desc);
            let v1 = TypedMoveBorrowedRustVec::new(&t1, utv1);
            let v2 = TypedMoveBorrowedRustVec::new(&t2, utv2);
            v1.cmp_ord(&v2)
        }
        (BTMV::Struct(t1, anyv1), BTMV::Struct(_t2, anyv2)) => compare_struct(&t1, anyv1, anyv2),
        (BTMV::Reference(_, _), BTMV::Reference(_, _)) => {
            unreachable!("reference comparison impossible in this context")
        }
        _ => {
            unreachable!("compare: mismatched value types")
        }
    }
}

/// Compare two struct values field-by-field lexicographically.
unsafe fn compare_struct(type_ve: &MoveType, s1: &AnyValue, s2: &AnyValue) -> Ordering {
    let st_info = (*(type_ve.type_info)).struct_;
    let fields1 = crate::structs::walk_fields(&st_info, s1);
    let fields2 = crate::structs::walk_fields(&st_info, s2);
    for ((fld_ty1, fld_ref1, _), (_, fld_ref2, _)) in Iterator::zip(fields1, fields2) {
        let ord = compare(fld_ty1, fld_ref1, fld_ref2);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}
