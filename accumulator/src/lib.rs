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

/// Defines the RSA group. Arbitrary set at MODULUS = 13 for testing.
/// Example (insecure) modulus -> RSA 100: "1522605027922533360535618378132637429718068114961380688657908494580122963258952897654000350692006139"
pub const MODULUS: &str = "13";

/// Security parameter that represents the size of elements added to the accumulator.
pub const LAMBDA: u32 = u32::max_value();

/// A witness can either be a membership witness or a non-membership witness.
#[derive(Clone, Copy)]
pub enum Witness {
    MemWit(U2048),
    NonMemWit((U2048, bool, U2048)),
}

/// A Bezout coefficient pair. This is a temporary workaround due to the lack of support for
/// signed BigInts(coefficients can be negative).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BezoutPair {
    coefficient_a: U2048,
    coefficient_b: U2048,
    sign_a: bool,
    sign_b: bool,
}

/// Add a single element to an accumulator.
pub fn add(state: U2048, elem: U2048) -> U2048 {
    return subroutines::mod_exp(state, elem, U2048::from_dec_str(MODULUS).unwrap());
}

/// Delete an element from the accumulator given a membership proof.
pub fn delete(state: U2048, elem: U2048, proof: U2048) -> Option<U2048> {
    if subroutines::mod_exp(proof, elem, U2048::from_dec_str(MODULUS).unwrap()) == state {
        return Some(proof);
    }
    return None;
}

/// Aggregates a set of accumulator elements + witnesses and batch deletes them from the accumulator.
/// Returns the state after deletion, the product of the deleted elements, and a proof of exponentiation.
pub fn batch_delete(state: U2048, elems: &Vec<(U2048, U2048)>) -> (U2048, U2048, U2048) {
    let (mut x_agg, mut new_state) = elems[0];
    for i in 1..elems.len() {
        let (x, witness) = elems[i];
        new_state = subroutines::shamir_trick(new_state, witness, x_agg, x).unwrap();
        x_agg *= x;
    }
    let proof = proofs::poe(new_state, x_agg, state);
    return (new_state, x_agg, proof);
}

/// Aggregates a set of accumulator elements + witnesses and batch adds them to the accumulator.
/// Returns the state after addition, the product of the added elements, and a proof of exponentiation.
pub fn batch_add(state: U2048, elems: &Vec<U2048>) -> (U2048, U2048, U2048) {
    let mut x_agg = U2048::from(1);
    for i in 0..elems.len() {
        x_agg *= elems[i];
    }

    let new_state = subroutines::mod_exp(state, x_agg, U2048::from_dec_str(MODULUS).unwrap());
    let proof = proofs::poe(state, x_agg, new_state);
    return (new_state, x_agg, proof);
}
