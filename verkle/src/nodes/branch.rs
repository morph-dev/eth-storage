use std::collections::BTreeMap;

use alloy_primitives::B256;
use anyhow::Result;
use banderwagon::{Element, Fr};
use ssz::{Decode, Encode};

use crate::{
    committer::DEFAULT_COMMITER,
    utils::{b256_to_element, element_to_b256},
    Db, TrieKey, TrieValue,
};

use super::{node::NodeTrait, CommitmentNode, LeafNode, Node};

pub struct BranchNode {
    values: BTreeMap<u8, Node>,
    commitment: Element,
}

impl BranchNode {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            commitment: Element::zero(),
        }
    }

    pub fn set(&mut self, index: u8, node: Node) {
        let old_node = self.values.insert(index, node);
        self.update_commitment(
            index,
            old_node
                .map(|node| node.commitment_hash())
                .unwrap_or_default(),
        );
    }

    pub(super) fn get_mut(&mut self, index: u8) -> Option<&mut Node> {
        self.values.get_mut(&index)
    }

    pub fn insert(&mut self, depth: usize, key: TrieKey, value: TrieValue, db: &Db) -> Result<()> {
        let index = key[depth];
        let pre_commitment = self.get_child_commit(index);
        match self.values.get_mut(&index) {
            Some(node) => {
                node.insert(depth + 1, key, value, db)?;
            }
            None => {
                self.values
                    .insert(index, Node::Leaf(LeafNode::new_for_key_value(&key, value)));
            }
        };
        self.update_commitment(index, pre_commitment);
        Ok(())
    }

    fn get_child_commit(&mut self, index: u8) -> Fr {
        self.values
            .get_mut(&index)
            .map(|node| node.commitment_hash_write())
            .unwrap_or_default()
    }

    fn update_commitment(&mut self, index: u8, pre_commitment: Fr) {
        let post_commitment = self.get_child_commit(index);
        self.commitment +=
            DEFAULT_COMMITER.scalar_mul(index as usize, post_commitment - pre_commitment);
    }

    pub fn write_and_commit(&mut self, db: &mut Db) -> Result<Element> {
        for (_, node) in self.values.iter_mut() {
            node.write_and_commit(db)?;
        }
        Ok(self.commitment_write())
    }
}

impl Default for BranchNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeTrait for BranchNode {
    fn commitment_write(&mut self) -> Element {
        self.commitment
    }

    fn commitment(&self) -> Element {
        self.commitment
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
            .map(|(index, node)| (*index, element_to_b256(&node.commitment())))
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

        let values = commitments
            .iter()
            .map(|(index, c)| {
                (
                    *index,
                    Node::Commitment(CommitmentNode::new(b256_to_element(c))),
                )
            })
            .collect();

        let commitment = DEFAULT_COMMITER.commit_sparse(
            commitments
                .iter()
                .map(|(index, c)| (*index as usize, b256_to_element(c).map_to_scalar_field()))
                .collect(),
        );

        Ok(Self { values, commitment })
    }
}
