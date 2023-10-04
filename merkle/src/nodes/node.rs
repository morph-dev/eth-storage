use alloy_primitives::B256;

use super::leaf::LeafNode;

pub enum Node {
    Nil,
    Hash(B256),
    Leaf(LeafNode),
}
