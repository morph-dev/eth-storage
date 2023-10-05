use alloy_primitives::B256;
use db::{memory_db::MemoryDb, Db};
use nibbles::Nibbles;
use nodes::node::NodeRef;

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

    pub fn insert_raw(&mut self, path: Vec<u8>, value: Vec<u8>) {
        self.root.update(Nibbles::unpack(path), value, &mut self.db);
    }

    pub fn get_raw(&mut self, path: &Vec<u8>) -> Option<Vec<u8>> {
        self.root.get(&Nibbles::unpack(path), self.db.as_ref())
    }
}
