use crate::nibbles::{Nibble, Nibbles};

use super::{BranchNode, LeafNode, Node};

pub struct ExtensionNode {
    pub prefix: Nibbles,
    pub node: Node,
}

impl ExtensionNode {
    pub fn new(prefix: Nibbles, node: Node) -> Self {
        if prefix.is_empty() {
            panic!("Extension node can't have empty prefix")
        }
        Self { prefix, node }
    }

    pub(crate) fn update(mut self, path: &[Nibble], value: Vec<u8>) -> Node {
        let common_path_prefix = self.prefix.common_prefix(path);

        let mut branch_node = BranchNode::default();

        let (_common, remaining_path) = self.prefix.split_at(common_path_prefix);
        match remaining_path.split_first() {
            Some((&first, remaining_path)) => {
                if remaining_path.is_empty() {
                    branch_node[*first as usize] = self.node
                } else {
                    self.prefix = Nibbles::from_slice(remaining_path);
                    branch_node[*first as usize] = Node::Extension(self.into())
                }
            }
            None => panic!("Can't update extension node"),
        }

        let (_common, remaining_path) = path.split_at(common_path_prefix);
        match remaining_path.split_first() {
            Some((first, remaining_path)) => {
                branch_node[**first as usize] =
                    Node::Leaf(LeafNode::new(Nibbles::from_slice(remaining_path), value))
            }
            None => branch_node.value = value,
        }

        let branch_node = Node::Branch(branch_node.into());

        if common_path_prefix == 0 {
            branch_node
        } else {
            Node::Extension(
                ExtensionNode::new(
                    Nibbles::from_slice(&path[..common_path_prefix]),
                    branch_node,
                )
                .into(),
            )
        }
    }
}
