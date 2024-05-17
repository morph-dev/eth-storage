use std::mem;

use alloy_primitives::B256;
use anyhow::{anyhow, bail, Result};
use banderwagon::{Element, Fr};
use ssz::{Decode, Encode};

use crate::{Db, TrieKey, TrieValue};

use super::{BranchNode, CommitmentNode, LeafNode};

pub trait NodeTrait {
    fn commitment_write(&mut self) -> Element;

    fn commitment(&self) -> Element;

    fn commitment_hash_write(&mut self) -> Fr {
        self.commitment_write().map_to_scalar_field()
    }

    fn commitment_hash(&self) -> Fr {
        self.commitment().map_to_scalar_field()
    }
}

pub enum Node {
    Branch(BranchNode),
    Leaf(LeafNode),
    Commitment(CommitmentNode),
}

impl NodeTrait for Node {
    fn commitment_write(&mut self) -> Element {
        match self {
            Node::Branch(branch_node) => branch_node.commitment_write(),
            Node::Leaf(leaf_node) => leaf_node.commitment_write(),
            Node::Commitment(commitment_node) => commitment_node.commitment_write(),
        }
    }

    fn commitment(&self) -> Element {
        match self {
            Node::Branch(branch_node) => branch_node.commitment(),
            Node::Leaf(leaf_node) => leaf_node.commitment(),
            Node::Commitment(commitment_node) => commitment_node.commitment(),
        }
    }

    fn commitment_hash_write(&mut self) -> Fr {
        match self {
            Node::Branch(branch_node) => branch_node.commitment_hash_write(),
            Node::Leaf(leaf_node) => leaf_node.commitment_hash_write(),
            Node::Commitment(commitment_node) => commitment_node.commitment_hash_write(),
        }
    }

    fn commitment_hash(&self) -> Fr {
        match self {
            Node::Branch(branch_node) => branch_node.commitment_hash(),
            Node::Leaf(leaf_node) => leaf_node.commitment_hash(),
            Node::Commitment(commitment_node) => commitment_node.commitment_hash(),
        }
    }
}

impl Node {
    pub fn new() -> Self {
        Self::Branch(BranchNode::new())
    }

    pub fn check(&self, commitment: &Element) -> Result<()> {
        if &self.commitment() == commitment {
            Ok(())
        } else {
            Err(anyhow!(
                "Node's commitment {:?} doesn't match expected {commitment:?}",
                self.commitment()
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
                Node::Commitment(commitment_node) => {
                    let Some(bytes) = db.read(&commitment_node.commitment())? else {
                        bail!("Node {:?} not found in db", commitment_node.commitment())
                    };
                    let new_node = Node::from_ssz_bytes(&bytes)
                        .map_err(|e| anyhow!("Error decoding node: {e:?}"))?;
                    new_node.check(&commitment_node.commitment())?;
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
            Node::Commitment(commitment_node) => {
                let Some(bytes) = db.read(&commitment_node.commitment())? else {
                    bail!("Node {:?} not found in db", commitment_node.commitment())
                };
                let mut node = Node::from_ssz_bytes(&bytes)
                    .map_err(|e| anyhow!("Error decoding node: {e:?}"))?;
                node.insert(depth, key, value, db)?;
                node.check(&commitment_node.commitment())?;
                *self = node;
            }
        };
        Ok(())
    }

    pub fn write_and_commit(&mut self, db: &mut Db) -> Result<Element> {
        match self {
            Node::Branch(branch_node) => {
                let c = branch_node.write_and_commit(db)?;
                db.write(c, self.as_ssz_bytes())?;
                *self = Node::Commitment(CommitmentNode::new(c));
                Ok(c)
            }
            Node::Leaf(leaf_node) => {
                let c = leaf_node.commitment_write();
                db.write(c, self.as_ssz_bytes())?;
                *self = Node::Commitment(CommitmentNode::new(c));
                Ok(c)
            }
            Node::Commitment(commitment_node) => Ok(commitment_node.commitment_write()),
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
