pub use account::AccountState;
use alloy_primitives::{keccak256, Address, B256};
use alloy_rlp::Decodable;
use db::{memory_db::MemoryDb, Db};
use nibbles::Nibbles;
use nodes::node::NodeRef;

pub mod account;
pub mod history;
pub mod nibbles;
pub mod nodes;

pub struct MerklePatriciaTrie {
    root: NodeRef,
    db: Box<dyn Db<B256, Vec<u8>>>,
}

impl Default for MerklePatriciaTrie {
    fn default() -> Self {
        Self {
            root: NodeRef::default(),
            db: Box::from(MemoryDb::<B256, Vec<u8>>::default()),
        }
    }
}

impl MerklePatriciaTrie {
    pub fn get_hash(&self) -> B256 {
        self.root.hash
    }

    pub fn set_raw(&mut self, path: &[u8], value: Vec<u8>) {
        self.root = self.root.update(Nibbles::unpack(path), value, &mut self.db);
    }

    pub fn get_raw(&mut self, path: &[u8]) -> Option<Vec<u8>> {
        self.root.get(&Nibbles::unpack(path), self.db.as_ref())
    }

    pub fn set_account(&mut self, address: Address, account: &AccountState) {
        self.set_raw(keccak256(address).as_ref(), alloy_rlp::encode(account));
    }

    pub fn get_account(&mut self, address: &Address) -> Option<AccountState> {
        self.get_raw(keccak256(address).as_ref())
            .map(|encoded| AccountState::decode(&mut encoded.as_ref()))
            .transpose()
            .unwrap_or_else(|e| {
                eprintln!("Error getting account {e}");
                None
            })
    }
}
