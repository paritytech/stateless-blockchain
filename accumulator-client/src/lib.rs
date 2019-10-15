use accumulator::*;
use wasm_bindgen::prelude::*;
use codec::{Encode, Decode};

#[wasm_bindgen]
pub fn hash_to_prime(elem: &[u8]) -> Vec<u8> {
    let mut result: [u8; 256] = [0; 256];
    subroutines::hash_to_prime(elem).to_little_endian(&mut result);
    return result.to_vec();
}

#[cfg(test)]
mod tests {
    use super::*;
    use accumulator::*;

    #[test]
    fn test_hash_to_prime() {
        let elem = [7, 10];
        assert_eq!(subroutines::hash_to_prime(&elem), U2048::from_dec_str("7380741765666080429").unwrap());
    }

}