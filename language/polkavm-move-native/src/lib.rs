#![cfg_attr(feature = "polkavm", no_std)]

pub mod conv;
#[cfg(feature = "polkavm")]
pub mod guest;
#[cfg(feature = "host")]
pub mod host;
pub mod structs;
pub mod types;
pub mod vector;

// abort codes used by native lib
pub const PANIC_CODE: u64 = 0xdead;
pub const ALLOC_CODE: u64 = 0xca11;
