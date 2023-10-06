use alloy_primitives::{keccak256, B256, U256};
use alloy_rlp::{RlpDecodable, RlpEncodable};

use crate::nodes::node::NodeRef;

#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct AccountState {
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: B256,
    pub code_hash: B256,
}

impl AccountState {
    pub fn new(balance: U256) -> Self {
        AccountState {
            nonce: 0,
            balance,
            storage_root: NodeRef::default().hash,
            code_hash: keccak256([]),
        }
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self::new(U256::ZERO)
    }
}
