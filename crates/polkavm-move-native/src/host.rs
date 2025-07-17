extern crate alloc;
use polkavm::MemoryAccessError;

use crate::{allocator::MemAllocator, storage::Storage};
use alloc::{boxed::Box, string::ToString};

#[derive(Debug)]
pub enum ProgramError {
    // move abort called with code
    Abort(u64),
    // panics are Rust construct, and are marked with special abort code - it usually means native lib did something weird
    NativeLibPanic,
    // there is no allocator available for guest program (Move program to be exact), any calls to malloc result in abort with special code
    NativeLibAllocatorCall,
    // memory access error when we work inside callbacks and do memory reading
    MemoryAccess(alloc::string::String),
}

impl From<MemoryAccessError> for ProgramError {
    fn from(value: MemoryAccessError) -> Self {
        ProgramError::MemoryAccess(value.to_string())
    }
}

pub struct Runtime {
    pub allocator: MemAllocator,
    pub storage: Box<dyn Storage>,
}
