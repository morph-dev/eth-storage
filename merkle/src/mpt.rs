use alloy_primitives::{keccak256, Address, B256};
use alloy_rlp::Decodable;
use anyhow::{bail, Result};
use db::memory_db::MemoryDb;

use crate::{
    account::AccountState,
    nibbles::{Nibble, Nibbles},
    nodes::{Node, NodeTraversalInfo},
    Db,
};

pub struct Mpt {
    root: Node,
    db: Box<Db>,
}

impl Mpt {
    pub fn get_hash(&mut self) -> Result<B256> {
        self.root.write(&mut *self.db)?;
        match &self.root {
            Node::Hash(hash) => Ok(*hash.clone()),
            _ => Ok(keccak256(self.root.write(&mut *self.db)?)),
        }
    }

    pub fn set_raw(&mut self, path: &[Nibble], value: Vec<u8>) -> Result<()> {
        self.root.update(path, value, &*self.db)?;
        Ok(())
    }

    pub fn get_raw(&self, path: &[Nibble]) -> Result<Option<Vec<u8>>> {
        let mut node: Node;
        let mut node_traversal_info = self.root.next_node(path);
        loop {
            match node_traversal_info {
                NodeTraversalInfo::Empty => return Ok(None),
                NodeTraversalInfo::Value(value) => return Ok(Some(value.to_vec())),
                NodeTraversalInfo::NextNode {
                    hash,
                    remaining_path,
                } => {
                    let Some(encoded_node) = self.db.read(&hash)? else {
                        bail!("Node missing from Db: {hash:?}")
                    };
                    node = Node::decode(&mut encoded_node.as_slice())?;
                    node_traversal_info = node.next_node(remaining_path);
                }
            }
        }
    }

    pub fn set_account(&mut self, address: Address, account: &AccountState) -> Result<()> {
        self.set_raw(
            &Nibbles::from_packed(keccak256(address)),
            alloy_rlp::encode(account),
        )
    }

    pub fn get_account(&mut self, address: &Address) -> Result<Option<AccountState>> {
        Ok(self
            .get_raw(&Nibbles::from_packed(keccak256(address)))?
            .map(|encoded| AccountState::decode(&mut encoded.as_slice()))
            .transpose()?)
    }
}

impl Default for Mpt {
    fn default() -> Self {
        Self {
            root: Node::Nil,
            db: Box::new(MemoryDb::new()),
        }
    }
}

#[cfg(test)]
mod test {

    use std::{str::FromStr, sync::Arc};

    use super::*;

    #[test]
    fn compute_hash() -> Result<()> {
        let mut tree = Mpt::default();

        tree.set_raw(&Nibbles::from_packed(b"first"), b"value".to_vec())?;
        tree.set_raw(&Nibbles::from_packed(b"second"), b"value".to_vec())?;

        assert_eq!(
            tree.get_hash()?,
            B256::from_str("f7537e7f4b313c426440b7fface6bff76f51b3eb0d127356efbe6f2b3c891501")?,
        );

        Ok(())
    }

    #[test]
    fn compute_hash_long() -> Result<()> {
        let mut tree = Mpt::default();

        tree.set_raw(&Nibbles::from_packed(b"first"), b"value".to_vec())?;
        tree.set_raw(&Nibbles::from_packed(b"second"), b"value".to_vec())?;
        tree.set_raw(&Nibbles::from_packed(b"third"), b"value".to_vec())?;
        tree.set_raw(&Nibbles::from_packed(b"fourth"), b"value".to_vec())?;

        assert_eq!(
            tree.get_hash()?,
            B256::from_str("e2ff76eca34a96b68e6871c74f2a5d9db58e59f82073276866fdd25e560cedea")?,
        );
        Ok(())
    }

    #[test]
    fn get_inserted() -> Result<()> {
        let mut tree = Mpt::default();

        tree.set_raw(&Nibbles::from_packed(b"first"), b"value".to_vec())?;
        tree.set_raw(&Nibbles::from_packed(b"second"), b"value".to_vec())?;

        tree.get_hash()?;

        let first = tree.get_raw(&Nibbles::from_packed(b"first"))?;
        assert!(first.is_some());
        let second = tree.get_raw(&Nibbles::from_packed(b"second"))?;
        assert!(second.is_some());
        Ok(())
    }

    #[test]
    fn get_inserted_zero() -> Result<()> {
        let mut tree = Mpt::default();
        let nibbles = Nibbles::from_slice([Nibble::try_from(0x0)?]);

        tree.set_raw(&nibbles, b"value".to_vec())?;

        tree.get_hash()?;

        let first = tree.get_raw(&nibbles)?;
        assert!(first.is_some());
        Ok(())
    }

    #[test]
    fn compute_hashes() {
        expect_hash(vec![
            (b"doe".to_vec(), b"reindeer".to_vec()),
            (b"dog".to_vec(), b"puppy".to_vec()),
            (b"dogglesworth".to_vec(), b"cat".to_vec()),
        ])
        .unwrap();
    }

    fn expect_hash(data: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        assert_eq!(
            compute_hash_cita_trie(data.clone())?,
            compute_hash_ours(data)?
        );
        Ok(())
    }

    fn compute_hash_ours(data: Vec<(Vec<u8>, Vec<u8>)>) -> Result<B256> {
        let mut tree = Mpt::default();

        for (path, val) in data {
            tree.set_raw(&Nibbles::from_packed(path), val)?;
        }

        tree.get_hash()
    }

    fn compute_hash_cita_trie(data: Vec<(Vec<u8>, Vec<u8>)>) -> Result<B256> {
        use cita_trie::{MemoryDB, PatriciaTrie, Trie};
        use hasher::HasherKeccak;

        let memdb = Arc::new(MemoryDB::new(true));
        let hasher = Arc::new(HasherKeccak::new());

        let mut trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));

        for (path, value) in data {
            trie.insert(path.to_vec(), value.to_vec()).unwrap();
        }

        trie.root()
            .map(|value| B256::from_slice(&value))
            .map_err(anyhow::Error::new)
    }
}
