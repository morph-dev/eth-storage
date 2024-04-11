use alloy_primitives::B256;
use anyhow::Result;

use crate::{
    nodes::Node,
    utils::{b256_to_fr, fr_to_b256},
    Db, TrieKey, TrieValue,
};

pub struct Trie {
    root: Node,
    db: Box<Db>,
}

impl Trie {
    pub fn new(db: Box<Db>) -> Self {
        Self {
            root: Node::Empty,
            db,
        }
    }

    pub fn new_with_root(root: B256, db: Box<Db>) -> Self {
        Self {
            root: Node::Commitment(b256_to_fr(&root)),
            db,
        }
    }
}

impl Trie {
    pub fn get(&mut self, key: TrieKey) -> Result<Option<TrieValue>> {
        self.root.get(key, self.db.as_ref())
    }

    pub fn insert(&mut self, key: TrieKey, value: TrieValue) -> Result<()> {
        self.root.insert(0, key, value, self.db.as_ref())
    }

    pub fn commit(&mut self) -> Result<B256> {
        Ok(fr_to_b256(&self.root.write_and_commit(self.db.as_mut())?))
    }
}
