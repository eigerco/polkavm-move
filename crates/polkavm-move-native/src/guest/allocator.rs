use core::alloc::{GlobalAlloc, Layout};

/// Base of the guest heap.
const HEAP_BASE: u32 = 0x30500;
/// End of the reserved heap region (exclusive).
// const HEAP_LIMIT: u32 = 0x30200 + 0x10000; // 64 KiB
/// Cursor lives in `.bss`; zeroed on every fresh `call`/`deploy`.
// static CURSOR: AtomicU32 = AtomicU32::new(0);
static mut OFFSET: u32 = 0;

pub struct BumpAlloc;

unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size() as u32;
        let align = layout.align() as u32;
        let cursor = OFFSET;
        let aligned = (cursor + align - 1) & !(align - 1);
        let new_end = aligned + size;
        OFFSET = new_end;
        (HEAP_BASE + aligned) as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL: BumpAlloc = BumpAlloc;
