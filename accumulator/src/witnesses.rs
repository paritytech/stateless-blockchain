/// Membership Witness Management

use crate::subroutines;
use crate::proofs;
use rstd::prelude::Vec;
use super::U2048;

/// Given an old state, the product of a set of elements that have been added, and a single element from that
/// set, returns the witness for that element.
/// NOTE: "old_state" represents the state *before* the elements are added.
/// This function will likely be used by an online user.
pub fn mem_wit_create(old_state: U2048, agg: U2048, elem: U2048) -> Option<U2048> {
    if agg % elem != U2048::from(0) {
        return None;
    }
    let quotient = agg / elem;
    return Some(subroutines::mod_exp(old_state, quotient, U2048::from_dec_str(super::MODULUS).unwrap()));
}

/// Verify the witness of an element.
pub fn verify_mem_wit(state: U2048, witness: U2048, elem: U2048) -> bool {
    let result = subroutines::mod_exp(witness, elem, U2048::from_dec_str(super::MODULUS).unwrap());
    return result == state;
}

/// Updates a membership witness based on untracked additions and deletions. Algorithm is based on
/// section 3.2 of the paper titled "Dynamic Accumulators and Applications to Efficient Revocation of
/// Anonymous Credentials". Note that "additions" represent the product of the added elements
/// and "deletions" represents the product of the deleted elements.
/// NOTE: Does not do any error checking on unwrap.
pub fn update_mem_wit(elem: U2048, mut witness: U2048, new_state: U2048, additions: U2048, deletions: U2048) -> U2048 {
    // Handle added elems
    witness = subroutines::mod_exp(witness, additions, U2048::from_dec_str(super::MODULUS).unwrap());

    // Handle deleted elems
    witness = subroutines::shamir_trick(witness, new_state, elem, deletions).unwrap();
    return witness;
}


/// Takes two elements + membership witnesses and returns the aggregated witness and aggregated proof.
/// NOTE: Does very little error checking (Ex: Does not do any error checking on unwrap).
pub fn agg_mem_wit(state: U2048, witness_x: U2048, witness_y: U2048, x: U2048, y: U2048) -> (U2048, U2048) {
    let aggregated = subroutines::shamir_trick(witness_x, witness_y, x, y).unwrap();
    let proof = proofs::poe(aggregated, subroutines::mul_mod(x, y, U2048::from_dec_str(super::MODULUS).unwrap()), state);
    return (aggregated, proof);
}

/// Verifies that a membership witness + proof for a set of accumulator elements are valid. Acts as a
/// wrapper for the proof of exponentiation verifier.
pub fn verify_agg_mem_wit(state: U2048, agg_elems: U2048, witness: U2048, proof: U2048) -> bool {
    return proofs::verify_poe(witness, agg_elems, state, proof);
}

/// Creates individual membership witnesses. Acts as a wrapper for the RootFactor subroutine.
/// NOTE: "old_state" represents the state *before* the elements are added.
/// This function will most likely be used by a service provider.
pub fn create_all_mem_wit(old_state: U2048, new_elems: &[U2048]) -> Vec<U2048> {
    return subroutines::root_factor(old_state, new_elems);
}

/// Below contains all of the non-membership witness functions required for vector commitments.

/// Creates a non-membership witness relative to some previous state. The current state should equal "old_state"
/// raised to the "agg_elems" power(represents product of added elements). The second value of the
/// tuple is the sign of the first value since the Bezout coefficient may be negative.
/// NOTE: Function assumes that "elem" is not contained in "agg_elems"
pub fn non_mem_wit_create(mut old_state: U2048, agg_elems: U2048, elem: U2048) -> (U2048, bool, U2048) {
    let pair = subroutines::bezout(agg_elems, elem).unwrap();

    if pair.sign_b {
        old_state = subroutines::mod_inverse(old_state);
    }

    let B = subroutines::mod_exp(old_state, U2048::from(pair.coefficient_b), U2048::from_dec_str(super::MODULUS).unwrap());
    return (pair.coefficient_a, pair.sign_a, B);
}

/// Verifies a non-membership witness. "state" represents the current state.
pub fn verify_non_mem_wit(old_state: U2048, mut state: U2048, witness: (U2048, bool, U2048), elem: U2048) -> bool {
    let (a, sign_a, B) = witness;

    if sign_a {
        state = subroutines::mod_inverse(state);
    }

    let exp_1 = subroutines::mod_exp(state, U2048::from(a), U2048::from_dec_str(super::MODULUS).unwrap());
    let exp_2 = subroutines::mod_exp(B, elem, U2048::from_dec_str(super::MODULUS).unwrap());

    return subroutines::mul_mod(exp_1, exp_2, U2048::from_dec_str(super::MODULUS).unwrap()) == old_state;
}

/// NEED TO IMPLEMENT AND TEST
pub fn update_non_mem_wit() {}

/// OPTIONAL FUNCTION.
/// Given the current state, the previous state, the product of the added elements, and a subset of
/// those elements, creates a witness for thoise elements.
pub fn mem_wit_create_star(cur_state: U2048, old_state: U2048, agg: U2048, new_elems: Vec<U2048>) -> (U2048, U2048) {
    let product = subroutines::prime_product(&new_elems);
    let witness = mem_wit_create(old_state, agg, product).unwrap();
    let proof = proofs::poe(witness, product, cur_state);
    return (witness, proof);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch_add;

    #[test]
    fn test_mem_wit_create() {
        assert_eq!(mem_wit_create(U2048::from(2), U2048::from(1155), U2048::from(3)).unwrap(), U2048::from(2));
        assert_eq!(mem_wit_create(U2048::from(2), U2048::from(1155), U2048::from(5)).unwrap(), U2048::from(8));
        assert_eq!(mem_wit_create(U2048::from(2), U2048::from(1155), U2048::from(7)).unwrap(), U2048::from(5));
        assert_eq!(mem_wit_create(U2048::from(2), U2048::from(1155),U2048::from(11)).unwrap(), U2048::from(5));
        assert_eq!(mem_wit_create(U2048::from(2), U2048::from(1155),U2048::from(4)).is_none(), true);
    }

    #[test]
    fn test_agg_mem_wit() {
        let (aggregate, proof) = agg_mem_wit(U2048::from(8), U2048::from(6), U2048::from(8),U2048::from(3), U2048::from(5));
        assert_eq!(aggregate, U2048::from(2));
        assert_eq!(verify_agg_mem_wit(U2048::from(8), U2048::from(15), aggregate, proof), true);
    }

    #[test]
    fn test_verify_agg_mem_wit() {
        let proof = proofs::poe(U2048::from(2), U2048::from(12123), U2048::from(8));
        assert_eq!(verify_agg_mem_wit(U2048::from(8), U2048::from(12123), U2048::from(2), proof), true);
        assert_eq!(verify_agg_mem_wit(U2048::from(7), U2048::from(12123), U2048::from(2), proof), false);
    }

    #[test]
    fn test_update_mem_wit() {
        let deletions = U2048::from(15);
        let additions = U2048::from(77);

        let elem = U2048::from(12131);
        let mut witness = U2048::from(8);
        let new_state = U2048::from(11);

        assert_eq!(update_mem_wit(elem, witness, new_state, additions, deletions), U2048::from(6));
    }

    #[test]
    fn test_create_all_mem_wit() {
        assert_eq!(create_all_mem_wit(U2048::from(2), &vec![U2048::from(3), U2048::from(5), U2048::from(7), U2048::from(11)]),
                   vec![U2048::from(2), U2048::from(8), U2048::from(5), U2048::from(5)]);
    }

    // Begin tests for non-membership witnesses.

    #[test]
    fn test_non_mem_wit() {
        let (a, sign_a, B) = non_mem_wit_create(U2048::from(2), U2048::from(105), U2048::from(11));

        assert_eq!(verify_non_mem_wit(U2048::from(2), U2048::from(5), (a, sign_a, B), U2048::from(11)), true);
        assert_eq!(verify_non_mem_wit(U2048::from(2), U2048::from(6), (a, sign_a, B), U2048::from(11)), false);
        assert_eq!(verify_non_mem_wit(U2048::from(2), U2048::from(5), (a, sign_a, B), U2048::from(5)), false);
    }

    #[test]
    fn test_mem_wit_create_star() {
        let old_state = U2048::from(2);
        let new_elems = vec![U2048::from(3), U2048::from(5), U2048::from(7), U2048::from(11), U2048::from(17)];
        let (new_state, agg, _) = batch_add(old_state, &new_elems);

        let subset = vec![U2048::from(5), U2048::from(11), U2048::from(17)];
        let subset_product = subroutines::prime_product(&subset);
        let (witness, proof) = mem_wit_create_star(new_state, old_state, agg, subset);

        assert_eq!(witness, U2048::from(5));
        assert_eq!(proofs::verify_poe(witness, subset_product, new_state, proof), true);
    }


}