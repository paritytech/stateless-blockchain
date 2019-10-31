/// Binary Vector Commitments
/// Functions have been slightly modified from original paper specification.

use accumulator::*;

/// A witness can either be a membership witness or a non-membership witness.
#[derive(Clone, Copy)]
pub enum Witness {
    MemWit(U2048),
    NonMemWit((i128, U2048)),
}

/// Commit a vector of bits(represented as bool array) to an accumulator. The second value of
/// the tuple is the product of the accumulated elements. In the stateless blockchain model,
/// after the validator commits the vector to the accumulator, users should immediately request
/// membership witnesses using the "product" value.
pub fn commit(accumulator: U2048, m: &[bool; super::N]) -> (U2048, U2048) {
    let elems: Vec<U2048>  = m
        .into_iter()
        .enumerate()
        .filter(|(_, i)| **i)
        .map(|(i, _)| subroutines::hash_to_prime(&i.to_le_bytes()))
        .collect();
    let (state, _, _) = batch_add(accumulator, &elems);
    let product = subroutines::prime_product(elems);
    return (state, product);
}

/// Create an opening for a bit commitment. The current state of the accumulator should equal
/// "old_state" raised to the "agg" power(product of aggregated elements).
pub fn open(old_state: U2048, bit: bool, index: usize, agg: U2048) -> Witness {
    let elem = subroutines::hash_to_prime(&index.to_le_bytes());
    if bit {
        return Witness::MemWit(witnesses::mem_wit_create(old_state, agg, elem).unwrap());
    }
    else {
        return Witness::NonMemWit(witnesses::non_mem_wit_create(old_state, agg, elem));
    }
}

/// Verifies a membership/non-membership proof (produced by an opening) for a given bit commitment.
pub fn verify(accumulator: U2048, bit: bool, index: usize, proof: Witness) -> bool {
    let elem = subroutines::hash_to_prime(&index.to_le_bytes());
    if bit {
        match proof {
            Witness::MemWit(witness) => {
                return witnesses::verify_mem_wit(accumulator, witness, elem);
            },
            Witness::NonMemWit(_) => {
                return false;
            },
        }
    }
    else {
        match proof {
            Witness::NonMemWit(witness) => {
                return witnesses::verify_non_mem_wit(accumulator, witness, elem);
            },
            Witness::MemWit(_) => {
                return false;
            },
        }
    }
}

pub fn batch_open() {

}

pub fn batch_verify() {

}


pub fn update() {

}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::N;

    #[test]
    fn test_commit() {
        let accumulator = U2048::from(2);
        let arr: [bool; N] = [true, false, true];
        let (state, _) = commit(accumulator, &arr);

        let h_0 = subroutines::hash_to_prime(&[0 as u8]);
        let h_2 = subroutines::hash_to_prime(&[2 as u8]);

        assert_eq!(subroutines::mod_exp(accumulator, h_0*h_2, U2048::from_dec_str(MODULUS).unwrap()), state);
    }

    #[test]
    fn test_open_and_verify() {
        // Commit vector
        let accumulator = U2048::from(2);
        let arr: [bool; N] = [true, false, true];
        let (state, product) = commit(accumulator, &arr);

        // Open at two indices
        let open_1 = open(U2048::from(2), false, 1, product);
        let open_2 = open(U2048::from(2), true, 2, product);

        // Verify
        assert_eq!(verify(state, false, 1, open_1), true);
        assert_eq!(verify(state, true, 1, open_1), false);
        assert_eq!(verify(state, false, 1, open_2), false);

        assert_eq!(verify(state, true, 2, open_2), true);
        assert_eq!(verify(state, false, 2, open_2), false);
        assert_eq!(verify(state, true, 2, open_1), false);
    }
    

}