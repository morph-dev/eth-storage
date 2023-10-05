use alloy_primitives::B256;
use alloy_rlp::{length_of_length, Encodable, Header};
use db::Db;

use crate::nibbles::Nibbles;

use super::node::{Node, NodeRef};

#[derive(Default, Clone)]
pub struct BranchNode {
    pub nodes: [NodeRef; 16],
    pub value: Option<Vec<u8>>,
}

impl BranchNode {
    fn calc_payload_length(&self) -> usize {
        let nodes_length = self
            .nodes
            .iter()
            .fold(0, |len, node_ref| len + node_ref.length());
        let value_length = self.value.as_ref().map_or(1, |value| value.length());

        nodes_length + value_length
    }

    pub fn get(&self, path: &Nibbles, db: &dyn Db<B256, Vec<u8>>) -> Option<Vec<u8>> {
        match path.first() {
            None => self.value.clone(),
            Some(nibble) => self.nodes[nibble].clone().get(&path.skip_head(1), db),
        }
    }

    pub fn update(&self, path: Nibbles, value: Vec<u8>, db: &mut dyn Db<B256, Vec<u8>>) -> Node {
        let mut nodes = self.nodes.clone();

        match path.first() {
            None => Node::Branch(BranchNode {
                nodes,
                value: Some(value),
            }),
            Some(nibble) => {
                nodes[nibble] = nodes[nibble].update(path.skip_head(1), value, db);
                Node::Branch(BranchNode {
                    nodes,
                    value: self.value.clone(),
                })
            }
        }
    }
}

impl Encodable for BranchNode {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        let header = Header {
            list: true,
            payload_length: self.calc_payload_length(),
        };
        header.encode(out);
        for node_ref in &self.nodes {
            node_ref.encode(out);
        }
        match &self.value {
            None => [].encode(out),
            Some(v) => v[..].encode(out),
        };
    }

    fn length(&self) -> usize {
        let payload_length = self.calc_payload_length();
        length_of_length(payload_length) + payload_length
    }
}
