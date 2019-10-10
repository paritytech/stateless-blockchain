#![cfg_attr(not(feature = "std"), no_std)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn foo(x: i32, y: i32) -> i32 {
    x + y
}

pub mod subroutines;
pub mod proofs;
pub mod witnesses;

// Defines the RSA group. Currently uses a temporary value for testing.
pub const MODULUS: u64 = 13;

// Security parameter that represents the size of elements added to the accumulator.
pub const LAMBDA: u64 = u64::max_value()/2;