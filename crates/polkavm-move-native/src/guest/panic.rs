#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    use super::imports::abort;
    use crate::PANIC_CODE;
    unsafe {
        abort(PANIC_CODE);
        core::hint::unreachable_unchecked()
    }
}
