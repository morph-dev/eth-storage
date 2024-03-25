use alloy_primitives::B256;
use alloy_rlp::{length_of_length, Encodable};
use db::Db;

use crate::{
    nibbles::Nibbles,
    nodes::{branch::BranchNode, extension::ExtensionNode, node::NodeRef},
};

use super::node::Node;

#[derive(Clone, Debug)]
pub struct LeafNode {
    pub path: Nibbles,
    pub value: Vec<u8>,
}

impl LeafNode {
    pub fn get(&self, path: &Nibbles, _db: &dyn Db<B256, Vec<u8>>) -> Option<Vec<u8>> {
        if self.path[..] == path[..] {
            Some(self.value.clone())
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
        // Path is the same
        if self.path.len() == common_prefix && path.len() == common_prefix {
            return Node::Leaf(LeafNode { path, value });
        }

        // We need a branch at common_prefix position
        let branch = {
            let branch = BranchNode::default();

            let branch = branch.update(self.path.skip_head(common_prefix), self.value.clone(), db);
            let Node::Branch(branch) = branch else {
                panic!("branch was expected")
            };

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

impl Encodable for LeafNode {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        let compact_path: &[u8] = &self.path.to_compact(true);
        let parts = [compact_path, &self.value];

        alloy_rlp::encode_list::<&[u8], [u8]>(&parts, out)
    }

    fn length(&self) -> usize {
        let mut compact_path_length = self.path.compact_len();
        // If compact path is length of 1, we don't need length extra byte for length.
        // Otherwise we do
        if compact_path_length > 1 {
            compact_path_length += 1;
        }

        let value_length = <[u8] as Encodable>::length(&self.value);

        let total_length = compact_path_length + value_length;
        length_of_length(total_length) + total_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_encoding_decoding() {
        for (path, expected) in [
            (vec![], vec![0xc2, 0x20, 0x80]),
            (vec![0xf], vec![0xc2, 0x3f, 0x80]),
            (vec![0xa, 0xb], vec![0xc4, 0x82, 0x20, 0xab, 0x80]),
            (vec![0xa, 0xb, 0xc], vec![0xc4, 0x82, 0x3a, 0xbc, 0x80]),
            (
                vec![0xa, 0xb, 0xc, 0xd],
                vec![0xc5, 0x83, 0x20, 0xab, 0xcd, 0x80],
            ),
        ] {
            verify_encode_decode(path, vec![], expected);
        }
    }

    #[test]
    fn value_encoding_decoding() {
        for (path, value, expected) in [
            (vec![0x0], vec![0x00], vec![0xc2, 0x30, 0x00]),
            (vec![0x0], vec![0x7f], vec![0xc2, 0x30, 0x7f]),
            (vec![0x0], vec![0x80], vec![0xc3, 0x30, 0x81, 0x80]),
            (vec![0x0], vec![0xff], vec![0xc3, 0x30, 0x81, 0xff]),
            (
                vec![0x0],
                vec![0x01, 0x02],
                vec![0xc4, 0x30, 0x82, 0x01, 0x02],
            ),
            (
                vec![0x0],
                vec![0xab, 0xcd, 0xef],
                vec![0xc5, 0x30, 0x83, 0xab, 0xcd, 0xef],
            ),
        ] {
            verify_encode_decode(path, value, expected);
        }
    }

    #[test]
    fn encode_short() {
        for (path, value, expected) in [
            (vec![0x2], vec![], vec![0xc2, 0x32, 0x80]),
            (vec![0x3], vec![0x01], vec![0xc2, 0x33, 0x01]),
            (vec![0x4], vec![0x0a], vec![0xc2, 0x34, 0x0a]),
            (vec![0x5], vec![0x7F], vec![0xc2, 0x35, 0x7F]),
            (vec![0x6], vec![0x80], vec![0xc3, 0x36, 0x81, 0x80]),
            (vec![0xF], vec![0xFF], vec![0xc3, 0x3F, 0x81, 0xFF]),
            (
                vec![0x0, 0x0],
                vec![0x01],
                vec![0xc4, 0x82, 0x20, 0x00, 0x01],
            ),
            (
                vec![0x0, 0x0],
                vec![0x01],
                vec![0xc4, 0x82, 0x20, 0x00, 0x01],
            ),
        ] {
            verify_encode_decode(path, value, expected);
        }
    }

    fn verify_encode_decode(path: Vec<u8>, value: Vec<u8>, encoded: Vec<u8>) {
        let leaf = LeafNode {
            path: Nibbles::from(&path),
            value,
        };

        assert_eq!(leaf.length(), encoded.len());

        let encoded_leaf = alloy_rlp::encode(&leaf);
        assert_eq!(encoded_leaf, encoded);
    }
}
