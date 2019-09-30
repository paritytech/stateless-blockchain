/// Membership Witness Management
/// WORK IN PROGRESS: EVERYTHING HERE IS INCOMPLETE AND UNTESTED

use primitive_types::U256;
//use runtime_io::blake2_256;
//use codec::{Encode};
use crate::subroutines;
use crate::proofs;
use rstd::prelude::Vec;

// Modify this include PoE
pub fn verify_mem_wit(state: U256, elem: U256, witness: U256) -> bool {
    return subroutines::mod_exp(witness, elem, U256::from(super::MODULUS)) == state;
}

// Based on section 4.2 of LLV paper
// Need to determine whether can use aggregated values
// Does not do any error checking on unwrap
pub fn update_mem_wit(elem: U256, mut witness: U256, new_state: U256, deletions: &Vec<U256>,
                      additions: &Vec<U256>) -> U256 {
    // Handle deleted elems
    for &deleted_elem in deletions.iter() {
        if elem != deleted_elem {
            let (a, b) = subroutines::bezout(elem, deleted_elem).unwrap();
            witness = subroutines::mul_mod(subroutines::mod_exp(witness, U256::from(b), U256::from(super::MODULUS)),
                                           subroutines::mod_exp(new_state, U256::from(a), U256::from(super::MODULUS)), U256::from(super::MODULUS));
        }
    }

    // Handle added elems
    for &added_elem in additions.iter() {
        witness = subroutines::mod_exp(witness, added_elem, U256::from(super::MODULUS));
    }
    return witness;
}

// Does not do any error checking on unwrap
pub fn agg_mem_wit(state: U256, witness_x: U256, witness_y: U256, x: U256, y: U256) -> (U256, U256) {
    let aggregated = subroutines::shamir_trick(witness_x, witness_y, x, y).unwrap();
    let proof = proofs::poe(aggregated, subroutines::mul_mod(x, y, U256::from(super::MODULUS)), state);
    return (aggregated, proof);
}

pub fn create_all_mem_wit() {}

#[cfg(test)]
mod tests {
    use super::*;

    const MODULUS: u64 = 13;

    #[test]
    fn test_update_mem_wit() {

    }

}