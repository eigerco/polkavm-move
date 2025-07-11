#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    use super::imports::terminate;
    use crate::PANIC_CODE;
    unsafe {
        let mut beneficiary = [0u8; 20];
        beneficiary[0] = PANIC_CODE as u8;
        terminate(beneficiary.as_ptr() as *const [u8; 20]);
        core::hint::unreachable_unchecked()
    }
}
