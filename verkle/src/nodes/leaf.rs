use std::{collections::BTreeMap, ops::Deref};

use ark_ff::{BigInteger, BigInteger256};
use banderwagon::{Element, Fr, One, PrimeField, Zero};
use derive_more::Index;
use once_cell::sync::Lazy;
use ssz::{Decode, SszDecoderBuilder};
use ssz_derive::Encode;

use crate::{
    committer::DEFAULT_COMMITER, constants::VERKLE_NODE_WIDTH, crs::CRS, stem::Stem, TrieKey,
    TrieValue,
};

use super::node::NodeTrait;

static TWO_POWER_128: Lazy<Fr> = Lazy::new(|| {
    let mut x = BigInteger256::one();
    x.muln(128);
    x.into()
});

#[derive(Index, Encode)]
pub struct LeafNode {
    stem: Stem,
    #[index]
    values: BTreeMap<u8, TrieValue>,

    #[ssz(skip_serializing)]
    c1: Element,
    #[ssz(skip_serializing)]
    c2: Element,

    #[ssz(skip_serializing)]
    const_c: Element,
    #[ssz(skip_serializing)]
    hash_commitment: Option<Fr>,
}

impl LeafNode {
    pub fn new(stem: Stem) -> Self {
        let const_c = DEFAULT_COMMITER.commit_sparse(vec![
            (0, Fr::one()),
            (1, Fr::from_le_bytes_mod_order(stem.as_slice())),
        ]);
        Self {
            stem,
            values: BTreeMap::new(),
            c1: Element::zero(),
            c2: Element::zero(),
            const_c,
            hash_commitment: None,
        }
    }

    pub fn new_for_key_value(key: &TrieKey, value: TrieValue) -> Self {
        let mut result = Self::new(key.stem());
        result.set(key.last(), value);
        result
    }

    pub fn stem(&self) -> &Stem {
        &self.stem
    }

    fn calculate_commitment(&self) -> Element {
        self.const_c
            + DEFAULT_COMMITER.commit_sparse(vec![
                (2, self.c1.map_to_scalar_field()),
                (3, self.c2.map_to_scalar_field()),
            ])
    }

    pub fn get(&self, index: u8) -> Option<&TrieValue> {
        self.values.get(&index)
    }

    pub fn set(&mut self, index: u8, value: TrieValue) {
        let old_value = self.values.insert(index, value);

        let index = index as usize;

        // update commitments

        let (value_low_16, value_high_16) = Self::value_low_high_16(&value);
        let (old_value_low_16, old_value_high_16) = old_value
            .map_or((Fr::zero(), Fr::zero()), |old_value| {
                Self::value_low_high_16(&old_value)
            });

        let low_index = index % (VERKLE_NODE_WIDTH / 2) * 2;
        let high_index = low_index + 1;

        let diff = CRS[low_index] * (value_low_16 - old_value_low_16)
            + CRS[high_index] * (value_high_16 - old_value_high_16);

        if index < VERKLE_NODE_WIDTH / 2 {
            self.c1 += diff;
        } else {
            self.c2 += diff;
        };
        self.hash_commitment = None;
    }

    pub fn set_all(&mut self, values: impl IntoIterator<Item = (u8, TrieValue)>) {
        for (index, value) in values {
            self.set(index, value)
        }
    }

    fn value_low_high_16(value: &TrieValue) -> (Fr, Fr) {
        let value_as_le_slice = value.as_le_slice();
        (
            Fr::from_le_bytes_mod_order(&value_as_le_slice[0..16]) + TWO_POWER_128.deref(),
            Fr::from_le_bytes_mod_order(&value_as_le_slice[16..32]),
        )
    }
}

impl NodeTrait for LeafNode {
    fn hash_commitment(&self) -> Fr {
        self.hash_commitment
            .unwrap_or_else(|| self.calculate_commitment().map_to_scalar_field())
    }

    fn hash_commitment_mut(&mut self) -> Fr {
        self.hash_commitment = Some(self.hash_commitment());
        self.hash_commitment.expect("Value must be present")
    }
}

impl Decode for LeafNode {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let mut decoder_builder = SszDecoderBuilder::new(bytes);
        decoder_builder.register_type::<Stem>()?;
        decoder_builder.register_type::<BTreeMap<u8, TrieValue>>()?;

        let mut decoder = decoder_builder.build()?;
        let stem = decoder.decode_next::<Stem>()?;
        let values = decoder.decode_next::<BTreeMap<u8, TrieValue>>()?;

        let mut result = Self::new(stem);
        result.set_all(values);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{B256, U256};

    use crate::utils::fr_to_b256;

    use super::*;

    #[test]
    fn insert_key0_value0() {
        let key = TrieKey::new(B256::ZERO);
        let mut leaf = LeafNode::new_for_key_value(&key, TrieValue::ZERO);

        assert_eq!(
            fr_to_b256(&leaf.hash_commitment_mut()).to_string(),
            "0x1c0727f0c6c9887189f75a9d08b804aba20892a238e147750767eac22a830d08"
        );
    }

    #[test]
    fn insert_key1_value1() {
        let key = TrieKey::new(U256::from(1).into());
        let mut leaf = LeafNode::new_for_key_value(&key, TrieValue::from(1));

        assert_eq!(
            fr_to_b256(&leaf.hash_commitment_mut()).to_string(),
            "0x6ef020caaeda01ff573afe6df6460d4aae14b4987e02ea39074f270ce62dfc14"
        );
    }

    #[test]
    fn insert_increasing() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let key = TrieKey::new(B256::from(bytes));
        let mut leaf = LeafNode::new_for_key_value(&key, TrieValue::from_le_bytes(bytes));

        assert_eq!(
            fr_to_b256(&leaf.hash_commitment_mut()).to_string(),
            "0xb897ba52c5317acd75f5f3c3922f461357d4fb8b685fe63f20a3b2adb014370a"
        );
    }

    #[test]
    fn insert_eoa_with_1eth_balance() {
        let stem = Stem::from(&TrieKey::from(B256::from([
            245, 110, 100, 66, 36, 244, 87, 100, 144, 207, 224, 222, 20, 36, 164, 83, 34, 18, 82,
            155, 254, 55, 71, 19, 216, 78, 125, 126, 142, 146, 114, 0,
        ])));
        let values = vec![
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            [
                0, 0, 100, 167, 179, 182, 224, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
            ],
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            [
                197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0,
                182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112,
            ],
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
        ];
        let mut leaf = LeafNode::new(stem);
        leaf.set_all(
            values
                .into_iter()
                .enumerate()
                .map(|(index, value)| (index as u8, TrieValue::from_le_bytes(value))),
        );

        assert_eq!(
            fr_to_b256(&leaf.hash_commitment_mut()).to_string(),
            "0xcc30be1f0d50eacfacaa3361b8df4d2014a849854a6cf35e6c55e07d6963f519"
        );
    }
}
