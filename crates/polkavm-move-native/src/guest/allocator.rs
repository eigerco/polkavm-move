use core::alloc::{GlobalAlloc, Layout};

use super::imports::guest_alloc;

pub struct DummyAlloc;

unsafe impl GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // We need to use u64 as the FFI interface can not do bigger integers.
        let address = guest_alloc(
            u64::try_from(layout.size()).unwrap_or(u64::MAX),
            u64::try_from(layout.align()).unwrap_or(u64::MAX),
        );
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
