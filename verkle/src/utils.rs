use alloy_primitives::B256;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, Fr};

pub fn fr_to_b256(value: &Fr) -> B256 {
    let mut buf = vec![];
    value.serialize_compressed(&mut buf).unwrap();
    B256::from_slice(&buf)
}

pub fn b256_to_fr(value: &B256) -> Fr {
    Fr::deserialize_compressed(value.as_slice()).unwrap()
}
