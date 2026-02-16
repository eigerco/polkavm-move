use log::trace;
use polkavm::{MemoryAccessError, MemoryMap};

pub struct MemAllocator {
    base: u32,
    size: u32,
    offset: u32,
}

impl Default for MemAllocator {
    fn default() -> Self {
        Self {
            base: 0xfffe0000,
            size: 4096,
            offset: 0,
        }
    }
}

impl MemAllocator {
    /// Initialize the memory allocator using the module's heap region.
    /// The heap must be grown via `instance.sbrk(size)` before allocating.
    pub fn init(memory_map: &MemoryMap) -> Self {
        Self {
            base: memory_map.heap_base(),
            size: memory_map.max_heap_size(),
            offset: 0,
        }
    }

    pub fn with_base(base: u32, size: u32) -> Self {
        Self {
            base,
            size,
            offset: 0,
        }
    }

    pub fn base(&self) -> u32 {
        self.base
    }

    /// Allocate guest memory in the heap region.
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
}
