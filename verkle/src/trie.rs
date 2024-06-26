use alloy_primitives::{keccak256, Address, B256, U256};
use anyhow::Result;
use banderwagon::Element;

use crate::{
    nodes::{CommitmentNode, Node},
    storage::AccountStorageLayout,
    utils::{b256_to_element, element_to_b256},
    Db, TrieKey, TrieValue,
};

pub struct Trie {
    root: Node,
    db: Box<Db>,
}

impl Trie {
    pub fn new(db: Box<Db>) -> Self {
        Self {
            root: Node::new(),
            db,
        }
    }

    pub fn new_with_root(root: B256, db: Box<Db>) -> Self {
        Self {
            root: Node::Commitment(CommitmentNode::new(b256_to_element(&root))),
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

    pub fn root_commitment(&mut self) -> Result<Element> {
        self.root.write_and_commit(self.db.as_mut())
    }

    pub fn root(&mut self) -> Result<B256> {
        Ok(element_to_b256(&self.root_commitment()?))
    }

    pub fn create_eoa(&mut self, address: Address, balance: U256, nonce: u64) -> Result<()> {
        let storage = AccountStorageLayout::new(address);
        self.insert(storage.version_key(), TrieValue::ZERO)?;
        self.insert(storage.balance_key(), balance)?;
        self.insert(storage.nonce_key(), TrieValue::from(nonce))?;
        self.insert(
            storage.code_hash_key(),
            TrieValue::from_le_bytes(keccak256([]).0),
        )?;
        Ok(())
    }

    pub fn create_sc(
        &mut self,
        address: Address,
        balance: U256,
        nonce: u64,
        code: Vec<u8>,
    ) -> Result<()> {
        let storage = AccountStorageLayout::new(address);
        self.insert(storage.version_key(), TrieValue::ZERO)?;
        self.insert(storage.balance_key(), balance)?;
        self.insert(storage.nonce_key(), TrieValue::from(nonce))?;
        self.insert(
            storage.code_hash_key(),
            TrieValue::from_le_bytes(keccak256(&code).0),
        )?;
        self.insert(storage.code_size_key(), TrieValue::from(code.len()))?;
        for (chunk_key, chunk_value) in storage.chunkify_code(&code) {
            self.insert(chunk_key, chunk_value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::U256;
    use anyhow::Result;
    use ark_ff::UniformRand;
    use claims::{assert_none, assert_some_eq};
    use db::memory_db::MemoryDb;
    use rand::{rngs::StdRng, SeedableRng};
    use rstest::rstest;

    use crate::utils::fr_to_b256;

    use super::*;

    fn init() -> Trie {
        Trie::new(Box::new(MemoryDb::new()))
    }

    #[test]
    fn empty() -> Result<()> {
        let mut trie = init();
        assert_eq!(trie.root_commitment()?, Element::zero());
        Ok(())
    }

    #[test]
    fn insert_key0_value0() -> Result<()> {
        let mut trie = init();
        let key = TrieKey::new(B256::ZERO);
        let value = TrieValue::ZERO;

        assert_none!(trie.get(key)?);

        trie.insert(key, value)?;
        assert_some_eq!(trie.get(key)?, value);

        assert_eq!(
            fr_to_b256(&trie.root_commitment()?.map_to_scalar_field()),
            B256::from_str("0xff00a9f3f2d4f58fc23bceebf6b2310419ceac2c30445e2f374e571487715015")?,
        );
        assert_some_eq!(trie.get(key)?, value);

        Ok(())
    }

    #[test]
    fn insert_key1_value1() -> Result<()> {
        let mut trie = init();
        let key = TrieKey::new(U256::from(1).into());
        let value = TrieValue::from(1);

        assert_none!(trie.get(key)?);

        trie.insert(key, value)?;
        assert_some_eq!(trie.get(key)?, value);

        assert_eq!(
            fr_to_b256(&trie.root_commitment()?.map_to_scalar_field()),
            B256::from_str("0x11b55d77cefcb0b1903d6156f3011511a81ec0c838a03a074eba374545b00a06")?,
        );
        assert_some_eq!(trie.get(key)?, value);

        Ok(())
    }

    #[test]
    fn insert_keys_0_1() -> Result<()> {
        let mut trie = init();

        let key0 = TrieKey::new(B256::ZERO);
        let value0 = TrieValue::ZERO;
        let key1 = TrieKey::new(U256::from(1).into());
        let value1 = TrieValue::from(1);

        trie.insert(key0, value0)?;
        trie.insert(key1, value1)?;
        assert_some_eq!(trie.get(key0)?, value0);
        assert_some_eq!(trie.get(key1)?, value1);

        trie.root()?;
        assert_some_eq!(trie.get(key0)?, value0);
        assert_some_eq!(trie.get(key1)?, value1);

        Ok(())
    }

    #[test]
    fn insert_keys_0_max() -> Result<()> {
        let mut trie = init();

        let key0 = TrieKey::new(B256::ZERO);
        let value0 = TrieValue::ZERO;
        let key_max = TrieKey::new(U256::MAX.into());
        let value_max = TrieValue::MAX;

        trie.insert(key0, value0)?;
        trie.insert(key_max, value_max)?;
        assert_some_eq!(trie.get(key0)?, value0);
        assert_some_eq!(trie.get(key_max)?, value_max);

        trie.root()?;
        assert_some_eq!(trie.get(key0)?, value0);
        assert_some_eq!(trie.get(key_max)?, value_max);

        Ok(())
    }

    #[rstest]
    #[case(12345, 10)]
    #[case(12345, 100)]
    #[case(12345, 1000)]
    fn insert_random(#[case] seed: u64, #[case] count: usize) -> Result<()> {
        let mut trie = init();

        let mut key_values = vec![];

        let mut rng = StdRng::seed_from_u64(seed);

        while key_values.len() < count {
            let key = TrieKey::new(B256::random_with(&mut rng));
            let value = U256::rand(&mut rng);
            key_values.push((key, value));
            trie.insert(key, value)?;
        }

        for (key, value) in &key_values {
            assert_some_eq!(trie.get(*key)?, *value);
        }

        trie.root()?;
        for (key, value) in key_values {
            assert_some_eq!(trie.get(key)?, value);
        }

        Ok(())
    }
}
