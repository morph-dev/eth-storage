use std::{collections::BTreeMap, ops::Deref};

use ark_ff::{BigInteger, BigInteger256};
use banderwagon::{Element, Fr, One, PrimeField, Zero};
use derive_more::Index;
use once_cell::sync::Lazy;
use ssz::{Decode, SszDecoderBuilder};
use ssz_derive::Encode;

use crate::{
    committer::DEFAULT_COMMITER, constants::VERKLE_NODE_WIDTH, crs::CRS, TrieKey, TrieStem,
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
    stem: TrieStem,
    #[index]
    values: BTreeMap<u8, TrieValue>,

    #[ssz(skip_serializing)]
    cp1: Element,
    #[ssz(skip_serializing)]
    cp2: Element,

    #[ssz(skip_serializing)]
    const_cp: Element,
    #[ssz(skip_serializing)]
    c: Option<Fr>,
}

impl LeafNode {
    pub fn new(stem: TrieStem) -> Self {
        let const_c = DEFAULT_COMMITER.commit_sparse(vec![
            (0, Fr::one()),
            (1, Fr::from_le_bytes_mod_order(stem.as_slice())),
        ]);
        Self {
            stem,
            values: BTreeMap::new(),
            cp1: Element::zero(),
            cp2: Element::zero(),
            const_cp: const_c,
            c: None,
        }
    }

    pub fn new_for_key_value(key: &TrieKey, value: TrieValue) -> Self {
        let mut result = Self::new(key.stem());
        result.set(key.last(), value);
        result
    }

    pub fn stem(&self) -> &TrieStem {
        &self.stem
    }

    fn calculate_commitment(&self) -> Fr {
        let c = self.const_cp
            + DEFAULT_COMMITER.commit_sparse(vec![
                (2, self.cp1.map_to_scalar_field()),
                (3, self.cp2.map_to_scalar_field()),
            ]);
        c.map_to_scalar_field()
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

        let low_index = (index % VERKLE_NODE_WIDTH) / 2;
        let high_index = low_index + 1;

        let diff = CRS[low_index] * (value_low_16 - old_value_low_16)
            + CRS[high_index] * (value_high_16 - old_value_high_16);

        if index < VERKLE_NODE_WIDTH / 2 {
            self.cp1 += diff;
        } else {
            self.cp2 += diff;
        };
        self.c = None;
    }

    pub fn set_all(&mut self, values: impl IntoIterator<Item = (u8, TrieValue)>) {
        for (index, value) in values {
            self.set(index, value)
        }
    }

    fn value_low_high_16(value: &TrieValue) -> (Fr, Fr) {
        (
            Fr::from_le_bytes_mod_order(&value.as_le_slice()[0..16]) + TWO_POWER_128.deref(),
            Fr::from_le_bytes_mod_order(&value.as_le_slice()[16..32]),
        )
    }
}

impl NodeTrait for LeafNode {
    fn commit(&self) -> Fr {
        self.c.unwrap_or_else(|| self.calculate_commitment())
    }

    fn commit_mut(&mut self) -> Fr {
        self.c = self.c.or_else(|| Some(self.calculate_commitment()));
        self.c.expect("Value must be present")
    }
}

// impl NodeTrait for LeafNode {
//     fn commit(&self) -> Fr {
//         self.c.unwrap_or_else(|| {
//             let c = self.const_c
//                 + CRS[2] * self.c1.map_to_scalar_field()
//                 + CRS[3] * self.c2.map_to_scalar_field();
//             c.map_to_scalar_field()
//         })
//     }

//     fn commit_and_save(&mut self, db: &mut Db) -> Fr {
//         let c = self.commit();
//         self.c = Some(c);
//         db.write(c, self.as_ssz_bytes()).unwrap();
//         c
//     }
// }

impl Decode for LeafNode {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let mut decoder_builder = SszDecoderBuilder::new(bytes);
        decoder_builder.register_type::<TrieStem>()?;
        decoder_builder.register_type::<BTreeMap<u8, TrieValue>>()?;

        let mut decoder = decoder_builder.build()?;
        let stem = decoder.decode_next::<TrieStem>()?;
        let values = decoder.decode_next::<BTreeMap<u8, TrieValue>>()?;

        let mut result = Self::new(stem);
        result.set_all(values);
        Ok(result)
    }
}
