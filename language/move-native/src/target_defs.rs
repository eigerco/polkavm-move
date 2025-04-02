// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pub use impls::*;

#[cfg(not(feature = "polkavm"))]
mod impls {
    // Move addresses are 16 bytes by default, but can be made 20 or 32 at compile time.
    pub const ACCOUNT_ADDRESS_LENGTH: usize = 16;

    pub fn print_string(_s: &str) {
        todo!()
    }

    pub fn print_stack_trace() {
        todo!()
    }

    pub fn abort(_code: u64) -> ! {
        todo!()
    }
}

#[cfg(feature = "polkavm")]
mod impls {
    // Solana pubkeys are 32 bytes.
    // Move addresses are 16 bytes by default, but can be made 20 or 32 at compile time.
    pub const ACCOUNT_ADDRESS_LENGTH: usize = 32;

    pub fn print_string(s: &str) {
        unsafe {
            syscalls::sol_log_(s.as_ptr(), s.len() as u64);
        }
    }

    pub fn print_stack_trace() {
        todo!()
    }

    pub fn abort(code: u64) -> ! {
        unsafe {
            syscalls::sol_log_64_(code, code, code, code, code);
            syscalls::abort()
        }
    }

    // NB: not using the "static-syscalls" sbf feature
    mod syscalls {
        extern "C" {
            pub fn abort() -> !;
            pub fn sol_log_(msg: *const u8, len: u64);
            pub fn sol_log_64_(_: u64, _: u64, _: u64, _: u64, _: u64);
        }
    }

    mod globals {
        use alloc::format;
        use emballoc::Allocator;
        const PANIC_ABORT_CODE: u64 = 101;

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            super::print_string(&format!("{}", info));
            super::abort(PANIC_ABORT_CODE);
        }

        const HEAP_SIZE: usize = 60 * 1024 * 1024;

        #[global_allocator]
        static ALLOCATOR: Allocator<HEAP_SIZE> = Allocator::new();
    }
}
