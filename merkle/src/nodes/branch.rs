use derive_more::{Index, IndexMut};

use super::Node;

#[derive(Default, Index, IndexMut)]
pub struct BranchNode {
    #[index]
    #[index_mut]
    pub children: [Node; 16],
    pub value: Vec<u8>,
}

impl BranchNode {
    pub fn new_with_value(value: Vec<u8>) -> Self {
        Self {
            children: Default::default(),
            value,
        }
    }

    pub fn new_with_child(index: usize, child: Node) -> Self {
        let mut branch_node = Self::default();
        branch_node[index] = child;
        branch_node
    }
}
