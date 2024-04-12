use alloy_primitives::{Address, B128, B256, U256};
use banderwagon::Fr;

use crate::{
    committer::DEFAULT_COMMITER,
    constants::{
        BALANCE_LEAF_KEY, CODE_KECCAK_LEAF_KEY, CODE_OFFSET, CODE_SIZE_LEAF_KEY,
        HEADER_STORAGE_OFFSET, MAIN_STORAGE_OFFSET, NONCE_LEAF_KEY, VERKLE_NODE_WIDTH_U256,
        VERSION_LEAF_KEY,
    },
    stem::Stem,
    utils::fr_to_b256,
    TrieKey, TrieValue,
};

pub struct AccountStorageLayout {
    pub address: Address,
    base_storage_stem: Stem,
}

impl AccountStorageLayout {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            base_storage_stem: tree_key(address, U256::ZERO, 0).into(),
        }
    }

    pub fn version_key(&self) -> TrieKey {
        TrieKey::from_stem_and_last_byte(&self.base_storage_stem, VERSION_LEAF_KEY)
    }

    pub fn balance_key(&self) -> TrieKey {
        TrieKey::from_stem_and_last_byte(&self.base_storage_stem, BALANCE_LEAF_KEY)
    }

    pub fn nonce_key(&self) -> TrieKey {
        TrieKey::from_stem_and_last_byte(&self.base_storage_stem, NONCE_LEAF_KEY)
    }

    pub fn code_hash_key(&self) -> TrieKey {
        TrieKey::from_stem_and_last_byte(&self.base_storage_stem, CODE_KECCAK_LEAF_KEY)
    }

    pub fn code_size_key(&self) -> TrieKey {
        TrieKey::from_stem_and_last_byte(&self.base_storage_stem, CODE_SIZE_LEAF_KEY)
    }

    pub fn storage_slot_key(&self, storage_key: U256) -> TrieKey {
        let pos = if storage_key < CODE_OFFSET - HEADER_STORAGE_OFFSET {
            HEADER_STORAGE_OFFSET + storage_key
        } else {
            MAIN_STORAGE_OFFSET + storage_key
        };
        tree_key(
            self.address,
            pos / VERKLE_NODE_WIDTH_U256,
            (pos % VERKLE_NODE_WIDTH_U256).byte(0),
        )
    }

    pub fn code_key(&self, chunk_id: usize) -> TrieKey {
        let pos = CODE_OFFSET + U256::from(chunk_id);
        tree_key(
            self.address,
            pos / VERKLE_NODE_WIDTH_U256,
            (pos % VERKLE_NODE_WIDTH_U256).byte(0),
        )
    }

    pub fn chunkify_code(&self, code: &[u8]) -> Vec<(TrieKey, TrieValue)> {
        const PUSH_OFFSET: u8 = 95;
        const PUSH1: u8 = PUSH_OFFSET + 1;
        const PUSH32: u8 = PUSH_OFFSET + 32;

        let mut remaining_push_data = 0u8;
        let mut result = vec![];
        for (chunk_id, chunk) in code.chunks(31).enumerate() {
            let mut value = Vec::with_capacity(32);
            value.push(remaining_push_data.min(31));
            value.extend(chunk);
            value.resize(32, 0);
            result.push((self.code_key(chunk_id), TrieValue::from_le_slice(&value)));

            for chunk_byte in chunk {
                if remaining_push_data > 0 {
                    remaining_push_data -= 1;
                } else if (PUSH1..=PUSH32).contains(chunk_byte) {
                    remaining_push_data = chunk_byte - PUSH_OFFSET;
                }
            }
        }
        result
    }
}

fn tree_key(address: Address, tree_index: U256, sub_index: u8) -> TrieKey {
    let address_bytes = *B256::left_padding_from(address.as_slice());
    let tree_index_bytes = tree_index.to_le_bytes::<32>();

    let scalars = [
        2u128 + 256 * 64,
        u128::from_le_bytes(B128::from_slice(&address_bytes[..16]).0),
        u128::from_le_bytes(B128::from_slice(&address_bytes[16..]).0),
        u128::from_le_bytes(B128::from_slice(&tree_index_bytes[..16]).0),
        u128::from_le_bytes(B128::from_slice(&tree_index_bytes[16..]).0),
    ]
    .map(Fr::from);
    let commitment = DEFAULT_COMMITER.commit_lagrange(&scalars);
    let hash_commitment = commitment.map_to_scalar_field();

    let mut key = fr_to_b256(&hash_commitment);
    key[31] = sub_index;
    key.into()
}
