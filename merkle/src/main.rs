use anyhow::Result;
use merkle::{
    history::{Deposit, HistoricalDeposits},
    AccountState, MerklePatriciaTrie,
};

fn main() -> Result<()> {
    let history = HistoricalDeposits::try_read_history_file()?;

    let mut mpt = MerklePatriciaTrie::default();

    println!("Creating state trie");

    let block = &history.blocks[0];
    for Deposit(address, balance) in &block.deposits {
        mpt.set_account(*address, &AccountState::new(balance));
        println!("Writing {address}");
    }
    println!("State trie created: {}", mpt.get_hash());

    for Deposit(address, balance) in &block.deposits {
        let account_state = mpt.get_account(address).map(|state| state.balance);
        assert_eq!(account_state, Some(*balance));
    }

    Ok(())
}
