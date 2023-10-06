use alloy_primitives::{keccak256, Address, B256};
use alloy_rlp::Decodable;
use db::{memory_db::MemoryDb, Db};

use crate::{
    nibbles::Nibbles,
    nodes::node::{Node, NodeRef},
    AccountState,
};

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

    pub fn debug_string(&self) -> String {
        Self::node_debug_string(&self.root, String::from("\n"))
    }

    fn node_debug_string(node_ref: &NodeRef, padding: String) -> String {
        let hash = node_ref.hash;
        let Some(node) = &node_ref.node else {
            return format!("{padding}{hash} -> None");
        };
        match node.as_ref() {
            Node::Nil => format!("{padding}  Nil"),
            Node::Leaf(leaf) => format!(
                "{padding}L {hash} -> {:x?} e:{:?} {:02X?}",
                leaf.path.to_vec(),
                String::from_utf8(leaf.value.clone()),
                alloy_rlp::encode(leaf),
            ),
            Node::Extension(extension) => format!(
                "{padding}E {hash} -> {:x?} e:{:02X?}{}",
                extension.path.to_vec(),
                alloy_rlp::encode(extension),
                Self::node_debug_string(&extension.node, padding.clone() + "  "),
            ),
            Node::Branch(branch) => {
                format!("{padding}B {hash} e:{:02X?}", alloy_rlp::encode(branch))
                    + branch
                        .nodes
                        .iter()
                        .map(|node| Self::node_debug_string(node, padding.clone() + "  "))
                        .collect::<Vec<String>>()
                        .join("")
                        .as_ref()
                    + &format!("{padding}  value = {:?}", branch.value)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::*;
    use alloy_primitives::hex;
    use proptest::collection::{btree_set, vec};
    use proptest::prelude::*;

    #[test]
    fn compute_hash() {
        let mut tree = MerklePatriciaTrie::default();

        tree.set_raw(b"first", b"value".to_vec());
        tree.set_raw(b"second", b"value".to_vec());

        assert_eq!(
            tree.get_hash().as_slice(),
            hex!("f7537e7f4b313c426440b7fface6bff76f51b3eb0d127356efbe6f2b3c891501"),
        );
    }

    #[test]
    fn compute_hash_long() {
        let mut tree = MerklePatriciaTrie::default();

        tree.set_raw(b"first", b"value".to_vec());
        tree.set_raw(b"second", b"value".to_vec());
        tree.set_raw(b"third", b"value".to_vec());
        tree.set_raw(b"fourth", b"value".to_vec());

        assert_eq!(
            tree.get_hash().as_slice(),
            hex!("e2ff76eca34a96b68e6871c74f2a5d9db58e59f82073276866fdd25e560cedea"),
        );
    }

    #[test]
    fn get_inserted() {
        let mut tree = MerklePatriciaTrie::default();

        tree.set_raw(b"first", b"value".to_vec());
        tree.set_raw(b"second", b"value".to_vec());

        let first = tree.get_raw(&&b"first"[..]);
        assert!(first.is_some());
        let second = tree.get_raw(&&b"second"[..]);
        assert!(second.is_some());
    }

    #[test]
    fn get_inserted_zero() {
        let mut tree = MerklePatriciaTrie::default();

        tree.set_raw(&[0x0], b"value".to_vec());
        let first = tree.get_raw(&&[0x0][..]);
        assert!(first.is_some());
    }

    proptest! {
        #[test]
        fn proptest_get_inserted(path in vec(any::<u8>(), 1..100), value in vec(any::<u8>(), 1..100)) {
            let mut tree = MerklePatriciaTrie::default();

            tree.set_raw(&path, value.clone());
            let item = tree.get_raw(&path);
            prop_assert!(item.is_some());
            let item = item.unwrap();
            prop_assert_eq!(item, value);
        }

        #[test]
        fn proptest_get_inserted_multiple(paths in btree_set(vec(any::<u8>(), 1..100), 1..100)) {
            let mut tree = MerklePatriciaTrie::default();

            let paths: Vec<Vec<u8>> = paths.into_iter().collect();
            let values = paths.clone();

            for (path, value) in paths.iter().zip(values.iter()) {
                tree.set_raw(&path, value.clone());
            }

            for (path, value) in paths.iter().zip(values.iter()) {
                let item = tree.get_raw(path);
                prop_assert!(item.is_some());
                prop_assert_eq!(&item.unwrap(), value);
            }
        }
    }

    #[test]
    // cc 27153906dcbc63f2c7af31f8d0f600cd44bddd133d806d251a8a4125fff8b082 # shrinks to paths = [[16], [16, 0]], values = [[0], [0]]
    fn proptest_regression_27153906dcbc63f2c7af31f8d0f600cd44bddd133d806d251a8a4125fff8b082() {
        let mut tree = MerklePatriciaTrie::default();
        tree.set_raw(&[16], vec![0]);
        tree.set_raw(&[16, 0], vec![0]);

        let item = tree.get_raw(&vec![16]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[0]);

        let item = tree.get_raw(&vec![16, 0]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[0]);
    }

    #[test]
    // cc 1b641284519306a352e730a589e07098e76c8a433103b50b3d82422f8d552328 # shrinks to paths = {[1, 0], [0, 0]}
    fn proptest_regression_1b641284519306a352e730a589e07098e76c8a433103b50b3d82422f8d552328() {
        let mut tree = MerklePatriciaTrie::default();
        tree.set_raw(&[0, 0], vec![0, 0]);
        tree.set_raw(&[1, 0], vec![1, 0]);

        let item = tree.get_raw(&vec![1, 0]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[1, 0]);

        let item = tree.get_raw(&vec![0, 0]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[0, 0]);
    }

    #[test]
    fn proptest_regression_247af0efadcb3a37ebb8f9e3258dc2096d295201a7c634a5470b2f17385417e1() {
        let mut tree = MerklePatriciaTrie::default();

        tree.set_raw(&[26, 192, 44, 251], vec![26, 192, 44, 251]);
        tree.set_raw(
            &[195, 132, 220, 124, 112, 201, 70, 128, 235],
            vec![195, 132, 220, 124, 112, 201, 70, 128, 235],
        );
        tree.set_raw(&[126, 138, 25, 245, 146], vec![126, 138, 25, 245, 146]);
        tree.set_raw(
            &[129, 176, 66, 2, 150, 151, 180, 60, 124],
            vec![129, 176, 66, 2, 150, 151, 180, 60, 124],
        );
        tree.set_raw(&[138, 101, 157], vec![138, 101, 157]);

        let item = tree.get_raw(&vec![26, 192, 44, 251]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[26, 192, 44, 251]);

        let item = tree.get_raw(&vec![195, 132, 220, 124, 112, 201, 70, 128, 235]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[195, 132, 220, 124, 112, 201, 70, 128, 235]);

        let item = tree.get_raw(&vec![126, 138, 25, 245, 146]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[126, 138, 25, 245, 146]);

        let item = tree.get_raw(&vec![129, 176, 66, 2, 150, 151, 180, 60, 124]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[129, 176, 66, 2, 150, 151, 180, 60, 124]);

        let item = tree.get_raw(&vec![138, 101, 157]);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), &[138, 101, 157]);
    }

    fn insert_vecs(tree: &mut MerklePatriciaTrie, vecs: &Vec<Vec<u8>>) {
        for x in vecs {
            tree.set_raw(x, x.clone());
        }
    }

    fn check_vecs(tree: &mut MerklePatriciaTrie, vecs: &Vec<Vec<u8>>) {
        for x in vecs {
            let item = tree.get_raw(x);
            assert!(item.is_some());
            assert_eq!(item.unwrap(), *x);
        }
    }

    #[test]
    fn proptest_regression_3a00543dc8638a854e0e97892c72c1afb55362b9a16f7f32f0b88e6c87c77a4d() {
        let vecs = vec![
            vec![52, 53, 143, 52, 206, 112],
            vec![14, 183, 34, 39, 113],
            vec![55, 5],
            vec![134, 123, 19],
            vec![0, 59, 240, 89, 83, 167],
            vec![22, 41],
            vec![13, 166, 159, 101, 90, 234, 91],
            vec![31, 180, 161, 122, 115, 51, 37, 61, 101],
            vec![208, 192, 4, 12, 163, 254, 129, 206, 109],
        ];

        let mut tree = MerklePatriciaTrie::default();

        insert_vecs(&mut tree, &vecs);
        check_vecs(&mut tree, &vecs);
    }

    #[test]
    fn proptest_regression_72044483941df7c265fa4a9635fd6c235f7790f35d878277fea7955387e59fea() {
        let mut tree = MerklePatriciaTrie::default();

        tree.set_raw(&[0x00], vec![0x00]);
        tree.set_raw(&[0xC8], vec![0xC8]);
        tree.set_raw(&[0xC8, 0x00], vec![0xC8, 0x00]);

        assert_eq!(tree.get_raw(&vec![0x00]), Some(vec![0x00]));
        assert_eq!(tree.get_raw(&vec![0xC8]), Some(vec![0xC8]));
        assert_eq!(tree.get_raw(&vec![0xC8, 0x00]), Some(vec![0xC8, 0x00]));
    }

    #[test]
    fn proptest_regression_4f3f0c44fdba16d943c33475dc4fa4431123ca274d17e3529dc7aa778de5655b() {
        let mut tree = MerklePatriciaTrie::default();

        tree.set_raw(&[0x00], vec![0x00]);
        tree.set_raw(&[0x01], vec![0x01]);
        tree.set_raw(&[0x10], vec![0x10]);
        tree.set_raw(&[0x19], vec![0x19]);
        tree.set_raw(&[0x19, 0x00], vec![0x19, 0x00]);
        tree.set_raw(&[0x1A], vec![0x1A]);

        assert_eq!(tree.get_raw(&vec![0x00]), Some(vec![0x00]));
        assert_eq!(tree.get_raw(&vec![0x01]), Some(vec![0x01]));
        assert_eq!(tree.get_raw(&vec![0x10]), Some(vec![0x10]));
        assert_eq!(tree.get_raw(&vec![0x19]), Some(vec![0x19]));
        assert_eq!(tree.get_raw(&vec![0x19, 0x00]), Some(vec![0x19, 0x00]));
        assert_eq!(tree.get_raw(&vec![0x1A]), Some(vec![0x1A]));
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

    proptest! {
        #[test]
        fn proptest_compare_hashes_simple(path in vec(any::<u8>(), 1..32), value in vec(any::<u8>(), 1..100)) {
            expect_hash(vec![(path, value)])?;
        }

        #[test]
        fn proptest_compare_hashes_multiple(data in btree_set((vec(any::<u8>(), 1..32), vec(any::<u8>(), 1..100)), 1..100)) {
            expect_hash(data.into_iter().collect())?;
        }
    }

    fn expect_hash(data: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), TestCaseError> {
        prop_assert_eq!(
            compute_hash_cita_trie(data.clone()),
            compute_hash_ours(data)
        );
        Ok(())
    }

    fn compute_hash_ours(data: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<u8> {
        let mut tree = MerklePatriciaTrie::default();

        for (path, val) in data {
            tree.set_raw(&path, val);
        }

        tree.get_hash().as_slice().to_vec()
    }

    fn compute_hash_cita_trie(data: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<u8> {
        use cita_trie::{MemoryDB, PatriciaTrie, Trie};
        use hasher::HasherKeccak;

        let memdb = Arc::new(MemoryDB::new(true));
        let hasher = Arc::new(HasherKeccak::new());

        let mut trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));

        for (path, value) in data {
            trie.insert(path.to_vec(), value.to_vec()).unwrap();
        }

        trie.root().unwrap()
    }

    #[test]
    fn proptest_regression_0_0_1_0000000000() {
        expect_hash(vec![
            (vec![0], vec![0]),
            (vec![1], vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        ])
        .unwrap();
    }

    #[test]
    fn proptest_regression_2e_0_2e00_0() {
        expect_hash(vec![(vec![0x00], vec![0xA]), (vec![0x00, 00], vec![0xB])]).unwrap();
    }
}
