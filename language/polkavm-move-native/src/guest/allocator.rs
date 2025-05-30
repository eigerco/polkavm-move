use core::alloc::{GlobalAlloc, Layout};

use super::imports::guest_alloc;

pub struct DummyAlloc;

unsafe impl GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let address = guest_alloc(layout.size(), layout.align());
        if address == 0 {
            core::ptr::null_mut()
        } else {
            address as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // no-op
    }
}

#[global_allocator]
static GLOBAL: DummyAlloc = DummyAlloc;
