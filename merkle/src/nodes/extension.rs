use alloy_rlp::{length_of_length, Encodable, Header};

use crate::nibbles::Nibbles;

use super::node::NodeRef;

pub struct ExtensionNode {
    pub path: Nibbles,
    pub node: NodeRef,
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
