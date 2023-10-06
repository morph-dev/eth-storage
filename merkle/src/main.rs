use std::io::{self, Write};

use alloy_primitives::keccak256;
use anyhow::Result;
use merkle::{
    history::{Deposit, HistoricalDeposits},
    MerklePatriciaTrie,
};

fn main() -> Result<()> {
    let history = HistoricalDeposits::try_read_history_file()?;

    let mut mpt = MerklePatriciaTrie::default();

    use cita_trie::{MemoryDB, PatriciaTrie, Trie};
    use hasher::HasherKeccak;
    use std::sync::Arc;
    let memdb = Arc::new(MemoryDB::new(true));
    let hasher = Arc::new(HasherKeccak::new());
    let mut trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));

    for block in &history.blocks {
        print!("Processing block: {:4} ...", block.block);
        io::stdout().flush()?;
        for Deposit(address, amount) in &block.deposits {
            let mut account = mpt.get_account(address).unwrap_or_default();
            account.balance += amount;

            mpt.set_account(*address, &account);
            trie.insert(keccak256(address).to_vec(), alloy_rlp::encode(&account))?;
        }

        println!(" state : {}", mpt.get_hash());
        assert_eq!(block.state_root, mpt.get_hash());
        assert_eq!(block.state_root.to_vec(), trie.root()?);
    }

    Ok(())
}
