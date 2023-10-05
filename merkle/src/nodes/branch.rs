use alloy_rlp::{length_of_length, Encodable, Header};

use super::node::NodeRef;

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
