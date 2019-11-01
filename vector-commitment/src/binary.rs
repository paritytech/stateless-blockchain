/// Binary Vector Commitments
/// Functions have been slightly modified from original paper specification.

use accumulator::*;
use runtime_io::print;
use accumulator::witnesses::mem_wit_create;

/// Commit a vector of bits(represented as bool array) to an accumulator. The second value of
/// the returned tuple is the product of the accumulated elements.
/// NOTE: In the stateless blockchain model, after the validator commits the vector to the accumulator,
/// users should immediately request membership witnesses for their committed bit using the "product" value.
pub fn commit(accumulator: U2048, m: &[bool]) -> (U2048, U2048) {
    let elems: Vec<U2048> = m
        .into_iter()
        .enumerate()
        .filter(|(_, i)| **i)
        .map(|(i, _)| subroutines::hash_to_prime(&i.to_le_bytes()))
        .collect();
    let (state, _, _) = batch_add(accumulator, &elems);
    let product = subroutines::prime_product(&elems);
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

/// Given a bit array and an array of corresponding indices, outputs the product of the "ones"
/// elements and the product of the "zeros" elements.
pub fn get_bit_elems(b: &[bool], i: &[usize]) -> (U2048, U2048) {
    let ones_indices: Vec<usize> = b
        .into_iter()
        .enumerate()
        .filter(|(_, bit)| **bit)
        .map(|(index, _)| index)
        .collect();

    let zeros_indices: Vec<usize> = b
        .into_iter()
        .enumerate()
        .filter(|(_, bit)| !**bit)
        .map(|(index, _)| index)
        .collect();

    let ones: Vec<U2048> = ones_indices
        .into_iter()
        .enumerate()
        .map(|(_, index)| subroutines::hash_to_prime(&(i[index]).to_le_bytes()))
        .collect();

    let zeros: Vec<U2048> = zeros_indices
        .into_iter()
        .enumerate()
        .map(|(_, index)| subroutines::hash_to_prime(&(i[index]).to_le_bytes()))
        .collect();

    let p_ones = subroutines::prime_product(&ones);
    let p_zeros = subroutines::prime_product(&zeros);
    return (p_ones, p_zeros)
}

/// Batch opens a set of bit commitments. The accumulated values of the commitments must be contained in
/// the inputted aggregated value(agg) and the current state of the accumulator must equal old_state^agg.
/// This function has been slightly modified from the original specification. See page 20 of the paper for more info.
pub fn batch_open(old_state: U2048, agg: U2048, b: &[bool], i: &[usize]) -> (Witness, Witness) {
    let (p_ones, p_zeros) = get_bit_elems(b, i);

    let pi_inclusion = Witness::MemWit(witnesses::mem_wit_create(old_state, agg, p_ones).unwrap());
    let pi_exclusion = Witness::NonMemWit(witnesses::non_mem_wit_create(old_state, agg, p_zeros));

    return (pi_inclusion, pi_exclusion);
}

/// Verifies a set of membership and non-membership witnesses for a set of bit commitments.
/// This function has been slightly modified from the original specification. See page 20 of the paper for more info.
pub fn batch_verify(accumulator: U2048, b: &[bool], i: &[usize], pi_i: Witness, pi_e: Witness) -> bool {
    let (p_ones, p_zeros) = get_bit_elems(b, i);

    let mut ver_mem_result = false;
    match pi_i {
        Witness::MemWit(mem_wit) => {
            ver_mem_result = witnesses::verify_mem_wit(accumulator, mem_wit, p_ones);
        },
        Witness::NonMemWit(_) => {
            return false;
        },
    }

    let mut ver_non_mem_result = false;
    match pi_e {
        Witness::MemWit(_) => {
            return false;
        },
        Witness::NonMemWit(non_mem_wit) => {
            ver_non_mem_result = witnesses::verify_non_mem_wit(accumulator,non_mem_wit, p_zeros);
        },
    }
    return ver_mem_result && ver_non_mem_result;
}

/// Updates a segment of a vector commitment.
/// Arguments:
/// - accumulator: The current state of the accumulator.
/// - old_state: A previous state.
/// - agg: Product of some aggregated elements s.t. old_state^agg = accumulator
/// - b: New bit array.
/// - i: Affected indices.
pub fn update(accumulator: U2048, old_state: U2048, agg: U2048, b: &[bool], i: &[usize]) -> U2048 {
    let (p_ones, p_zeros) = get_bit_elems(b, i);

    // Delete p_zeros elements
    let mem_wit = mem_wit_create(old_state, agg, p_zeros).unwrap();
    let mut new_state = delete(accumulator, p_zeros, mem_wit).unwrap();

    // Add p_ones elements
    new_state = add(new_state, p_ones);

    return new_state;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::N;

    #[test]
    fn test_open_and_verify() {
        // Commit vector
        let accumulator = U2048::from(2);
        let arr: [bool; N] = [true, false, true];
        let (state, product) = commit(accumulator, &arr);

        // Check commit
        let h_0 = subroutines::hash_to_prime(&(0 as usize).to_le_bytes());
        let h_2 = subroutines::hash_to_prime(&(2 as usize).to_le_bytes());
        assert_eq!(subroutines::mod_exp(accumulator, h_0*h_2, U2048::from_dec_str(MODULUS).unwrap()), state);

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

    #[test]
    fn test_get_bit_elems() {
        let arr: [bool; N] = [false, false, true];
        let indices = [0, 1, 5];

        let h_0 = subroutines::hash_to_prime(&(0 as usize).to_le_bytes());
        let h_1 = subroutines::hash_to_prime(&(1 as usize).to_le_bytes());
        let h_5 = subroutines::hash_to_prime(&(5 as usize).to_le_bytes());

        let (p_ones, p_zeros) = get_bit_elems(&arr, &indices);
        assert_eq!(p_ones, h_5);
        assert_eq!(p_zeros, h_0 * h_1);
    }

    #[test]
    fn test_batch_open_and_verify() {
        let accumulator = U2048::from(2);
        let arr: [bool; 6] = [true, false, true, false, false, true];
        let (state, product) = commit(accumulator, &arr);

        let (i, e) = batch_open(accumulator, product, &[true, false, false, true], &[0, 3, 4, 5]);

        let h_0 = subroutines::hash_to_prime(&(0 as usize).to_le_bytes());
        let h_3 = subroutines::hash_to_prime(&(3 as usize).to_le_bytes());
        let h_4 = subroutines::hash_to_prime(&(4 as usize).to_le_bytes());
        let h_5 = subroutines::hash_to_prime(&(5 as usize).to_le_bytes());

//        // Manual check of openings
//        let ones_product = subroutines::prime_product(&vec![h_0, h_5]);
//        let zeros_product = subroutines::prime_product(&vec![h_3, h_4]);
//
//        let mut mem_result = false;
//        let mut non_mem_result = false;
//
//        match i {
//            Witness::MemWit(mem_wit) => {
//                mem_result = witnesses::verify_mem_wit(state, mem_wit, ones_product);
//            },
//            Witness::NonMemWit(_) => { },
//        }
//
//        match e {
//            Witness::MemWit(_) => { },
//            Witness::NonMemWit(non_mem_wit) => {
//                non_mem_result = witnesses::verify_non_mem_wit(state, non_mem_wit, zeros_product);
//            },
//        }
//        assert_eq!(mem_result && non_mem_result, true);

        assert_eq!(batch_verify(state, &[true, false, false, true], &[0, 3, 4, 5], i, e), true);
    }

    #[test]
    fn test_update() {
        let accumulator = U2048::from(2);
        let arr: [bool; 6] = [true, false, true, false, false, true];
        let (state, product) = commit(accumulator, &arr);

        let h_0 = subroutines::hash_to_prime(&(0 as usize).to_le_bytes());
        let h_3 = subroutines::hash_to_prime(&(3 as usize).to_le_bytes());
        let h_4 = subroutines::hash_to_prime(&(4 as usize).to_le_bytes());

        // Missing: checking that inputs are valid
        let new_state = update(state, accumulator, product, &[false, true, true, false], &[2, 3, 4, 5]);
        assert_eq!(new_state, subroutines::mod_exp(accumulator, h_0 * h_3 * h_4, U2048::from_dec_str(MODULUS).unwrap()));
    }

}