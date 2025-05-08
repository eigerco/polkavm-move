use crate::host::{abort, PANIC_CODE};

#[cfg(feature = "polkavm")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        abort(PANIC_CODE);
        core::hint::unreachable_unchecked()
    }
}
