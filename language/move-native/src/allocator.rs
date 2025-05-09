use core::alloc::{GlobalAlloc, Layout};

pub struct DummyAlloc;

#[global_allocator]
static GLOBAL: DummyAlloc = DummyAlloc;

unsafe impl GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // need to call polkavm runtime allocations
        core::ptr::null_mut() // always fail for now
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // no-op
    }
}
