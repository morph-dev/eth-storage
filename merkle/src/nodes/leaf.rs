use crate::nibbles::{Nibble, Nibbles};

use super::{BranchNode, ExtensionNode, Node};

pub struct LeafNode {
    pub prefix: Nibbles,
    pub value: Vec<u8>,
}

impl LeafNode {
    pub fn new(prefix: Nibbles, value: Vec<u8>) -> Self {
        Self { prefix, value }
    }

    pub(crate) fn update(mut self, path: &[Nibble], value: Vec<u8>) -> Node {
        if *self.prefix == path {
            self.value = value;
            return Node::Leaf(self);
        }
        let common_path_prefix = self.prefix.common_prefix(path);

        let mut branch_node = BranchNode::default();

        let (_common, remaining_path) = self.prefix.split_at(common_path_prefix);
        match remaining_path.split_first() {
            Some((first, remaining_path)) => {
                branch_node[**first as usize] = Node::Leaf(LeafNode::new(
                    Nibbles::from_slice(remaining_path),
                    self.value,
                ))
            }
            None => branch_node.value = self.value,
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
