use core::alloc::{GlobalAlloc, Layout};

use crate::host::{abort, ALLOC_CODE};

pub struct DummyAlloc;

unsafe impl GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        abort(ALLOC_CODE);
        // unreachable
        core::hint::unreachable_unchecked()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // no-op
    }
}

#[global_allocator]
static GLOBAL: DummyAlloc = DummyAlloc;
