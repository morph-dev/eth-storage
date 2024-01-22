use alloy_primitives::{keccak256, B256};
use alloy_rlp::{Decodable, Encodable, EMPTY_STRING_CODE};
use anyhow::{bail, Result};
use bytes::Bytes;
use db::Db;

use crate::{nibbles::Nibbles, nodes::decode::RlpStructure};

use super::{branch::BranchNode, extension::ExtensionNode, leaf::LeafNode};

#[derive(Clone, Default, Debug)]
pub enum Node {
    #[default]
    Nil,
    Leaf(LeafNode),
    Extension(ExtensionNode),
    Branch(BranchNode),
}

impl Node {
    pub fn hash(&self) -> B256 {
        let encoded = alloy_rlp::encode(self);
        keccak256(encoded)
    }
}

impl Encodable for Node {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        match &self {
            Node::Nil => [].encode(out),
            Node::Leaf(leaf) => leaf.encode(out),
            Node::Extension(extension) => extension.encode(out),
            Node::Branch(branch) => branch.encode(out),
        }
    }

    fn length(&self) -> usize {
        match &self {
            Node::Nil => alloy_rlp::length_of_length(1),
            Node::Leaf(leaf) => leaf.length(),
            Node::Extension(extension) => extension.length(),
            Node::Branch(branch) => branch.length(),
        }
    }
}

impl Decodable for Node {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        if buf[0] == EMPTY_STRING_CODE {
            return Ok(Node::Nil);
        }

        let RlpStructure::List(payloads) = RlpStructure::decode(buf)? else {
            return Err(alloy_rlp::Error::UnexpectedString);
        };
        match payloads.len() {
            2 => {
                let nibbles = Bytes::decode(&mut payloads[0].as_ref())?;
                let (path, is_leaf) = Nibbles::from_compact(&nibbles);

                let node = if is_leaf {
                    Node::Leaf(LeafNode {
                        path,
                        value: Bytes::decode(&mut payloads[1].as_ref())?.into(),
                    })
                } else {
                    let node_ref = NodeRef::try_from_bytes(&mut payloads[1].as_ref())?;
                    Node::Extension(ExtensionNode {
                        path,
                        node: node_ref,
                    })
                };

                Ok(node)
            }
            17 => {
                let mut nodes: [NodeRef; 16] = <[NodeRef; 16]>::default();
                for i in 0..16 {
                    nodes[i] = NodeRef::try_from_bytes(&mut payloads[i].as_ref())?;
                }
                let value = Bytes::decode(&mut payloads[16].as_ref())?;
                let value = if value.is_empty() {
                    None
                } else {
                    Some(value.into())
                };
                Ok(Node::Branch(BranchNode { nodes, value }))
            }
            _ => Err(alloy_rlp::Error::Custom("Expected length of 2 or 17")),
        }
    }
}

#[derive(Debug)]
pub struct NodeRef {
    pub hash: B256,
    pub node: Option<Box<Node>>,
}

impl Default for NodeRef {
    fn default() -> Self {
        NodeRef {
            hash: Node::Nil.hash(),
            node: Some(Box::from(Node::Nil)),
        }
    }
}

impl Clone for NodeRef {
    fn clone(&self) -> Self {
        let node = match &self.node {
            None => None,
            Some(node) => {
                if node.length() < 32 {
                    Some(node.clone())
                } else {
                    None
                }
            }
        };

        Self {
            hash: self.hash,
            node,
        }
    }
}

impl NodeRef {
    pub fn from(node: Node, db: &mut Box<dyn Db<B256, Vec<u8>>>) -> Self {
        let node_ref = NodeRef {
            hash: node.hash(),
            node: Some(Box::from(node)),
        };
        node_ref
            .save(db)
            .unwrap_or_else(|e| eprintln!("Error writing to db: {e}"));
        node_ref
    }

    pub fn get(&self, path: &Nibbles, db: &dyn Db<B256, Vec<u8>>) -> Option<Vec<u8>> {
        let node = self.load(db);

        match &node {
            Node::Nil => None,
            Node::Branch(branch) => branch.get(path, db),
            Node::Extension(extension) => extension.get(path, db),
            Node::Leaf(leaf) => leaf.get(path, db),
        }
    }

    pub fn update(
        &self,
        path: Nibbles,
        value: Vec<u8>,
        db: &mut Box<dyn Db<B256, Vec<u8>>>,
    ) -> NodeRef {
        let tmp;
        let node = match &self.node {
            None => {
                tmp = self.load(db.as_ref());
                &tmp
            }
            Some(node_ref) => node_ref.as_ref(),
        };
        let new_node = match node {
            Node::Nil => Node::Leaf(LeafNode { path, value }),
            Node::Branch(branch) => branch.update(path, value, db),
            Node::Extension(extension) => extension.update(path, value, db),
            Node::Leaf(leaf) => leaf.update(path, value, db),
        };
        NodeRef::from(new_node, db)
    }

    pub fn save(&self, db: &mut Box<dyn Db<B256, Vec<u8>>>) -> Result<()> {
        match &self.node {
            None => bail!("Trying to save unknown node"),
            Some(node) => Ok(db.write(&self.hash, &alloy_rlp::encode(node))?),
        }
    }

    pub fn load(&self, db: &dyn Db<B256, Vec<u8>>) -> Node {
        self.try_load(db).unwrap()
    }

    pub fn try_load(&self, db: &dyn Db<B256, Vec<u8>>) -> Result<Node> {
        if let Some(node) = &self.node {
            return Ok(node.as_ref().clone());
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
                Ok(node)
            }
        }
    }

    pub fn try_from_bytes(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        match buf.len() {
            1..=31 => {
                let inner_node = Node::decode(buf)?;
                Ok(Self {
                    hash: inner_node.hash(),
                    node: Some(Box::from(inner_node)),
                })
            }
            33 => Ok(Self {
                hash: B256::decode(buf)?,
                node: None,
            }),
            _ => Err(alloy_rlp::Error::UnexpectedLength),
        }
    }
}

impl Encodable for NodeRef {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        let Some(node) = &self.node else {
            self.hash.encode(out);
            return;
        };

        if node.length() < 32 {
            node.encode(out);
        } else {
            self.hash.encode(out);
        }
    }

    fn length(&self) -> usize {
        match self.node.as_ref() {
            None => self.hash.length(),
            Some(node) => {
                let encoded_length = node.length();
                if encoded_length < 32 {
                    encoded_length
                } else {
                    self.hash.length()
                }
            }
        }
    }
}
