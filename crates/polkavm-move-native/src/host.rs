extern crate std;

use log::{debug, trace};
use polkavm::{MemoryAccessError, MemoryMap};
use std::vec::Vec;

use crate::{
    storage::{GlobalStorage, Storage, StructTagHash},
    types::MoveAddress,
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

pub struct MemAllocator {
    base: u32,
    size: u32,
    offset: u32,
    storage: GlobalStorage,
}

impl Default for MemAllocator {
    fn default() -> Self {
        Self {
            base: 0xfffe0000,
            size: 4096,
            offset: 0,
            storage: GlobalStorage::new(),
        }
    }
}
impl MemAllocator {
    /// Initialize the memory allocator with the module's auxiliary data memory map.
    /// This must be called after the module is loaded and before any memory operations.
    /// Guest memory is allocated in the auxiliary data memory region defined in the module.
    pub fn init(memory_map: &MemoryMap) -> Self {
        Self {
            base: memory_map.aux_data_address(),
            size: memory_map.aux_data_size(),
            offset: 0,
            storage: GlobalStorage::new(),
        }
    }

    pub fn base(&self) -> u32 {
        self.base
    }

    /// Store a global value at the specified address with the given type.
    pub fn store_global(
        &mut self,
        address: MoveAddress,
        typ: StructTagHash,
        value: Vec<u8>,
    ) -> Result<(), ProgramError> {
        self.storage.store(address, typ, value)?;
        Ok(())
    }

    /// Load a global value from the specified address with the given type.
    pub fn load_global(
        &mut self,
        address: MoveAddress,
        typ: StructTagHash,
        remove: bool,
        is_mut: bool,
    ) -> Result<Vec<u8>, ProgramError> {
        let value = self.storage.load(address, typ, remove, is_mut)?;
        Ok(value)
    }

    /// Check if a global value exists at the specified address with the given type.
    pub fn exists(
        &mut self,
        address: MoveAddress,
        typ: StructTagHash,
    ) -> Result<bool, ProgramError> {
        let value = self.storage.exists(address, typ)?;
        Ok(value)
    }

    /// Release a global value at the specified address with the given tag.
    pub fn release(&mut self, address: MoveAddress, tag: [u8; 32]) {
        self.storage.release(address, tag);
    }

    /// Allocate guest memory in the auxiliary data region.
    pub fn alloc(&mut self, size: usize, align: usize) -> Result<u32, MemoryAccessError> {
        let align = align.max(1);
        let align = u32::try_from(align).map_err(|_| MemoryAccessError::OutOfRangeAccess {
            address: self.offset,
            length: size as u64,
        })?;

        let align_mask = align - 1;
        let aligned_offset = (self.offset + align_mask) & !(align_mask);

        if (aligned_offset as usize) + size > self.size as usize {
            return Err(MemoryAccessError::OutOfRangeAccess {
                address: aligned_offset,
                length: size as u64,
            });
        }

        let address =
            self.base
                .checked_add(aligned_offset)
                .ok_or(MemoryAccessError::OutOfRangeAccess {
                    address: aligned_offset,
                    length: size as u64,
                })?;

        let new_offset = aligned_offset
            .checked_add(
                u32::try_from(size).map_err(|_| MemoryAccessError::OutOfRangeAccess {
                    address: aligned_offset,
                    length: size as u64,
                })?,
            )
            .ok_or(MemoryAccessError::OutOfRangeAccess {
                address: aligned_offset,
                length: size as u64,
            })?;

        self.offset = new_offset;

        trace!(
            "Allocated {size} bytes at aligned address: 0x{address:#X} (offset: {aligned_offset})"
        );

        Ok(address)
    }

    pub fn release_all(&mut self) {
        debug!("Releasing all global storage");
        self.storage.release_all();
    }

    pub fn is_borrowed(&self, move_signer: MoveAddress, tag: [u8; 32]) -> bool {
        self.storage.is_borrowed(move_signer, tag)
    }

    pub fn update(
        &mut self,
        address: MoveAddress,
        tag: [u8; 32],
        value: Vec<u8>,
    ) -> Result<(), ProgramError> {
        self.storage.update(address, tag, value)
    }
}
