// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use core::mem;

use crate::{
    types::{
        AnyValue, MoveAddress, MoveSigner, MoveType, MoveUntypedReference, MoveUntypedVector,
        TypeDesc, U256,
    },
    vector::TypedMoveBorrowedRustVec,
};

/// This is a placeholder for the unstable `ptr::invalid_mut`.
///
/// It is a potential future way to create invalid pointers, which is
/// required for correctly initializing empty vectors.
///
/// This crate initializes empty vectors knowing only the alignment of their
/// elements, but not the full type.
#[allow(clippy::useless_transmute)]
pub const fn invalid_mut<T>(addr: usize) -> *mut T {
    // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
    // We use transmute rather than a cast so tools like Miri can tell that this
    // is *not* the same as from_exposed_addr.
    // SAFETY: every valid integer is also a valid pointer (as long as you don't dereference that
    // pointer).
    unsafe { mem::transmute(addr) }
}

pub enum BorrowedTypedMoveValue<'mv> {
    Bool(&'mv bool),
    U8(&'mv u8),
    U16(&'mv u16),
    U32(&'mv u32),
    U64(&'mv u64),
    U128(&'mv u128),
    U256(&'mv U256),
    Address(&'mv MoveAddress),
    Signer(&'mv MoveSigner),
    Vector(MoveType, &'mv MoveUntypedVector),
    Struct(MoveType, &'mv AnyValue),
    Reference(MoveType, &'mv MoveUntypedReference),
    // todo
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn borrow_move_value_as_rust_value<'mv>(
    type_: &MoveType,
    value: &'mv AnyValue,
) -> BorrowedTypedMoveValue<'mv> {
    // todo need to think about the soundness of this transmute
    match type_.type_desc {
        TypeDesc::Bool => BorrowedTypedMoveValue::Bool(mem::transmute::<&AnyValue, &bool>(value)),
        TypeDesc::U8 => BorrowedTypedMoveValue::U8(mem::transmute::<&AnyValue, &u8>(value)),
        TypeDesc::U16 => BorrowedTypedMoveValue::U16(mem::transmute::<&AnyValue, &u16>(value)),
        TypeDesc::U32 => BorrowedTypedMoveValue::U32(mem::transmute::<&AnyValue, &u32>(value)),
        TypeDesc::U64 => BorrowedTypedMoveValue::U64(mem::transmute::<&AnyValue, &u64>(value)),
        TypeDesc::U128 => BorrowedTypedMoveValue::U128(mem::transmute::<&AnyValue, &u128>(value)),
        TypeDesc::U256 => BorrowedTypedMoveValue::U256(mem::transmute::<&AnyValue, &U256>(value)),
        TypeDesc::Address => {
            BorrowedTypedMoveValue::Address(mem::transmute::<&AnyValue, &MoveAddress>(value))
        }
        TypeDesc::Signer => {
            BorrowedTypedMoveValue::Signer(mem::transmute::<&AnyValue, &MoveSigner>(value))
        }
        TypeDesc::Vector => {
            let element_type = *(*type_.type_info).vector.element_type;
            let move_ref = mem::transmute::<&AnyValue, &MoveUntypedVector>(value);
            BorrowedTypedMoveValue::Vector(element_type, move_ref)
        }
        TypeDesc::Struct => {
            // Previously we stored the StructTypeInfo here. But passing the enclosing
            // MoveType instead gives routines access to the struct name (i.e., more
            // context). Otherwise we would need an uplevel pointer in StructTypeInfo or
            // to redundantly store the name there.
            BorrowedTypedMoveValue::Struct(*type_, value)
        }
        TypeDesc::Reference => {
            let element_type = *(*type_.type_info).reference.element_type;
            let move_ref = mem::transmute::<&AnyValue, &MoveUntypedReference>(value);
            BorrowedTypedMoveValue::Reference(element_type, move_ref)
        }
    }
}

/// The same as `BorrowedTypedMoveValue` but with raw pointers.
///
/// Allows for uninitialized values.
pub enum RawBorrowedTypedMoveValue {
    Bool(*mut bool),
    U8(*mut u8),
    U16(*mut u16),
    U32(*mut u32),
    U64(*mut u64),
    U128(*mut u128),
    U256(*mut U256),
    Address(*mut MoveAddress),
    Signer(*mut MoveSigner),
    Vector(MoveType, *mut MoveUntypedVector),
    Struct(MoveType, *mut AnyValue),
    #[allow(dead_code)]
    Reference(MoveType, *mut MoveUntypedReference),
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn raw_borrow_move_value_as_rust_value(
    type_: &MoveType,
    value: *mut AnyValue,
) -> RawBorrowedTypedMoveValue {
    match type_.type_desc {
        TypeDesc::Bool => {
            RawBorrowedTypedMoveValue::Bool(mem::transmute::<*mut AnyValue, *mut bool>(value))
        }
        TypeDesc::U8 => {
            RawBorrowedTypedMoveValue::U8(mem::transmute::<*mut AnyValue, *mut u8>(value))
        }
        TypeDesc::U16 => {
            RawBorrowedTypedMoveValue::U16(mem::transmute::<*mut AnyValue, *mut u16>(value))
        }
        TypeDesc::U32 => {
            RawBorrowedTypedMoveValue::U32(mem::transmute::<*mut AnyValue, *mut u32>(value))
        }
        TypeDesc::U64 => {
            RawBorrowedTypedMoveValue::U64(mem::transmute::<*mut AnyValue, *mut u64>(value))
        }
        TypeDesc::U128 => {
            RawBorrowedTypedMoveValue::U128(mem::transmute::<*mut AnyValue, *mut u128>(value))
        }
        TypeDesc::U256 => {
            RawBorrowedTypedMoveValue::U256(mem::transmute::<*mut AnyValue, *mut U256>(value))
        }
        TypeDesc::Address => RawBorrowedTypedMoveValue::Address(mem::transmute::<
            *mut AnyValue,
            *mut MoveAddress,
        >(value)),
        TypeDesc::Signer => RawBorrowedTypedMoveValue::Signer(mem::transmute::<
            *mut AnyValue,
            *mut MoveSigner,
        >(value)),
        TypeDesc::Vector => {
            let element_type = *(*type_.type_info).vector.element_type;
            let move_ref = mem::transmute::<*mut AnyValue, *mut MoveUntypedVector>(value);
            RawBorrowedTypedMoveValue::Vector(element_type, move_ref)
        }
        TypeDesc::Struct => RawBorrowedTypedMoveValue::Struct(*type_, value),
        TypeDesc::Reference => {
            let element_type = *(*type_.type_info).reference.element_type;
            let move_ref = mem::transmute::<*mut AnyValue, *mut MoveUntypedReference>(value);
            RawBorrowedTypedMoveValue::Reference(element_type, move_ref)
        }
    }
}

impl core::fmt::Debug for BorrowedTypedMoveValue<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BorrowedTypedMoveValue::Bool(v) => v.fmt(f),
            BorrowedTypedMoveValue::U8(v) => v.fmt(f),
            BorrowedTypedMoveValue::U16(v) => v.fmt(f),
            BorrowedTypedMoveValue::U32(v) => v.fmt(f),
            BorrowedTypedMoveValue::U64(v) => v.fmt(f),
            BorrowedTypedMoveValue::U128(v) => v.fmt(f),
            BorrowedTypedMoveValue::U256(v) => v.fmt(f),
            BorrowedTypedMoveValue::Address(v) => v.fmt(f),
            BorrowedTypedMoveValue::Signer(v) => v.fmt(f),
            BorrowedTypedMoveValue::Vector(t, v) => unsafe {
                let rv = TypedMoveBorrowedRustVec::new(t, v);
                rv.fmt(f)
            },
            BorrowedTypedMoveValue::Struct(t, v) => unsafe {
                let st = (*(t.type_info)).struct_;
                write!(f, "{} {{ ", t.name.as_ascii_str())?;
                let fields = crate::structs::walk_fields(&st, v);
                for (type_, ref_, fld_name) in fields {
                    let rv = borrow_move_value_as_rust_value(type_, ref_);
                    write!(f, "{}: ", fld_name.as_ascii_str())?;
                    rv.fmt(f)?;
                    f.write_str(", ")?;
                }
                f.write_str("}")?;
                Ok(())
            },
            BorrowedTypedMoveValue::Reference(t, v) => unsafe {
                let rv = borrow_move_value_as_rust_value(t, &*v.0);
                rv.fmt(f)
            },
        }
    }
}
