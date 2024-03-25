use std::{collections::HashMap, fs::File, io::BufReader};

use alloy_primitives::{Address, BlockHash, BlockNumber, B256, U256};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::AccountState;

#[derive(Serialize, Deserialize)]
pub struct Deposit(pub Address, pub U256);

#[derive(Serialize, Deserialize)]
pub struct BlockDeposits {
    pub block: BlockNumber,
    pub hash: BlockHash,
    pub state_root: B256,
    pub deposits: Vec<Deposit>,
}

#[derive(Serialize, Deserialize)]
pub struct HistoricalDeposits {
    pub blocks: Vec<BlockDeposits>,
}

pub struct BlockSummary {
    pub block_number: BlockNumber,
    pub hash: BlockHash,
    pub state_root: B256,
    pub balaces: HashMap<Address, AccountState>,
}

const HISTORY_FILEPATH: &str = "history.json";

impl HistoricalDeposits {
    pub fn try_read_history_file() -> Result<Self> {
        let file = File::open(HISTORY_FILEPATH)?;
        let reader = BufReader::new(file);

        println!("Reading history");
        let history: Self = serde_json::from_reader(reader)?;

        history.check_deposits_history()?;

        Ok(history)
    }

    fn check_deposits_history(&self) -> Result<()> {
        if self
            .blocks
            .iter()
            .enumerate()
            .all(|(i, block_deposits)| block_deposits.block == i as u64)
        {
            Ok(())
        } else {
            Err(anyhow!("invalid block history"))
        }
    }

    pub fn to_balances(&self) -> HashMap<BlockNumber, BlockSummary> {
        let mut result = HashMap::new();

        let mut current: HashMap<Address, AccountState> = HashMap::new();

        for block in &self.blocks {
            for Deposit(address, amount) in &block.deposits {
                let mut state = current.get(address).cloned().unwrap_or_default();
                state.balance += amount;
                current.insert(*address, state);
            }

            result.insert(
                block.block,
                BlockSummary {
                    block_number: block.block,
                    hash: block.hash,
                    state_root: block.state_root,
                    balaces: current.clone(),
                },
            );
        }

        result
    }
}
