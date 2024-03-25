use alloy_primitives::B256;
use alloy_rlp::{length_of_length, Encodable, Header};
use db::Db;

use crate::{nibbles::Nibbles, nodes::branch::BranchNode};

use super::node::{Node, NodeRef};

#[derive(Clone, Debug)]
pub struct ExtensionNode {
    pub path: Nibbles,
    pub node: NodeRef,
}

impl ExtensionNode {
    pub fn get(&self, path: &Nibbles, db: &dyn Db<B256, Vec<u8>>) -> Option<Vec<u8>> {
        let common_prefix = self.path.common_prefix(path);
        if self.path.len() == common_prefix {
            self.node.get(&path.skip_head(common_prefix), db)
        } else {
            None
        }
    }

    pub fn update(
        &self,
        path: Nibbles,
        value: Vec<u8>,
        db: &mut Box<dyn Db<B256, Vec<u8>>>,
    ) -> Node {
        let common_prefix = self.path.common_prefix(&path);

        if self.path.len() == common_prefix {
            return Node::Extension(ExtensionNode {
                path: self.path.clone(),
                node: self.node.update(path.skip_head(common_prefix), value, db),
            });
        }

        // We need a branch at common_prefix position
        let branch = {
            let bridge = if common_prefix + 1 == self.path.len() {
                self.node.clone()
            } else {
                NodeRef::from(
                    Node::Extension(ExtensionNode {
                        path: self.path.skip_head(common_prefix + 1),
                        node: self.node.clone(),
                    }),
                    db,
                )
            };

            let mut branch = BranchNode::default();

            branch.nodes[self.path[common_prefix] as usize] = bridge;

            branch.update(path.skip_head(common_prefix), value, db)
        };

        if common_prefix == 0 {
            branch
        } else {
            let branch_ref = NodeRef::from(branch, db);
            Node::Extension(ExtensionNode {
                path: Nibbles::from(&path[..common_prefix]),
                node: branch_ref,
            })
        }
    }
}

impl Encodable for ExtensionNode {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        let compact_path: &[u8] = &self.path.to_compact(false);

        let header = Header {
            list: true,
            payload_length: compact_path.length() + self.node.length(),
        };
        header.encode(out);

        compact_path.encode(out);
        self.node.encode(out);
    }

    fn length(&self) -> usize {
        let mut compact_path_length = self.path.compact_len();
        // If compact path is length of 1, we don't need length extra byte for length.
        // Otherwise we do
        if compact_path_length > 1 {
            compact_path_length += 1;
        }

        let node_length = self.node.length();

        let total_length = compact_path_length + node_length;
        length_of_length(total_length) + total_length
    }
}
