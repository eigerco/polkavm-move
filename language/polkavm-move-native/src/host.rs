use core::mem::MaybeUninit;

extern crate std;

use log::info;
use polkavm::{Caller, Instance, Linker, MemoryAccessError, Module, RawInstance};

use crate::{types::MoveType, ALLOC_CODE, PANIC_CODE};

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
    base: u32,
    size: usize,
    offset: u32,
}

impl MemAllocator {
    /// Initialize the memory allocator with the module's auxiliary data memory map.
    /// This must be called after the module is loaded and before any memory operations.
    /// Guest memory is allocated in the auxiliary data memory region defined in the module.
    pub fn init(module: &Module) -> Self {
        let memory_map = module.memory_map();
        Self {
            base: memory_map.aux_data_address(),
            size: memory_map.aux_data_size() as usize,
            offset: 0,
        }
    }

    /// Copy memory host -> guest (aux)
    pub fn load_to<T: Sized + Copy, U>(
        &mut self,
        instance: &mut Instance<U, ProgramError>,
        value: &T,
    ) -> Result<u32, MemoryAccessError> {
        let size_to_write = core::mem::size_of::<T>();
        if self.offset as usize + size_to_write > self.size {
            return Err(MemoryAccessError::OutOfRangeAccess {
                address: self.offset,
                length: size_to_write as u64,
            });
        }

        // safety: we know we have memory, we just checked
        let slice =
            unsafe { core::slice::from_raw_parts((value as *const T) as *const u8, size_to_write) };

        let address_to_write = self.base.checked_add(self.offset).ok_or_else(|| {
            MemoryAccessError::OutOfRangeAccess {
                address: self.offset,
                length: size_to_write as u64,
            }
        })?;
        instance.write_memory(address_to_write, slice)?;

        self.offset += size_to_write as u32;

        Ok(address_to_write)
    }
}

/// Copy memory guest (aux) -> host
fn load_from<T: Sized + Copy>(
    instance: &mut RawInstance,
    address: u32,
) -> Result<T, MemoryAccessError> {
    let mut uninit = MaybeUninit::<T>::uninit();
    unsafe {
        let dst_bytes: &mut [u8] =
            core::slice::from_raw_parts_mut(uninit.as_mut_ptr() as *mut u8, size_of::<T>());
        instance.read_memory_into(address, dst_bytes.as_mut())?;
        Ok(uninit.assume_init())
    }
}
