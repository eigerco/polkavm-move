#![cfg_attr(feature = "polkavm", no_std)]

#[cfg(feature = "polkavm")]
pub mod guest;
#[cfg(feature = "host")]
pub mod host;
pub mod types;

// abort codes used by native lib
pub const PANIC_CODE: u64 = 0xca11;
pub const ALLOC_CODE: u64 = 0xdead;
