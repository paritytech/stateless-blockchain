/// Integer Subroutines for Accumulator Functions.

use core::convert::TryFrom;
use runtime_io::blake2_256;
use rstd::prelude::Vec;
use super::U2048;

/// Implements fast modular exponentiation. Algorithm inspired by https://github.com/pwoolcoc/mod_exp-rs/blob/master/src/lib.rs
/// NOTE: Overflow error occurs when size of result exceeds U2048.
pub fn mod_exp(mut base: U2048, mut exp: U2048, modulus: U2048) -> U2048 {
    let mut result: U2048 = U2048::from(1);
    base = base % modulus;
    while exp > U2048::from(0) {
        if exp % U2048::from(2) == U2048::from(1) {
            result = mul_mod(result, base, modulus);
        }

        if exp == U2048::from(1) {
            return result;
        }

        exp = exp >> U2048::from(1);
        base = mul_mod(base, base, modulus);
    }
    return result;
}

/// Defines the multiplication operation for the group. Idea courtesy of:
/// https://www.geeksforgeeks.org/how-to-avoid-overflow-in-modular-multiplication/
pub fn mul_mod(mut a: U2048, mut b: U2048, modulus: U2048) -> U2048 {
    let mut result = U2048::from(0);
    a = a % modulus;
    while b > U2048::from(0) {
        if b % U2048::from(2) == U2048::from(1) {
            result = (result + a) % modulus;
        }

        a = (a * U2048::from(2)) % modulus;
        b /= U2048::from(2);
    }
    return result % modulus;
}

/// Given the xth root of g and yth root of g, finds the xyth root. If the roots are invalid or
/// x and y are not coprime, None is returned. Otherwise, the function performs relevant modular
/// inverse operations on the Bezout coefficients (returned as signed integers) and finds the xyth root.
pub fn shamir_trick(mut xth_root: U2048, mut yth_root: U2048, x: U2048, y: U2048) -> Option<U2048> {
    // Check if the inputs are valid.
    if mod_exp(xth_root, x, U2048::from_dec_str(super::MODULUS).unwrap())
        != mod_exp(yth_root, y, U2048::from_dec_str(super::MODULUS).unwrap()) {
        return None;
    }

    match bezout(x, y) {
        None => {
            return None;
        },
        Some(coefficients) => {
            // Receive coefficient as signed integers.
            let (mut a, mut b) = coefficients;

            // Calculate relevant modular inverses to allow for exponentiation later on.
            if b < 0 {
                xth_root = mod_inverse(xth_root);
                b = -b;
            }

            if a < 0 {
                yth_root = mod_inverse(yth_root);
                a = -a
            }

            let combined_root: U2048 = (mod_exp(xth_root, U2048::from(b), U2048::from_dec_str(super::MODULUS).unwrap())
                * mod_exp(yth_root, U2048::from(a), U2048::from_dec_str(super::MODULUS).unwrap())) % U2048::from_dec_str(super::MODULUS).unwrap();
            return Some(combined_root);
        },
    }
}

/// Computes the modular multiplicative inverse.
/// NOTE: Does not check if gcd != 1(none exists if so).
pub fn mod_inverse(elem: U2048) -> U2048 {
    let (_, x, _) = extended_gcd(elem, U2048::from_dec_str(super::MODULUS).unwrap());

    // Accommodate for negative x coefficient
    if x < 0 {
        // Since we're assuming that the U2048::from(super::MODULUS) will always be larger than than coefficient in
        // absolute value, we simply subtract x from the U2048::from(super::MODULUS) to get a positive value mod N.
        let pos_x = U2048::from_dec_str(super::MODULUS).unwrap() - U2048::from(x*-1);
        return pos_x % U2048::from_dec_str(super::MODULUS).unwrap();
    }
    return U2048::from(x) % U2048::from_dec_str(super::MODULUS).unwrap();
}

/// Returns Bezout coefficients as *signed* integers (since they may be negative).
/// Acts as a wrapper for extended_gcd.
pub fn bezout(a: U2048, b: U2048) -> Option<(i128, i128)> {
    let (gcd, x, y) = extended_gcd(a, b);
    // Check if a and b are coprime
    if gcd != U2048::from(1) {
        return None;
    }
    else {
        return Some((x, y));
    }
}

/// Implements the Extended Euclidean Algorithm (https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm).
/// NOTE: I assume that the absolute value of the Bezout coefficients are at most 64 bits(hence 128 bit
/// signed integers). Otherwise, the function panics during the unwrap.
/// Reference: https://math.stackexchange.com/questions/670405/does-the-extended-euclidean-algorithm-always-return-the-smallest-coefficients-of
pub fn extended_gcd(a: U2048, b: U2048) -> (U2048, i128, i128) {
    let (mut s, mut old_s): (i128, i128) = (0, 1);
    let (mut t, mut old_t): (i128, i128) = (1, 0);
    let (mut r, mut old_r): (U2048, U2048) = (b, a);

    while r != U2048::from(0) {
        let quotient = old_r / r;
        let new_r = old_r - quotient * r;
        old_r = r;
        r = new_r;

        let new_s = old_s - i128::try_from(quotient).unwrap() * s;
        old_s = s;
        s = new_s;

        let new_t = old_t - i128::try_from(quotient).unwrap() * t;
        old_t = t;
        t = new_t;
    }
    return (old_r, old_s, old_t);
}

/// Continuously hashes the input until the result is prime. Assumes input values are transcoded in
/// little endian(uses parity-scale-codec).
/// Consideration: Currently unclear about the impact of Lambda on the security of the scheme.
pub fn hash_to_prime(elem: &[u8]) -> U2048 {
    let mut hash = blake2_256(elem);
    let mut result = U2048::from_little_endian(&hash) % U2048::from(super::LAMBDA);

    // While the resulting hash is not a prime, keep trying
    while !miller_rabin(result) {
        hash = blake2_256(&hash);
        result = U2048::from_little_endian(&hash) % U2048::from(super::LAMBDA);
    }

    return result;
}

/// Implements a deterministic variant of the Miller-Rabin primality test for u64/u32 integers based
/// on the algorithm from the following link: https://en.wikipedia.org/wiki/Miller–Rabin_primality_test
/// Complexity of the algorithm is O((log n)^4) in soft-O notation.
pub fn miller_rabin(n: U2048) -> bool {
    // Find r and d such that 2^r * d + 1 = n
    let r = (n-U2048::from(1)).trailing_zeros();
    let d = (n-U2048::from(1)) >> U2048::from(r);

    // See https://stackoverflow.com/questions/7594307/simple-deterministic-primality-testing-for-small-numbers
    //let bases = [2,3,5,7,11,13,17]; // Deterministic for 64 bit integers
    let bases = [2, 7, 61];  // Deterministic for 32 bit integers

    'outer: for &a in bases.iter() {
        // Annoying edge case to make sure a is within [2, n-2] for small n
        if n-U2048::from(2) < U2048::from(a) { break; }

        let mut x = mod_exp(U2048::from(a), d, n);

        if x == U2048::from(1) || x == (n-U2048::from(1)) {
            continue;
        }
        for _ in 1..r {
            x = mod_exp(x, U2048::from(2), n);
            if x == (n-U2048::from(1)) {
                continue 'outer;
            }
        }
        return false;
    }
    return true;
}

/// Given an element g and a set of elements x, computes the xith root of g^x for each element
/// in the set. Runs in O(n log(n)).
pub fn root_factor(g: U2048, elems: &[U2048]) -> Vec<U2048> {
    if elems.len() == 1 {
        let mut ret = Vec::new();
        ret.push(g);
        return ret;
    }

    let n_prime = elems.len()/2;

    let mut g_left = g;
    for i in 0..n_prime {
        g_left = mod_exp(g_left, elems[i], U2048::from_dec_str(super::MODULUS).unwrap());
    }

    let mut g_right = g;
    for i in n_prime..elems.len() {
        g_right = mod_exp(g_right, elems[i], U2048::from_dec_str(super::MODULUS).unwrap());
    }

    let mut left = root_factor(g_right, &elems[0..n_prime]);
    let mut right = root_factor(g_left, &elems[n_prime..]);
    left.append(&mut right);
    return left;
}

/// Short helper function that calculates the product of elements in the vector.
pub fn prime_product(elems: &[U2048]) -> U2048 {
    let mut result: U2048 = U2048::from(1);
    for &elem in elems.iter() {
        result *= elem;
    }
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MODULUS;
    use codec::{Encode};

    #[test]
    fn test_mul_mod() {
        assert_eq!(mul_mod(U2048::from(121), U2048::from(12314), U2048::from_dec_str(MODULUS).unwrap()),
                   U2048::from(12));
        assert_eq!(mul_mod(U2048::from(128), U2048::from(23), U2048::from(75)),
                   U2048::from(19));
    }

    #[test]
    fn test_mod_exp() {
        assert_eq!(mod_exp(U2048::from(2), U2048::from(7), U2048::from_dec_str(MODULUS).unwrap()), U2048::from(11));
        assert_eq!(mod_exp(U2048::from(7), U2048::from(15), U2048::from_dec_str(MODULUS).unwrap()), U2048::from(5));
    }

    #[test]
    fn test_extended_gcd() {
        assert_eq!(extended_gcd(U2048::from(180), U2048::from(150)), (U2048::from(30), 1, -1));
        assert_eq!(extended_gcd(U2048::from(13), U2048::from(17)), (U2048::from(1), 4, -3));
    }

    #[test]
    fn test_bezout() {
        assert_eq!(bezout(U2048::from(4), U2048::from(10)), None);
        assert_eq!(bezout(U2048::from(3434), U2048::from(2423)), Some((-997, 1413)));
    }

    #[test]
    fn test_shamir_trick() {
        assert_eq!(shamir_trick(U2048::from(11), U2048::from(6), U2048::from(7), U2048::from(5)), Some(U2048::from(7)));
        assert_eq!(shamir_trick(U2048::from(11), U2048::from(7), U2048::from(7), U2048::from(11),), Some(U2048::from(6)));
        assert_eq!(shamir_trick(U2048::from(6), U2048::from(7), U2048::from(5), U2048::from(11)), Some(U2048::from(11)));
        assert_eq!(shamir_trick(U2048::from(12), U2048::from(7), U2048::from(7), U2048::from(11)), None);
    }

    #[test]
    fn test_mod_inverse() {
        assert_eq!(mod_inverse(U2048::from(9)), U2048::from(3));
        assert_eq!(mod_inverse(U2048::from(6)), U2048::from(11));
    }

    #[test]
    fn test_miller_rabin() {
        assert_eq!(miller_rabin(U2048::from(5)), true);
        assert_eq!(miller_rabin(U2048::from(7)), true);
        assert_eq!(miller_rabin(U2048::from(241)), true);
        assert_eq!(miller_rabin(U2048::from(7919)), true);
        assert_eq!(miller_rabin(U2048::from(48131)), true);
        assert_eq!(miller_rabin(U2048::from(76463)), true);
        assert_eq!(miller_rabin(U2048::from(4222234741u64)), true);
        assert_eq!(miller_rabin(U2048::from(187278659180417234321u128)), true);

        assert_eq!(miller_rabin(U2048::from(21)), false);
        assert_eq!(miller_rabin(U2048::from(87)), false);
        assert_eq!(miller_rabin(U2048::from(155)), false);
        assert_eq!(miller_rabin(U2048::from(9167)), false);
        assert_eq!(miller_rabin(U2048::from(102398)), false);
        assert_eq!(miller_rabin(U2048::from(801435)), false);
        assert_eq!(miller_rabin(U2048::from(51456119958243u128)), false);
    }

    #[test]
    fn test_hash_to_prime() {
        //assert_eq!(hash_to_prime(&[7, 10]), U2048::from(...));
        // Key values checked: 0, 1, 2
    }

    #[test]
    fn test_root_factor() {
        assert_eq!(root_factor(U2048::from(2), &vec![U2048::from(3), U2048::from(5), U2048::from(7), U2048::from(11)]),
                   vec![U2048::from(2), U2048::from(8), U2048::from(5), U2048::from(5)]);
    }

    #[test]
    fn test_prime_product() {
        let elems = vec![U2048::from(2), U2048::from(3), U2048::from(4)];
        assert_eq!(prime_product(&elems), U2048::from(24));
    }


}