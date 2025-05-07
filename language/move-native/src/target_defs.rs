// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pub use impls::*;

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
