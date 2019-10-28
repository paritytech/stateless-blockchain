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

/// Need to test
#[wasm_bindgen]
pub fn get_witness(old_state: &[u8], elem: &[u8], agg: &[u8]) -> Vec<u8> {
    let mut result: [u8; 256] = [0; 256];
    witnesses::create_single_mem_wit(U2048::from_little_endian(old_state), U2048::from_little_endian(elem),
                                     U2048::from_little_endian(agg)).unwrap().to_little_endian(&mut result);
    return result.to_vec();
}

/// Need to test
#[wasm_bindgen]
pub fn update_witness(elem: &[u8], witness: &[u8], new_state: &[u8], added: &[u8], deleted: &[u8]) -> Vec<u8> {
    let mut result: [u8; 256] = [0; 256];
    witnesses::update_mem_wit(U2048::from_little_endian(elem), U2048::from_little_endian(witness), U2048::from_little_endian(new_state),
                              U2048::from_little_endian(added), U2048::from_little_endian(deleted)).to_little_endian(&mut result);
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