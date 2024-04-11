use alloy_primitives::{B256, U256};
use derive_more::{Deref, Index};

use banderwagon::Fr;
pub use stem::Stem as TrieStem;

mod committer;
mod constants;
pub mod crs;
pub mod nodes;
mod stem;
pub mod trie;
mod utils;

pub type TrieValue = U256;

type Db = dyn db::Db<Fr, Vec<u8>>;

#[derive(PartialEq, Eq, Clone, Index, Deref)]
pub struct TrieKey(B256);

impl TrieKey {
    pub fn length() -> usize {
        B256::len_bytes()
    }

    pub fn stem(&self) -> stem::Stem {
        self.into()
    }

    pub fn last(&self) -> u8 {
        self[self.len() - 1]
    }
}
