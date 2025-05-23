use log::info;
use polkavm::{Instance, Linker, MemoryAccessError, Module};

use crate::types::MoveSigner;

#[derive(Debug)]
pub enum ProgramError {
    // move abort called with code
    Abort(u64),
}

pub type PolkaError = polkavm::Error;
pub type LinkerResult<T> = Result<T, PolkaError>;

pub type MoveProgramLinker<T> = Linker<T, ProgramError>;

// creates new polkavm linker with native functions prepared for move program
// all native functions declared by move std must defined here
pub fn new_move_program_linker<T>() -> LinkerResult<MoveProgramLinker<T>> {
    let mut linker: MoveProgramLinker<T> = Linker::new();

    // additional "native" function used by move program and also exposed by host
    // it is just for testing/debuging only
    linker.define_typed("debug_print", |ptr_to_type: u32, ptr_to_data: u32| {
        info!("debug_print called. type ptr: {ptr_to_type:x} Data ptr: {ptr_to_data:x}");
        Ok(())
    })?;

    linker.define_typed("abort", |code: u64| {
        Result::<(), _>::Err(ProgramError::Abort(code))
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
            next_available_address: memory_map.heap_base(),
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
