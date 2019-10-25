use accumulator::*;
use wasm_bindgen::prelude::*;
use codec::{Encode, Decode};
use primitive_types::H256;

#[wasm_bindgen]
#[derive(Encode, Decode)]
pub struct UTXO {
    pub_key: H256,
    id: u64,
}

#[wasm_bindgen]
pub fn create_utxo(pub_key: &[u8], id: u64) -> UTXO {
    let result = UTXO {
        pub_key: H256::from_slice(pub_key),
        id,
    };
    return result;
}

#[wasm_bindgen]
pub fn get_utxo_elem(pub_key: &[u8], id: u64) -> Vec<u8> {
    return create_utxo(pub_key, id).encode();
}

#[wasm_bindgen]
pub fn hash_to_prime(elem: &[u8]) -> Vec<u8> {
    let mut result: [u8; 256] = [0; 256];  // Change this constant
    subroutines::hash_to_prime(elem).to_little_endian(&mut result);
    return result.to_vec();
}

#[cfg(test)]
mod tests {
    use super::*;
    use accumulator::*;

    #[test]
    fn test_hash_to_prime() {
        let utxo = UTXO {
            pub_key: H256::from_slice(hex::decode("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").unwrap()),
            id: 0,
        };
        assert_eq!(subroutines::hash_to_prime(&utxo.encode()), U2048::from_dec_str("2882671871935824533").unwrap());
    }

}