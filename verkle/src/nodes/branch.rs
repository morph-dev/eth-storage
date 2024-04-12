use std::{array, collections::BTreeMap, ops::DerefMut};

use alloy_primitives::B256;
use anyhow::Result;
use banderwagon::{Element, Fr};
use ssz::{Decode, Encode};

use crate::{
    committer::DEFAULT_COMMITER,
    constants::VERKLE_NODE_WIDTH,
    utils::{b256_to_fr, fr_to_b256},
    Db, TrieKey, TrieValue,
};

use super::{node::NodeTrait, Node};

pub struct BranchNode {
    values: Box<[Node; VERKLE_NODE_WIDTH]>,
    cp: Element,
}

impl BranchNode {
    pub fn new() -> Self {
        Self {
            values: array::from_fn(|_| Node::Empty).into(),
            cp: Element::zero(),
        }
    }

    pub fn set(&mut self, index: usize, node: Node) {
        let node_at_index = &mut self.values[index];
        let pre_commitment = node_at_index.commit();
        *node_at_index = node;
        let post_commitment = node_at_index.commit();
        self.cp += DEFAULT_COMMITER.scalar_mul(index, post_commitment - pre_commitment);
    }

    pub(super) fn get_mut(&mut self, index: usize) -> &mut Node {
        &mut self.values[index]
    }

    pub fn insert(&mut self, depth: usize, key: TrieKey, value: TrieValue, db: &Db) -> Result<()> {
        let index = key[depth] as usize;
        let node = &mut self.values[index];
        let pre_commitment = node.commit();
        node.insert(depth + 1, key, value, db)?;
        let post_commitment = node.commit();
        self.cp += DEFAULT_COMMITER.scalar_mul(index, post_commitment - pre_commitment);
        Ok(())
    }

    pub fn write_and_commit(&mut self, db: &mut Db) -> Result<Fr> {
        for node in self.values.deref_mut() {
            node.write_and_commit(db)?;
        }
        Ok(self.commit())
    }
}

impl Default for BranchNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeTrait for BranchNode {
    fn commitment(&self) -> Fr {
        self.cp.map_to_scalar_field()
    }
}

impl Encode for BranchNode {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let commitments: BTreeMap<u8, B256> = self
            .values
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                if node.is_empty() {
                    None
                } else {
                    Some((index as u8, fr_to_b256(&node.commitment())))
                }
            })
            .collect();
        commitments.ssz_append(buf);
    }

    fn ssz_bytes_len(&self) -> usize {
        // TODO: optimize this
        // let number_of_non_empty_items = self.values.iter().filter(|node| !node.is_empty()).count();
        // let size_per_item = <(u8, B256) as Decode>::ssz_fixed_len() ;
        // number_of_non_empty_items * size_per_item
        self.as_ssz_bytes().len()
    }
}

impl Decode for BranchNode {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let commitments = BTreeMap::<u8, B256>::from_ssz_bytes(bytes)?;
        let commitments: BTreeMap<usize, Fr> = commitments
            .iter()
            .map(|(index, commitment)| (*index as usize, b256_to_fr(commitment)))
            .collect();

        let values = array::from_fn(|i| {
            commitments
                .get(&i)
                .map(|c| Node::Commitment(*c))
                .unwrap_or_else(|| Node::Empty)
        });
        let cp = DEFAULT_COMMITER.commit_sparse(commitments.into_iter().collect());

        Ok(Self {
            values: values.into(),
            cp,
        })
    }
}
