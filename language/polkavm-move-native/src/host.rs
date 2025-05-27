use core::mem::MaybeUninit;

extern crate std;

use log::info;
use polkavm::{Caller, Instance, Linker, MemoryAccessError, Module, RawInstance};

use crate::{
    types::{MoveSigner, MoveType, TypeDesc},
    ALLOC_CODE, PANIC_CODE,
};

#[derive(Debug)]
pub enum ProgramError {
    // move abort called with code
    Abort(u64),
    // panics are Rust construct, and are marked with special abort code - it usually means native lib did something weird
    NativeLibPanic,
    // there is no allocator available for guest program (Move program to be exact), any calls to malloc result in abort with special code
    NativeLibAllocatorCall,
    // memory access error when we work inside callbacks and do memory reading
    MemoryAccess(std::string::String),
}

impl From<MemoryAccessError> for ProgramError {
    fn from(value: MemoryAccessError) -> Self {
        ProgramError::MemoryAccess(value.to_string())
    }
}

pub type PolkaError = polkavm::Error;
pub type LinkerResult<T> = Result<T, PolkaError>;

pub type MoveProgramLinker = Linker<(), ProgramError>;

// creates new polkavm linker with native functions prepared for move program
// all native functions declared by move std must defined here
pub fn new_move_program_linker() -> LinkerResult<MoveProgramLinker> {
    let mut linker: MoveProgramLinker = Linker::new();

    // additional "native" function used by move program and also exposed by host
    // it is just for testing/debuging only
    linker.define_typed(
        "debug_print",
        |caller: Caller, ptr_to_type: u32, ptr_to_data: u32| {
            info!("debug_print called. type ptr: {ptr_to_type:x} Data ptr: {ptr_to_data:x}");
            let move_type: MoveType = load_from(caller.instance, ptr_to_type)?;
            info!("type info: {:?}", move_type);
            let move_value: u64 = load_from(caller.instance, ptr_to_data)?;
            info!("value: {move_value}");
            Result::<(), ProgramError>::Ok(())
        },
    )?;

    linker.define_typed("abort", |code: u64| {
        let program_error = match code {
            PANIC_CODE => ProgramError::NativeLibPanic,
            ALLOC_CODE => ProgramError::NativeLibAllocatorCall,
            _ => ProgramError::Abort(code),
        };
        Result::<(), _>::Err(program_error)
    })?;
    Ok(linker)
}

// we probably gonna need to wrap polkavm instance too
pub struct MemAllocator {
    next_available_address: u32,
}

impl MemAllocator {
    pub fn init(module: &Module) -> Self {
        let memory_map = module.memory_map();
        Self {
            next_available_address: memory_map.aux_data_address(),
        }
    }
    // this can be generalized to any arbitrary type &T
    pub fn load_to<T>(
        &mut self,
        instance: &mut Instance<T, ProgramError>,
        signer: &MoveSigner,
    ) -> Result<u32, MemoryAccessError> {
        let size_to_write = size_of::<MoveSigner>();
        // TODO: add available mem checking

        let slice = unsafe {
            core::slice::from_raw_parts((signer as *const MoveSigner) as *const u8, size_to_write)
        };

        let address_to_write = self.next_available_address;
        instance.write_memory(address_to_write, slice)?;

        self.next_available_address += size_to_write as u32;

        Ok(address_to_write)
    }
}

fn load_from<T: Sized>(instance: &mut RawInstance, address: u32) -> Result<T, MemoryAccessError> {
    let mut uninit = MaybeUninit::<T>::uninit();
    unsafe {
        let dst_bytes: &mut [u8] =
            core::slice::from_raw_parts_mut(uninit.as_mut_ptr() as *mut u8, size_of::<T>());
        instance.read_memory_into(address, dst_bytes.as_mut())?;
        Ok(uninit.assume_init())
    }
}

impl core::fmt::Debug for MoveType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MoveType")
            //.field("name", &self.name)
            .field("type", &self.type_desc)
            .finish()
    }
}

impl core::fmt::Debug for TypeDesc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bool => write!(f, "Bool"),
            Self::U8 => write!(f, "U8"),
            Self::U16 => write!(f, "U16"),
            Self::U32 => write!(f, "U32"),
            Self::U64 => write!(f, "U64"),
            Self::U128 => write!(f, "U128"),
            Self::U256 => write!(f, "U256"),
            Self::Address => write!(f, "Address"),
            Self::Signer => write!(f, "Signer"),
            Self::Vector => write!(f, "Vector"),
            Self::Struct => write!(f, "Struct"),
            Self::Reference => write!(f, "Reference"),
        }
    }
}
