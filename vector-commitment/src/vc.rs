/// Vector Commitments for Integer Key Value Store
/// Acts as a wrapper for binary vector commitments.

use accumulator::*;
use bit_vec::BitVec;
use crate::binary;

type ValueType = u8;

pub fn commit(accumulator: U2048, keys: &[usize], values: &[ValueType]) -> (U2048, U2048) {
    let (binary_vec, indices) = convert_key_value(keys, values);
    return binary::commit(accumulator, &binary_vec, &indices);
}

/// UNTESTED
/// This function would be immediately called by a user following a relevant state commitment.
pub fn open_at_key(old_state: U2048, product: U2048, key: usize, value: ValueType) -> (Witness, Witness) {
    let (binary_vec, indices) = convert_key_value(&[key], &[value]);
    return binary::batch_open(old_state, product, &binary_vec, &indices);
}

/// UNTESTED
pub fn verify_at_key(old_state: U2048, accumulator: U2048, key: usize, value: ValueType, pi_i: Witness, pi_e: Witness) -> bool {
    let (binary_vec, indices) = convert_key_value(&[key], &[value]);
    return binary::batch_verify(old_state, accumulator, &binary_vec, &indices, pi_i, pi_e);
}

/// UNTESTED
pub fn update(accumulator: U2048, old_state: U2048, agg: U2048, keys: &[usize], values: &[ValueType]) -> U2048 {
    let (binary_vec, indices) = convert_key_value(keys, values);
    return binary::update(accumulator, old_state, agg, &binary_vec, &indices);
}

pub fn convert_key_value(keys: &[usize], values: &[ValueType]) -> (Vec<bool>, Vec<usize>) {
    let mut binary_vec: Vec<bool> = [].to_vec();
    let mut indices: Vec<usize> = [].to_vec();
    for (i, &value) in values.iter().enumerate() {
        let mut value_vec = to_binary(value);
        let offset = core::mem::size_of::<ValueType>()*8;
        let mut index_vec = (keys[i]*offset..keys[i]*offset+offset).collect();
        binary_vec.append(&mut value_vec);
        indices.append(&mut index_vec);
    }
    return (binary_vec, indices);
}

pub fn to_binary(elem: ValueType) -> Vec<bool> {
    let byte_vec = elem.to_le_bytes().to_vec();
    let bv = BitVec::from_bytes(&byte_vec);
    return bv.iter().collect::<Vec<bool>>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::N;

    #[test]
    fn test_to_binary() {
        let elem: ValueType = 6;
        let bv = to_binary(elem);
        assert_eq!(bv, vec![false, false, false, false, false, true, true, false]);
    }

    #[test]
    fn test_commit() {
        let accumulator: U2048 = U2048::from(2);
        let keys = [0, 1];
        let values = vec![4, 7];

        let (new_accumulator, product) = commit(accumulator, &keys, &values);

        // Manual check
        let check_product = subroutines::hash_to_prime(&(5 as usize).to_le_bytes())
            * subroutines::hash_to_prime(&(13 as usize).to_le_bytes())
            * subroutines::hash_to_prime(&(14 as usize).to_le_bytes())
            * subroutines::hash_to_prime(&(15 as usize).to_le_bytes());

        assert_eq!(new_accumulator, subroutines::mod_exp(U2048::from(2), U2048::from(check_product), U2048::from_dec_str(MODULUS).unwrap()));
    }

    #[test]
    fn test_convert() {
        let keys = vec![0, 1];
        let values = vec![4, 7];
        let (binary_vec, indices) = convert_key_value(&keys, &values);
        assert_eq!(binary_vec, vec![false, false, false, false, false, true, false, false, false, false, false, false,
            false, true, true, true]);
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn test_vc_open_and_verify() {
        let accumulator: U2048 = U2048::from(2);
        let keys = vec![0, 1];
        let values = vec![4, 7];
        let (new_accumulator, product) = commit(accumulator, &keys, &values);
        runtime_io::print(product.low_u64());

        let (pi_i, pi_e) = open_at_key(accumulator, product, 1, 7);
        assert_eq!(verify_at_key(accumulator, new_accumulator, 1, 7, pi_i, pi_e), true);

        assert_eq!(verify_at_key(accumulator, new_accumulator, 0, 7, pi_i, pi_e), false);
        assert_eq!(verify_at_key(accumulator, new_accumulator, 1, 4, pi_i, pi_e), false);
    }
}