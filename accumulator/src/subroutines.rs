/// Integer subroutines for accumulator functions.

use primitive_types::U256;
use core::convert::TryFrom;
use runtime_io::blake2_256;

/// Implement fast modular exponentiation. Algorithm inspired by https://github.com/pwoolcoc/mod_exp-rs/blob/master/src/lib.rs
/// NOTE: Overflow error occurs when size of result exceeds U256.
pub fn mod_exp(mut base: U256, mut exp: U256, modulus: U256) -> U256 {
    let mut result: U256 = U256::from(1);
    base = base % modulus;
    while exp > U256::from(0) {
        if exp % U256::from(2) == U256::from(1) {
            result = mul_mod(result, base, modulus);
        }

        if exp == U256::from(1) {
            return result;
        }

        exp = exp >> U256::from(1);
        base = mul_mod(base, base, modulus);
    }
    return result;
}

/// Define the multiplication operation for the group. Idea courtesy of:
/// https://www.geeksforgeeks.org/how-to-avoid-overflow-in-modular-multiplication/
/// NOTE: Function does not work if a > U256::max_value()/2 (we get a stack overflow if we try to
/// recursively call itself).
pub fn mul_mod(mut a: U256, mut b: U256, modulus: U256) -> U256 {
    let mut result = U256::from(0);
    a = a % modulus;
    while b > U256::from(0) {
        if b % U256::from(2) == U256::from(1) {
            result = (result + a) % modulus;
        }

        a = (a * U256::from(2)) % modulus;
        b /= U256::from(2);
    }
    return result % modulus;
}

/// Given the xth root of g and yth root of g, finds the xyth root. If the roots are invalid or
/// x and y are not coprime, None is returned. Otherwise, the function performs relevant modular
/// inverse operations on the Bezout coefficients (returned as signed integers) and finds the xyth root.
pub fn shamir_trick(mut xth_root: U256, mut yth_root: U256, x: U256, y: U256, modulus: U256) -> Option<U256> {
    // Check if the inputs are valid.
    if mod_exp(xth_root, x, modulus) != mod_exp(yth_root, y, modulus) {
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
                xth_root = mod_inverse(xth_root, modulus);
                b = -b;
            }

            if a < 0 {
                yth_root = mod_inverse(yth_root, modulus);
                a = -a
            }

            let combined_root: U256 = (mod_exp(xth_root, U256::from(b),  modulus)
                * mod_exp(yth_root, U256::from(a),  modulus)) % modulus;
            return Some(combined_root);
        },
    }
}

/// Compute the modular multiplicative inverse.
/// NOTE: Does not check if gcd != 1(none exists if so).
pub fn mod_inverse(elem: U256, modulus: U256) -> U256 {
    let (_, x, _) = extended_gcd(elem, modulus);

    // Accommodate for negative x coefficient
    if x < 0 {
        // Since we're assuming that the MODULUS will always be larger than than coefficient in
        // absolute value, we simply subtract x from the MODULUS to get a positive value mod N.
        let pos_x = modulus - U256::from(x*-1);
        return pos_x % modulus;
    }
    return U256::from(x) % modulus;
}

/// Returns Bezout coefficients as *signed* integers (since they may be negative).
/// Acts as a wrapper for extended_gcd.
pub fn bezout(a: U256, b: U256) -> Option<(i128, i128)> {
    let (gcd, x, y) = extended_gcd(a, b);
    // Check if a and b are coprime
    if gcd != U256::from(1) {
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
pub fn extended_gcd(a: U256, b: U256) -> (U256, i128, i128) {
    let (mut s, mut old_s): (i128, i128) = (0, 1);
    let (mut t, mut old_t): (i128, i128) = (1, 0);
    let (mut r, mut old_r): (U256, U256) = (b, a);

    while r != U256::from(0) {
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
/// little endian(uses parity-scale-codec). Lambda defines the size of the primes.
/// NOTE: For testing purposes, we are currently limiting primes to u64. Currently unclear about the security
/// of such an adjustment.
pub fn hash_to_prime(elem: &[u8], lambda: U256) -> U256 {
    let mut hash = blake2_256(elem);

    let mut result = U256::from(hash) % lambda;

    // While the resulting hash is not a prime, keep trying
    while !miller_rabin(result) {
        hash = blake2_256(&hash);
        result = U256::from(hash) % lambda;
    }

    return result;
}

/// Implements a deterministic variant of the Miller-Rabin primality test for u64 integers based
/// on the algorithm from the following link: https://en.wikipedia.org/wiki/Miller–Rabin_primality_test
/// Complexity of the algorithm is O((log n)^4) in soft-O notation.
/// In a production setting, one should use the probabilistic variant with larger integers.
pub fn miller_rabin(n: U256) -> bool {
    // Find r and d such that 2^r * d + 1 = n
    let r = (n-U256::from(1)).trailing_zeros();
    let d = (n-U256::from(1)) >> U256::from(r);

    // See section: "Testing against small sets of bases" from the link:
    // https://en.wikipedia.org/wiki/Miller%E2%80%93Rabin_primality_test
    let bases = [2,3,5,7,11,13,17,19,23,29,31,37,41];

    'outer: for &a in bases.iter() {
        // Annoying edge case to make sure a is within [2, n-2] for small n
        if n-U256::from(2) < U256::from(a) { break; }

        let mut x = mod_exp(U256::from(a), d, n);

        if x == U256::from(1) || x == (n-U256::from(1)) {
            continue;
        }
        for _ in 1..r {
            x = mod_exp(x, U256::from(2), n);
            if x == (n-U256::from(1)) {
                continue 'outer;
            }
        }
        return false;
    }
    return true;
}

pub fn root_factor() {

}

#[cfg(test)]
mod tests {
    use super::*;

    const MODULUS: u64 = 13;

    #[test]
    fn test_mod_exp() {
        assert_eq!(mod_exp(U256::from(2), U256::from(7), U256::from(MODULUS)), U256::from(11));
        assert_eq!(mod_exp(U256::from(7), U256::from(15), U256::from(MODULUS)), U256::from(5));
    }

    #[test]
    fn test_extended_gcd() {
        assert_eq!(extended_gcd(U256::from(180), U256::from(150)), (U256::from(30), 1, -1));
        assert_eq!(extended_gcd(U256::from(13), U256::from(17)), (U256::from(1), 4, -3));
    }

    #[test]
    fn test_bezout() {
        assert_eq!(bezout(U256::from(4), U256::from(10)), None);
        assert_eq!(bezout(U256::from(3434), U256::from(2423)), Some((-997, 1413)));
    }

    #[test]
    fn test_shamir_trick() {
        assert_eq!(shamir_trick(U256::from(11), U256::from(6), U256::from(7), U256::from(5), U256::from(MODULUS)), Some(U256::from(7)));
        assert_eq!(shamir_trick(U256::from(11), U256::from(7), U256::from(7), U256::from(11), U256::from(MODULUS)), Some(U256::from(6)));
        assert_eq!(shamir_trick(U256::from(6), U256::from(7), U256::from(5), U256::from(11), U256::from(MODULUS)), Some(U256::from(11)));
        assert_eq!(shamir_trick(U256::from(12), U256::from(7), U256::from(7), U256::from(11), U256::from(MODULUS)), None);
    }

    #[test]
    fn test_mod_inverse() {
        assert_eq!(mod_inverse(U256::from(9), U256::from(MODULUS)), U256::from(3));
        assert_eq!(mod_inverse(U256::from(6), U256::from(MODULUS)), U256::from(11));
    }

    #[test]
    fn test_miller_rabin() {
        assert_eq!(miller_rabin(U256::from(5)), true);
        assert_eq!(miller_rabin(U256::from(7)), true);
        assert_eq!(miller_rabin(U256::from(241)), true);
        assert_eq!(miller_rabin(U256::from(7919)), true);
        assert_eq!(miller_rabin(U256::from(48131)), true);
        assert_eq!(miller_rabin(U256::from(76463)), true);
        assert_eq!(miller_rabin(U256::from(4222234741u64)), true);
        assert_eq!(miller_rabin(U256::from(187278659180417234321u128)), true);

        assert_eq!(miller_rabin(U256::from(21)), false);
        assert_eq!(miller_rabin(U256::from(87)), false);
        assert_eq!(miller_rabin(U256::from(155)), false);
        assert_eq!(miller_rabin(U256::from(9167)), false);
        assert_eq!(miller_rabin(U256::from(102398)), false);
        assert_eq!(miller_rabin(U256::from(801435)), false);
        assert_eq!(miller_rabin(U256::from(51456119958243u128)), false);
    }

    #[test]
    fn test_hash_to_prime() {
        // Key values checked: 0, 1, 2
        //assert_eq!(hash_to_prime(&U256::from(0).encode(), U256::max_value()/U256::from(8)), U256::from(1121));
    }
}