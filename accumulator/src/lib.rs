#![cfg_attr(not(feature = "std"), no_std)]

pub mod subroutines;
pub mod proofs;
pub mod witnesses;

pub const MODULUS: u64 = 13;
pub const LAMBDA: u64 = u64::max_value();