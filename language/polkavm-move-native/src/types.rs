/// A Move vector with an untyped buffer.
///
/// Used in the API for generic vector arguments.
///
/// The only way to interact with these is to convert them from / to Rust
/// vectors or references to Rust vectors, with functions in the [`conv`]
/// module.
///
/// The only way to create and destroy them is with the
/// [`move_native_vec_empty`] and [`move_native_vec_destroy_empty`] native
/// calls.
#[repr(C)]
pub struct MoveUntypedVector {
    pub ptr: *mut u8,  // Safety: must be correctly aligned per type
    pub capacity: u64, // in typed elements, not u8
    pub length: u64,   // in typed elements, not u8
}
pub const MOVE_UNTYPED_VEC_DESC_SIZE: u64 = core::mem::size_of::<MoveUntypedVector>() as u64;

/// A Move vector of bytes.
///
/// These occur in the API enough to warrant their own type, and there are
/// dedicated functions to convert them to Rust vectors.
#[repr(C)]
pub struct MoveByteVector {
    pub ptr: *mut u8,
    pub capacity: u64,
    pub length: u64,
}

/// A Move vector of signers.
///
/// This type occurs in the native API, but it will probably be removed, in
/// favor of just using `MoveUntypedVector`.
#[repr(C)]
pub struct MoveSignerVector {
    pub ptr: *mut MoveSigner,
    pub capacity: u64,
    pub length: u64,
}

/// A reification of the Move runtime type description.
///
/// This is structured as a `TypeDesc` indicating which type a thing is,
/// and an undiscriminated union holding additional information about the
/// type.
///
/// cc runtime_types::Type
///
/// # Safety
///
/// The pointer must be to static memory and never mutated.
#[repr(C)]
pub struct MoveType {
    pub name: StaticTypeName,
    pub type_desc: TypeDesc,
    pub type_info: *const TypeInfo,
}
pub const MOVE_TYPE_DESC_SIZE: u64 = core::mem::size_of::<MoveType>() as u64;

/// # Safety
///
/// The pointer must be to static memory and never mutated.
#[repr(C)]
pub struct StaticTypeName {
    pub ptr: *const u8,
    pub len: u64,
}

unsafe impl Sync for StaticTypeName {}

pub type StaticName = StaticTypeName;

static DUMMY_TYPE_NAME_SLICE: &[u8] = b"dummy";
pub static DUMMY_TYPE_NAME: StaticTypeName = StaticTypeName {
    ptr: DUMMY_TYPE_NAME_SLICE as *const [u8] as *const u8,
    len: 5,
};

#[repr(u64)]
pub enum TypeDesc {
    Bool = 1,
    U8 = 2,
    U16 = 3,
    U32 = 4,
    U64 = 5,
    U128 = 6,
    U256 = 7,
    Address = 8,
    Signer = 9,
    Vector = 10,
    Struct = 11,
    Reference = 12,
    //MutableReference = 13,
}

#[repr(C)]
pub union TypeInfo {
    pub nothing: u8, // if no type info is needed
    pub vector: VectorTypeInfo,
    pub struct_: StructTypeInfo,
    pub struct_instantiation: u8, // todo
    pub reference: ReferenceTypeInfo,
    pub mutable_reference: ReferenceTypeInfo,
    pub ty_param: u8, // todo
}

#[repr(C)]
#[derive(Copy, Clone)]

pub struct VectorTypeInfo {
    pub element_type: &'static MoveType,
}

/// # Safety
///
/// This type is `Sync` so that it can be declared statically. The value
/// pointed to by `field_array_ptr` should not be mutated, or `Sync` will be
/// violated.
#[repr(C)]
#[derive(Copy, Clone)]

pub struct StructTypeInfo {
    /// Pointer to an array of field infos.
    ///
    /// This would ideally be a Rust static slice, but the layout is
    /// seemingly undefined.
    pub field_array_ptr: *const StructFieldInfo,
    pub field_array_len: u64,
    /// Size of the struct within an array.
    pub size: u64,
    /// Alignment of the struct.
    pub alignment: u64,
}

#[repr(C)]
pub struct StructFieldInfo {
    pub type_: MoveType,
    /// Offset in bytes within the struct.
    pub offset: u64,
    pub name: StaticName,
}

#[repr(C)]
#[derive(Copy, Clone)]

pub struct ReferenceTypeInfo {
    pub element_type: &'static MoveType,
}

#[repr(transparent)]
pub struct AnyValue(u8);

#[repr(transparent)]
pub struct MoveSigner(pub MoveAddress);

pub const ACCOUNT_ADDRESS_LENGTH: usize = 32;

/// A Move address.
///
/// This is mapped to the address size of the target platform, and may
/// differ from Move VM.
///
/// Bytes are in little-endian order.
#[repr(transparent)]
pub struct MoveAddress(pub [u8; ACCOUNT_ADDRESS_LENGTH]);

// Defined in std::type_name; not a primitive.
//
// todo how is drop glue handled?
#[repr(C)]
pub struct TypeName {
    pub name: MoveAsciiString,
}

// Defined in std::ascii; not a primitive.
//
// todo how is drop glue handled?
#[repr(C)]
pub struct MoveAsciiString {
    pub bytes: MoveByteVector,
}

// todo this would be more correct with a lifetime attached
#[repr(transparent)]
pub struct MoveUntypedReference(pub *const AnyValue);

#[repr(transparent)]
pub struct U256(pub [u128; 2]);
