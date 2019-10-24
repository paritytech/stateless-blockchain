#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use rstd::vec::Vec;
extern crate alloc;
use alloc::borrow::ToOwned;
#[macro_use]
extern crate uint;

pub mod subroutines;
pub mod proofs;
pub mod witnesses;

/// Construct BigInt type.
construct_uint! {
    #[derive(Encode, Decode)]
	pub struct U2048(32);
}

/// Defines the RSA group. Currently uses a temporary value for testing.
/// RSA 100: "1522605027922533360535618378132637429718068114961380688657908494580122963258952897654000350692006139"
/// NOTE: Tests only work for MODULUS = 13
pub const MODULUS: &str = "13";

/// Security parameter that represents the size of elements added to the accumulator.
pub const LAMBDA: u32 = u32::max_value();

