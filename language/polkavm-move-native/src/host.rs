use log::info;
use polkavm::{Instance, Linker};

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

pub fn load_to<T>(instance: &Instance<T, ProgramError>, signer: &MoveSigner) -> u32 {
    0
}
