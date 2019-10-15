/// Succinct Non-interactive Proofs of Exponentiation.

use runtime_io::blake2_256;
use codec::{Encode};
use crate::subroutines;
use super::U2048;

/// Generates proof of exponentiation that u^x = w (based on Wesolowski). Protocol is only useful
/// if the verifier can compute the residue r = x mod l faster than computing u^x.
/// To investigate: Security parameter should be larger than that of accumulator elements.
pub fn poe(u: U2048, x: U2048, w: U2048) -> U2048 {
    let l = subroutines::hash_to_prime(&(u, x, w).encode());
    let q = x / l;
    return subroutines::mod_exp(u, q, U2048::from_dec_str(super::MODULUS).unwrap());
}

/// Verifies proof of exponentiation.
pub fn verify_poe(u: U2048, x: U2048, w: U2048, Q: U2048) -> bool {
    let l = subroutines::hash_to_prime(&(u, x, w).encode());
    let r = x % l;
    let lhs = subroutines::mul_mod(subroutines::mod_exp(Q, l, U2048::from_dec_str(super::MODULUS).unwrap()), subroutines::mod_exp(u, r, U2048::from_dec_str(super::MODULUS).unwrap()),
                                   U2048::from_dec_str(super::MODULUS).unwrap());
    return lhs == w;
}

/// Generates proof of knowledge of exponentiation that u^x = w. We will assume that the generator
/// g = 2 is a group element of unknown order.
/// To investigate: Security parameter should be larger than that of accumulator elements.
pub fn poke(u: U2048, x: U2048, w: U2048) -> (U2048, U2048, U2048) {
    let z = subroutines::mod_exp(U2048::from(2), x, U2048::from_dec_str(super::MODULUS).unwrap());
    let l = subroutines::hash_to_prime(&(u, w, z).encode());
    let alpha = U2048::from_little_endian(&blake2_256(&(u, w, z, l).encode()));
    let q = x / l;
    let r = x % l;
    let Q = subroutines::mod_exp(subroutines::mul_mod(u, subroutines::mod_exp(U2048::from(2), alpha, U2048::from_dec_str(super::MODULUS).unwrap()),
                                                      U2048::from_dec_str(super::MODULUS).unwrap()), q, U2048::from_dec_str(super::MODULUS).unwrap());
    let pi = (z, Q, r);
    return pi;
}

/// Verifies proof of knowledge of exponentiation.
pub fn verify_poke(u: U2048, w: U2048, z: U2048, Q: U2048, r: U2048) -> bool {
    let l = subroutines::hash_to_prime(&(u, w, z).encode());
    let alpha = U2048::from_little_endian(&blake2_256(&(u, w, z, l).encode()));
    let lhs = subroutines::mul_mod(subroutines::mod_exp(Q, l, U2048::from_dec_str(super::MODULUS).unwrap()),
                                   subroutines::mod_exp(subroutines::mul_mod(u, subroutines::mod_exp(U2048::from(2), alpha, U2048::from_dec_str(super::MODULUS).unwrap()),
                                                                             U2048::from_dec_str(super::MODULUS).unwrap()), r, U2048::from_dec_str(super::MODULUS).unwrap()), U2048::from_dec_str(super::MODULUS).unwrap());
    let rhs = subroutines::mul_mod(w, subroutines::mod_exp(z, alpha, U2048::from_dec_str(super::MODULUS).unwrap()), U2048::from_dec_str(super::MODULUS).unwrap());
    return lhs == rhs;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poe() {
        let mut proof = poe(U2048::from(2), U2048::from(6), U2048::from(12));
        assert_eq!(verify_poe(U2048::from(2), U2048::from(6), U2048::from(12), proof), true);

        proof = poe(U2048::from(121314), U2048::from(14123), U2048::from(6));
        assert_eq!(verify_poe(U2048::from(121314), U2048::from(14123), U2048::from(6), proof), true);

        // Fake proof
        assert_eq!(verify_poe(U2048::from(2), U2048::from(6), U2048::from(12), U2048::from(3)), false);
        assert_eq!(verify_poe(U2048::from(4), U2048::from(12), U2048::from(7), U2048::from(1)), false);
    }

    #[test]
    fn test_poke() {
        let (z, Q, r) = poke(U2048::from(2), U2048::from(6), U2048::from(12));
        assert_eq!(verify_poke(U2048::from(2), U2048::from(12), z, Q, r), true);

        let (z, Q, r) = poke(U2048::from(121314), U2048::from(14123), U2048::from(6));
        assert_eq!(verify_poke(U2048::from(121314), U2048::from(6), z, Q, r), true);

        // Fake proof
        assert_eq!(verify_poke(U2048::from(121314), U2048::from(7), z, Q, r), false);
        assert_eq!(verify_poke(U2048::from(2), U2048::from(12), U2048::from(4), U2048::from(1), U2048::from(2)), false);
    }

}