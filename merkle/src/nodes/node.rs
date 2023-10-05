use alloy_primitives::{keccak256, B256};
use alloy_rlp::{Decodable, Encodable, EMPTY_STRING_CODE};
use anyhow::{bail, Result};
use bytes::Bytes;
use db::Db;

use crate::nibbles::Nibbles;

use super::{branch::BranchNode, extension::ExtensionNode, leaf::LeafNode};

#[derive(Clone, Default)]
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
                    if !payloads[i].is_empty() {
                        nodes[i] = NodeRef::try_from_bytes(&mut payloads[i].as_ref())?;
                    }
                }
                let value = if payloads[16].is_empty() {
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

#[derive(Clone)]
pub struct NodeRef {
    pub hash: B256,
    pub node: Option<Box<Node>>,
}

impl Default for NodeRef {
    fn default() -> Self {
        NodeRef::from(Node::Nil)
    }
}

impl From<B256> for NodeRef {
    fn from(hash: B256) -> Self {
        NodeRef { hash, node: None }
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
    pub fn get(&mut self, path: &Nibbles, db: &dyn Db<B256, Vec<u8>>) -> Option<Vec<u8>> {
        self.load(db);

        match &self.node {
            None => panic!("Node should be present"),
            Some(node) => match node.as_ref() {
                Node::Nil => None,
                Node::Branch(branch) => branch.get(path, db),
                Node::Extension(extension) => extension.get(path, db),
                Node::Leaf(leaf) => leaf.get(path, db),
            },
        }
    }

    pub fn update(
        &mut self,
        path: Nibbles,
        value: Vec<u8>,
        db: &mut dyn Db<B256, Vec<u8>>,
    ) -> NodeRef {
        self.load(db);

        match &self.node {
            None => panic!("Node should be present"),
            Some(node) => {
                let new_node = match node.as_ref() {
                    Node::Nil => Node::Leaf(LeafNode { path, value }),
                    Node::Branch(branch) => branch.update(path, value, db),
                    Node::Extension(extension) => extension.update(path, value, db),
                    Node::Leaf(leaf) => leaf.update(path, value, db),
                };
                NodeRef::from(new_node)
            }
        }
    }

    pub fn save(&self, db: &mut dyn Db<B256, Vec<u8>>) -> Result<()> {
        match &self.node {
            None => bail!("Trying to save unknown node"),
            Some(node) => Ok(db.write(&self.hash, &alloy_rlp::encode(node))?),
        }
    }

    pub fn load(&mut self, db: &dyn Db<B256, Vec<u8>>) {
        if let Err(err) = self.try_load(db) {
            panic!("{}", err);
        }
    }

    pub fn try_load(&mut self, db: &dyn Db<B256, Vec<u8>>) -> Result<()> {
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

    pub fn try_from_bytes(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        match buf.len() {
            0 => Ok(NodeRef::default()),
            1..=31 => {
                let inner_node = Node::decode(buf)?;
                Ok(Self {
                    hash: inner_node.hash(),
                    node: Some(Box::from(inner_node)),
                })
            }
            32 => Ok(Self {
                hash: B256::try_from(*buf)
                    .map_err(|_| alloy_rlp::Error::Custom("unknown error converting to B256"))?,
                node: None,
            }),
            _ => Err(alloy_rlp::Error::UnexpectedLength),
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
