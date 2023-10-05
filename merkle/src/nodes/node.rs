use alloy_primitives::{keccak256, B256};
use alloy_rlp::{Decodable, Encodable};
use anyhow::{bail, Result};
use bytes::Bytes;
use db::Db;

use crate::nibbles::Nibbles;

use super::{branch::BranchNode, extension::ExtensionNode, leaf::LeafNode};

pub enum Node {
    Leaf(LeafNode),
    Extension(ExtensionNode),
    Branch(BranchNode),
}

impl Node {
    pub fn hash(&self) -> B256 {
        let encoded = match &self {
            Node::Leaf(leaf) => alloy_rlp::encode(leaf),
            Node::Extension(extension) => alloy_rlp::encode(extension),
            Node::Branch(branch) => alloy_rlp::encode(branch),
        };
        keccak256(encoded)
    }
}

impl Encodable for Node {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        match &self {
            Node::Leaf(leaf) => leaf.encode(out),
            Node::Extension(extension) => extension.encode(out),
            Node::Branch(branch) => branch.encode(out),
        }
    }

    fn length(&self) -> usize {
        match &self {
            Node::Leaf(leaf) => leaf.length(),
            Node::Extension(extension) => extension.length(),
            Node::Branch(branch) => branch.length(),
        }
    }
}

impl Decodable for Node {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let payloads = Vec::<Bytes>::decode(buf)?;
        match payloads.len() {
            2 => {
                let (path, is_leaf) = Nibbles::from_compact(&payloads[0]);

                let node = if is_leaf {
                    Node::Leaf(LeafNode {
                        path,
                        value: Vec::from(payloads[1].as_ref()),
                    })
                } else {
                    let node_ref = NodeRef::try_from_bytes(payloads[1].as_ref())?;
                    Node::Extension(ExtensionNode {
                        path,
                        node: node_ref,
                    })
                };

                Ok(node)
            }
            17 => {
                let mut nodes: [Option<NodeRef>; 16] = [
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None,
                ];
                for i in 0..16 {
                    if payloads[i].len() > 0 {
                        nodes[i] = Some(NodeRef::try_from_bytes(&payloads[i])?);
                    }
                }
                let value = if payloads[16].len() == 0 {
                    None
                } else {
                    Some(Vec::from(payloads[16].as_ref()))
                };
                Ok(Node::Branch(BranchNode { nodes, value }))
            }
            _ => Err(alloy_rlp::Error::Custom("Expected length of 2 or 17")),
        }
    }
}

pub struct NodeRef {
    hash: B256,
    node: Option<Box<Node>>,
}

impl From<B256> for NodeRef {
    fn from(hash: B256) -> Self {
        NodeRef {
            hash: hash,
            node: None,
        }
    }
}

impl From<Node> for NodeRef {
    fn from(node: Node) -> Self {
        NodeRef {
            hash: node.hash(),
            node: Some(Box::from(node)),
        }
    }
}

impl NodeRef {
    pub fn expand(&mut self, db: &dyn Db<B256, Vec<u8>>) -> Result<()> {
        if self.node.is_some() {
            return Ok(());
        }
        let encoded_node = db.read(&self.hash)?;
        match encoded_node {
            None => {
                bail!("node not found in Db");
            }
            Some(encoded_node) => {
                let node = Node::decode(&mut encoded_node.as_ref())?;
                if node.hash() != self.hash {
                    bail!("extracted node's hash doesn't match");
                }
                self.node = Some(Box::from(node));
            }
        };
        Ok(())
    }

    pub fn try_from_bytes(buf: &[u8]) -> alloy_rlp::Result<Self> {
        match buf.len() {
            l if l == 0 || l > 32 => Err(alloy_rlp::Error::UnexpectedLength),
            32 => Ok(Self {
                hash: B256::try_from(buf)
                    .map_err(|_| alloy_rlp::Error::Custom("unknown error converting to B256"))?,
                node: None,
            }),
            _ => {
                let inner_node = Node::decode(&mut buf.as_ref())?;
                Ok(Self {
                    hash: inner_node.hash(),
                    node: Some(Box::from(inner_node)),
                })
            }
        }
    }
}

impl Encodable for NodeRef {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        if self.node.is_none() || self.length() == 32 {
            self.hash.encode(out);
            return;
        }
        self.node.as_ref().unwrap().encode(out);
    }

    fn length(&self) -> usize {
        match self.node.as_ref() {
            None => 1,
            Some(node) => {
                let encoded_length = node.length();
                std::cmp::min(32, encoded_length)
            }
        }
    }
}
