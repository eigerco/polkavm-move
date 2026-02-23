use core::alloc::{GlobalAlloc, Layout};

use super::polkavm_imports::get_heap_base;

static mut OFFSET: u32 = 0;
static mut HEAP_BASE_CACHED: u32 = 0;
static mut INITIALIZED: bool = false;

pub struct BumpAlloc;

unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !INITIALIZED {
            HEAP_BASE_CACHED = get_heap_base();
            INITIALIZED = true;
        }
        let size = layout.size() as u32;
        let align = layout.align() as u32;
        let cursor = OFFSET;
        let aligned = (cursor + align - 1) & !(align - 1);
        let new_end = aligned + size;
        OFFSET = new_end;
        (HEAP_BASE_CACHED + aligned) as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL: BumpAlloc = BumpAlloc;
