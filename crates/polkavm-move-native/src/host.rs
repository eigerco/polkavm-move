extern crate std;

use core::mem::MaybeUninit;
use log::debug;
use polkavm::{MemoryAccessError, MemoryMap, RawInstance};

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

// we probably gonna need to wrap polkavm instance too
pub struct MemAllocator {
    base: u32,
    size: u32,
    offset: u32,
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
        }
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

        debug!(
            "Allocated {size} bytes at aligned address: 0x{address:#X} (offset: {aligned_offset})"
        );

        Ok(address)
    }

    /// Copy memory host -> guest (aux)
    pub fn copy_to_guest<T: Sized + Copy>(
        &mut self,
        instance: &mut RawInstance,
        value: &T,
    ) -> Result<u32, MemoryAccessError> {
        debug!(
            "Copying value of type {} to guest memory",
            core::any::type_name::<T>()
        );
        let size_to_write = core::mem::size_of::<T>();
        let address = self.alloc(size_to_write, core::mem::align_of::<T>())?;

        // safety: we know we have memory, we just checked
        let slice =
            unsafe { core::slice::from_raw_parts((value as *const T) as *const u8, size_to_write) };

        instance.write_memory(address, slice)?;

        Ok(address)
    }

    /// Copy a byte slice (host -> guest aux memory)
    pub fn copy_bytes_to_guest(
        &mut self,
        instance: &mut RawInstance,
        bytes: &[u8],
    ) -> Result<u32, MemoryAccessError> {
        let size = bytes.len();
        let align = core::mem::align_of::<u8>(); // usually 1, but explicit for clarity

        debug!("Copying {size} bytes to guest memory with alignment {align}");

        let address = self.alloc(size, align)?;

        instance.write_memory(address, bytes)?;

        Ok(address)
    }
}

/// Copy memory guest (aux) -> host
pub fn copy_from_guest<T: Sized + Copy>(
    instance: &mut RawInstance,
    address: u32,
) -> Result<T, MemoryAccessError> {
    debug!(
        "Copying value of type {} from guest memory at address 0x{:X}",
        core::any::type_name::<T>(),
        address
    );
    let mut uninit = MaybeUninit::<T>::uninit();
    unsafe {
        let dst_bytes: &mut [u8] =
            core::slice::from_raw_parts_mut(uninit.as_mut_ptr() as *mut u8, size_of::<T>());
        debug!(
            "Reading {} bytes from guest memory at address 0x{:X}",
            size_of::<T>(),
            address
        );
        instance.read_memory_into(address, dst_bytes.as_mut())?;
        Ok(uninit.assume_init())
    }
}

/// Copy memory guest (aux) -> host into a Vec<u8>
pub fn copy_bytes_from_guest(
    instance: &mut RawInstance,
    address: u32,
    length: usize,
) -> Result<std::vec::Vec<u8>, MemoryAccessError> {
    debug!("Copying {length} bytes from guest memory at address 0x{address:X}");
    let mut uninit: std::boxed::Box<[MaybeUninit<u8>]> = std::boxed::Box::new_uninit_slice(length);

    // Step 2: let `read_memory_into` initialize it
    let initialized: &mut [u8] = instance.read_memory_into(address, &mut *uninit)?;

    // Step 3: create a Vec<u8> from the slice
    Ok(initialized.to_vec())
}
