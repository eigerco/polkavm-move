use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // panic inside polkavm program should call abort or smth similar
    loop {}
}
