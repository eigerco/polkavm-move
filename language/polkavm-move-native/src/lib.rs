#![no_std]

extern crate alloc;

mod allocator;
pub(crate) mod host;
mod panic;

#[export_name = "move_rt_abort"]
unsafe extern "C" fn move_rt_abort(code: u64) {
    host::abort(code)
}

#[used]
static MOVE_RT_ABORT: unsafe extern "C" fn(code: u64) = move_rt_abort;
