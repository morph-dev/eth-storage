use std::{collections::BTreeSet, mem};

use alloy_primitives::{keccak256, B256};
use alloy_rlp::{Buf, BufMut, Decodable, Encodable, Header};
use anyhow::{bail, Result};

use crate::{
    nibbles::{Nibble, Nibbles},
    Db,
};

use super::{BranchNode, ExtensionNode, HashNode, LeafNode};

#[derive(Default)]
pub enum Node {
    #[default]
    Nil,
    Leaf(LeafNode),
    Extension(Box<ExtensionNode>),
    Branch(Box<BranchNode>),
    Hash(HashNode),
}

/// The result of traversing a node via path
pub enum NodeTraversalInfo<'value, 'path> {
    Empty,
    Value(&'value [u8]),
    NextNode {
        hash: B256,
        remaining_path: &'path [Nibble],
    },
}

#[derive(Default)]
pub struct UpdateNodeInfo {
    pub updated_nodes: BTreeSet<B256>,
}

impl Node {
    pub fn next_node<'me, 'path>(
        &'me self,
        path: &'path [Nibble],
    ) -> NodeTraversalInfo<'me, 'path> {
        match self {
            Node::Nil => NodeTraversalInfo::Empty,
            Node::Leaf(leaf_node) => {
                if *leaf_node.prefix == path {
                    NodeTraversalInfo::Value(&leaf_node.value)
                } else {
                    NodeTraversalInfo::Empty
                }
            }
            Node::Extension(extension_node) => {
                if path.starts_with(&extension_node.prefix) {
                    extension_node
                        .node
                        .next_node(&path[extension_node.prefix.len()..])
                } else {
                    NodeTraversalInfo::Empty
                }
            }
            Node::Branch(branch_node) => match path.split_first() {
                Some((first, remaining_path)) => {
                    branch_node[**first as usize].next_node(remaining_path)
                }
                None => NodeTraversalInfo::Value(&branch_node.value),
            },
            Node::Hash(hash_node) => NodeTraversalInfo::NextNode {
                hash: **hash_node,
                remaining_path: path,
            },
        }
    }

    pub fn update(&mut self, path: &[Nibble], value: Vec<u8>, db: &Db) -> Result<UpdateNodeInfo> {
        match self {
            Node::Nil => {
                *self = Node::Leaf(LeafNode::new(Nibbles::from_slice(path), value));
                Ok(UpdateNodeInfo::default())
            }
            Node::Leaf(leaf_node) => {
                // Replace leaf_node with dummy
                let leaf_node =
                    mem::replace(leaf_node, LeafNode::new(Nibbles::from_iter([]), vec![]));
                *self = leaf_node.update(path, value);
                Ok(UpdateNodeInfo::default())
            }
            Node::Extension(extension_node) => {
                if path.starts_with(&extension_node.prefix) {
                    return extension_node.node.update(
                        &path[extension_node.prefix.len()..],
                        value,
                        db,
                    );
                }
                // Replace extension_node with dummy
                let extension_node = mem::replace(
                    extension_node,
                    ExtensionNode {
                        prefix: Nibbles::from_slice([]),
                        node: Node::Nil,
                    }
                    .into(),
                );
                *self = extension_node.update(path, value);
                Ok(UpdateNodeInfo::default())
            }
            Node::Branch(branch_node) => match path.split_first() {
                Some((first, remaining_path)) => {
                    branch_node[**first as usize].update(remaining_path, value, db)
                }
                None => {
                    branch_node.value = value;
                    Ok(UpdateNodeInfo::default())
                }
            },
            Node::Hash(hash_node) => {
                let hash = **hash_node;
                let Some(encoded_node) = db.read(&hash)? else {
                    bail!("Node with hash {hash:?} not found in db")
                };
                let node = Node::decode(&mut encoded_node.as_slice())?;
                if matches!(node, Node::Hash(_)) {
                    bail!("Decoded node is Hash node. hash: {hash:?}")
                }
                *self = node;
                let mut updated_node_info = self.update(path, value, db)?;
                updated_node_info.updated_nodes.insert(hash);
                Ok(updated_node_info)
            }
        }
    }

    pub fn write(&mut self, db: &mut Db) -> Result<Vec<u8>> {
        let encoded = match self {
            Node::Nil => return Ok(vec![alloy_rlp::EMPTY_STRING_CODE]),
            Node::Hash(hash_node) => return Ok(alloy_rlp::encode(**hash_node)),
            Node::Leaf(leaf_node) => {
                let mut payload = vec![];
                leaf_node
                    .prefix
                    .to_compact(/*is_leaf=*/ true)
                    .as_slice()
                    .encode(&mut payload);
                leaf_node.value.as_slice().encode(&mut payload);

                let mut buf = vec![];
                Header {
                    list: true,
                    payload_length: payload.len(),
                }
                .encode(&mut buf);
                buf.put_slice(&payload);
                buf
            }
            Node::Extension(extension_node) => {
                let mut payload = vec![];
                extension_node
                    .prefix
                    .to_compact(/*is_leaf=*/ false)
                    .as_slice()
                    .encode(&mut payload);
                payload.put_slice(&extension_node.node.write(db)?);

                let mut buf = vec![];
                Header {
                    list: true,
                    payload_length: payload.len(),
                }
                .encode(&mut buf);
                buf.put_slice(&payload);
                buf
            }
            Node::Branch(branch_node) => {
                let mut payload = vec![];
                for i in 0..16 {
                    payload.put_slice(&branch_node[i].write(db)?);
                }
                branch_node.value.as_slice().encode(&mut payload);

                let mut buf = vec![];
                Header {
                    list: true,
                    payload_length: payload.len(),
                }
                .encode(&mut buf);
                buf.put_slice(&payload);
                buf
            }
        };
        if encoded.len() < 32 {
            Ok(encoded)
        } else {
            let hash = keccak256(&encoded);
            db.write(hash, encoded)?;
            *self = Node::Hash(hash.into());
            self.write(db)
        }
    }
}

impl Decodable for Node {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let header = Header::decode(buf)?;
        if header.list {
            let mut item_count = 0;
            let mut buf_copy = &buf[..header.payload_length];
            while buf_copy.has_remaining() {
                let tmp_header = Header::decode(&mut buf_copy)?;
                buf_copy.advance(tmp_header.payload_length);
                item_count += 1;
            }

            match item_count {
                2 => {
                    let (nibbles, is_leaf) =
                        Nibbles::from_compact(Header::decode_bytes(buf, /*is_list=*/ false)?)
                            .map_err(|_| alloy_rlp::Error::Custom("Error decoding nibbles"))?;
                    if is_leaf {
                        let value = Header::decode_bytes(buf, /* is_list= */ false)?.to_vec();
                        Ok(Self::Leaf(LeafNode::new(nibbles, value)))
                    } else {
                        let node = Self::decode(buf)?;
                        Ok(Self::Extension(ExtensionNode::new(nibbles, node).into()))
                    }
                }
                17 => {
                    let mut branch_node = BranchNode::default();
                    for i in 0..branch_node.children.len() {
                        branch_node[i] = Self::decode(buf)?;
                    }
                    branch_node.value = Header::decode_bytes(buf, /* is_list= */ false)?.to_vec();

                    Ok(Self::Branch(branch_node.into()))
                }
                _ => Err(alloy_rlp::Error::Custom(
                    "expected list with length 2 or 17",
                )),
            }
        } else {
            match header.payload_length {
                0 => Ok(Self::Nil),
                32 => {
                    let hash = B256::from_slice(&buf[..header.payload_length]);
                    buf.advance(header.payload_length);
                    Ok(Self::Hash(hash.into()))
                }
                _ => Err(alloy_rlp::Error::UnexpectedLength),
            }
        }
    }
}
