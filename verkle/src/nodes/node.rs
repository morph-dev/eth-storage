use std::mem;

use alloy_primitives::B256;
use anyhow::{anyhow, bail, Result};
use banderwagon::Fr;
use ssz::{Decode, Encode};

use crate::{Db, TrieKey, TrieValue};

use super::{BranchNode, LeafNode};

pub trait NodeTrait {
    fn hash_commitment(&self) -> Fr;

    fn hash_commitment_mut(&mut self) -> Fr {
        self.hash_commitment()
    }
}

pub enum Node {
    Branch(BranchNode),
    Leaf(LeafNode),
    Commitment(Fr),
}

impl NodeTrait for Node {
    fn hash_commitment(&self) -> Fr {
        match self {
            Node::Branch(branch_node) => branch_node.hash_commitment(),
            Node::Leaf(leaf_node) => leaf_node.hash_commitment(),
            Node::Commitment(c) => *c,
        }
    }

    fn hash_commitment_mut(&mut self) -> Fr {
        match self {
            Node::Branch(branch_node) => branch_node.hash_commitment_mut(),
            Node::Leaf(leaf_node) => leaf_node.hash_commitment_mut(),
            Node::Commitment(c) => *c,
        }
    }
}

impl Node {
    pub fn new() -> Self {
        Self::Branch(BranchNode::new())
    }

    pub fn check(&self, commitment: &Fr) -> Result<()> {
        if &self.hash_commitment() == commitment {
            Ok(())
        } else {
            Err(anyhow!(
                "Node's commitment {:?} doesn't match expected {commitment:?}",
                self.hash_commitment()
            ))
        }
    }

    pub fn get(&mut self, key: TrieKey, db: &Db) -> Result<Option<TrieValue>> {
        let mut depth = 0;
        let mut node = self;
        loop {
            match node {
                Node::Branch(branch_node) => {
                    node = match branch_node.get_mut(key[depth]) {
                        Some(node) => node,
                        None => return Ok(None),
                    };
                    depth += 1;
                }
                Node::Leaf(leaf_node) => {
                    if leaf_node.stem() == &key.stem() {
                        return Ok(leaf_node.get(key.last()).cloned());
                    } else {
                        return Ok(None);
                    }
                }
                Node::Commitment(c) => {
                    let Some(bytes) = db.read(c)? else {
                        bail!("Node {c:?} not found in db")
                    };
                    let new_node = Node::from_ssz_bytes(&bytes)
                        .map_err(|e| anyhow!("Error decoding node: {e:?}"))?;
                    new_node.check(c)?;
                    *node = new_node;
                }
            };
        }
    }

    pub fn insert(&mut self, depth: usize, key: TrieKey, value: TrieValue, db: &Db) -> Result<()> {
        match self {
            Node::Branch(branch_node) => branch_node.insert(depth, key, value, db)?,
            Node::Leaf(leaf_node) => {
                if leaf_node.stem() == &key.stem() {
                    leaf_node.set(key.last(), value);
                } else {
                    let mut branch_node = BranchNode::new();
                    branch_node.set(
                        leaf_node.stem()[depth],
                        Node::Leaf(mem::replace(
                            leaf_node,
                            LeafNode::new(TrieKey(B256::ZERO).stem()),
                        )),
                    );
                    branch_node.insert(depth, key, value, db)?;

                    *self = Node::Branch(branch_node)
                }
            }
            Node::Commitment(c) => {
                let Some(bytes) = db.read(c)? else {
                    bail!("Node {c:?} not found in db")
                };
                let mut node = Node::from_ssz_bytes(&bytes)
                    .map_err(|e| anyhow!("Error decoding node: {e:?}"))?;
                node.insert(depth, key, value, db)?;
                node.check(c)?;
                *self = node;
            }
        };
        Ok(())
    }

    pub fn write_and_commit(&mut self, db: &mut Db) -> Result<Fr> {
        match self {
            Node::Branch(branch_node) => {
                branch_node.write_and_commit(db)?;
                let c = branch_node.hash_commitment_mut();
                db.write(c, self.as_ssz_bytes())?;
                *self = Node::Commitment(c);
                Ok(c)
            }
            Node::Leaf(leaf_node) => {
                let c = leaf_node.hash_commitment_mut();
                db.write(c, self.as_ssz_bytes())?;
                *self = Node::Commitment(c);
                Ok(c)
            }
            _ => Ok(self.hash_commitment_mut()),
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

const SSZ_TAG_BRANCH: u8 = 1;
const SSZ_TAG_LEAF: u8 = 0;

impl Encode for Node {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        match self {
            Node::Branch(branch_node) => {
                buf.push(SSZ_TAG_BRANCH);
                branch_node.ssz_append(buf);
            }
            Node::Leaf(leaf_node) => {
                buf.push(SSZ_TAG_LEAF);
                leaf_node.ssz_append(buf);
            }
            Node::Commitment(_) => panic!("Can't encode Commitment node"),
        }
    }

    fn ssz_bytes_len(&self) -> usize {
        match self {
            Node::Branch(branch_node) => 1 + branch_node.ssz_bytes_len(),
            Node::Leaf(leaf_node) => 1 + leaf_node.ssz_bytes_len(),
            Node::Commitment(_) => panic!("Can't encode Commitment node"),
        }
    }
}

impl Decode for Node {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let Some((&tag, bytes)) = bytes.split_first() else {
            return Err(ssz::DecodeError::InvalidByteLength {
                len: 0,
                expected: 1,
            });
        };

        match tag {
            SSZ_TAG_BRANCH => BranchNode::from_ssz_bytes(bytes).map(Self::Branch),
            SSZ_TAG_LEAF => LeafNode::from_ssz_bytes(bytes).map(Self::Leaf),
            _ => Err(ssz::DecodeError::UnionSelectorInvalid(tag)),
        }
    }
}
