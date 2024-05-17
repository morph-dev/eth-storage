use alloy_primitives::B256;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, Element, Fr};

pub fn element_to_b256(value: &Element) -> B256 {
    let mut b256 = B256::ZERO;
    value.serialize_compressed(b256.as_mut_slice()).unwrap();
    b256
}

pub fn b256_to_element(value: &B256) -> Element {
    Element::deserialize_compressed(value.as_slice()).unwrap()
}

pub fn fr_to_b256(value: &Fr) -> B256 {
    let mut buf = vec![];
    value.serialize_compressed(&mut buf).unwrap();
    B256::from_slice(&buf)
}
