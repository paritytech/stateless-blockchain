/// Succinct non-interactive proofs of exponentiation.

use primitive_types::U256;
use runtime_io::blake2_256;
use codec::{Encode};
use crate::subroutines;

/// Generate proof of exponentiation for u^x = w (based on Wesolowski). Protocol is only useful
/// if the verifier can compute the reside r = x mod l faster than computing u^x.
/// To investigate: Security parameter should be larger than that for accumulator elements.
pub fn poe(u: U256, x: U256, w: U256) -> U256 {
    let l = subroutines::hash_to_prime(&(u, x, w).encode());
    let q = x / l;
    return subroutines::mod_exp(u, q, U256::from(super::MODULUS));
}

/// Verify proof of exponentiation.
pub fn verify_poe(u: U256, x: U256, w: U256, Q: U256) -> bool {
    let l = subroutines::hash_to_prime(&(u, x, w).encode());
    let r = x % l;
    let lhs = subroutines::mul_mod(subroutines::mod_exp(Q, l, U256::from(super::MODULUS)), subroutines::mod_exp(u, r, U256::from(super::MODULUS)),
                                   U256::from(super::MODULUS));
    return lhs == w;
}

/// Generate proof of knowledge of exponentiation for u^x = w. We will assume that the generator
/// g = 2 is a group element of unknown order.
/// To investigate: Security parameter should be larger than that for accumulator elements.
pub fn poke(u: U256, x: U256, w: U256) -> (U256, U256, U256) {
    let z = subroutines::mod_exp(U256::from(2), x, U256::from(super::MODULUS));
    let l = subroutines::hash_to_prime(&(u, w, z).encode());
    let alpha = U256::from(blake2_256(&(u, w, z, l).encode()));
    let q = x / l;
    let r = x % l;
    let Q = subroutines::mod_exp(subroutines::mul_mod(u, subroutines::mod_exp(U256::from(2), alpha, U256::from(super::MODULUS)),
                                                      U256::from(super::MODULUS)), q, U256::from(super::MODULUS));
    let pi = (z, Q, r);
    return pi;
}

/// Verify proof of knowledge of exponentiation.
pub fn verify_poke(u: U256, w: U256, z: U256, Q: U256, r: U256) -> bool {
    let l = subroutines::hash_to_prime(&(u, w, z).encode());
    let alpha = U256::from(blake2_256(&(u, w, z, l).encode()));
    let lhs = subroutines::mul_mod(subroutines::mod_exp(Q, l, U256::from(super::MODULUS)),
                                   subroutines::mod_exp(subroutines::mul_mod(u, subroutines::mod_exp(U256::from(2), alpha, U256::from(super::MODULUS)),
                                                                             U256::from(super::MODULUS)), r, U256::from(super::MODULUS)), U256::from(super::MODULUS));
    let rhs = subroutines::mul_mod(w, subroutines::mod_exp(z, alpha, U256::from(super::MODULUS)), U256::from(super::MODULUS));
    return lhs == rhs;
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODULUS: u64 = 13;

    #[test]
    fn test_poe() {
        let mut proof = poe(U256::from(2), U256::from(6), U256::from(12));
        assert_eq!(verify_poe(U256::from(2), U256::from(6), U256::from(12), proof), true);

        proof = poe(U256::from(121314), U256::from(14123), U256::from(6));
        assert_eq!(verify_poe(U256::from(121314), U256::from(14123), U256::from(6), proof), true);

        // Fake proof
        assert_eq!(verify_poe(U256::from(2), U256::from(6), U256::from(12), U256::from(3)), false);
        assert_eq!(verify_poe(U256::from(4), U256::from(12), U256::from(7), U256::from(1)), false);
    }

    #[test]
    fn test_poke() {
        let (z, Q, r) = poke(U256::from(2), U256::from(6), U256::from(12));
        assert_eq!(verify_poke(U256::from(2), U256::from(12), z, Q, r), true);

        let (z, Q, r) = poke(U256::from(121314), U256::from(14123), U256::from(6));
        assert_eq!(verify_poke(U256::from(121314), U256::from(6), z, Q, r), true);

        // Fake proof
        assert_eq!(verify_poke(U256::from(121314), U256::from(7), z, Q, r), false);
        assert_eq!(verify_poke(U256::from(2), U256::from(12), U256::from(4), U256::from(1), U256::from(2)), false);
    }
    
}