use alloy_primitives::{keccak256, B256, U256};
use alloy_rlp::{RlpDecodable, RlpEncodable};

#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct AccountState {
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: B256,
    pub code_hash: B256,
}

impl AccountState {
    pub fn new_eoa(balance: U256) -> Self {
        Self {
            nonce: 0,
            balance,
            storage_root: keccak256([alloy_rlp::EMPTY_STRING_CODE]),
            code_hash: keccak256([]),
        }
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self::new_eoa(U256::ZERO)
    }
}
