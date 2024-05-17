#[cfg(test)]
mod devnet6 {
    use std::{collections::HashMap, fs::File, io::BufReader, str::FromStr};

    use alloy_primitives::{Address, B256, U256, U64};
    use anyhow::Result;
    use db::memory_db::MemoryDb;
    use serde::Deserialize;
    use serde_json::Value;
    use verkle::{storage::AccountStorageLayout, Trie};

    const GENESIS_FILEPATH: &str = "assets/devnet6_genesis.json";
    const STATE_ROOT: &str = "0x1fbf85345a3cbba9a6d44f991b721e55620a22397c2a93ee8d5011136ac300ee";

    #[test]
    fn state_root() -> Result<()> {
        let file = File::open(GENESIS_FILEPATH)?;
        let genesis_config: Value = serde_json::from_reader(BufReader::new(file))?;
        let genesis_config = genesis_config
            .as_object()
            .expect("genesis_config should be object");

        let alloc = genesis_config
            .get("alloc")
            .expect("genesis_config should contain alloc")
            .as_object()
            .expect("alloc should be object");

        let mut trie = Trie::new(Box::new(MemoryDb::new()));

        for (address, account_state) in alloc {
            let address = Address::from_str(address)?;
            let account_state = account_state
                .as_object()
                .expect("account_state should be object");

            let balance = U256::deserialize(
                account_state
                    .get("balance")
                    .expect("balance should be present"),
            )?;
            let nonce = account_state
                .get("nonce")
                .map(|v| U64::deserialize(v).expect("nonce should deserialize to U64"))
                .unwrap_or_default()
                .to::<u64>();

            match account_state.get("code") {
                Some(code) => {
                    let code = const_hex::deserialize(code).expect("code should deserialize");
                    trie.create_sc(address, balance, nonce, code)?;

                    let Some(storage) = account_state.get("storage") else {
                        continue;
                    };

                    let storage_layout = AccountStorageLayout::new(address);
                    let storage = HashMap::<U256, B256>::deserialize(storage)
                        .expect("storage should deserialize");
                    for (key, value) in storage {
                        let value = U256::from_le_slice(value.as_slice());
                        trie.insert(storage_layout.storage_slot_key(key), value)?;
                    }
                }
                None => {
                    trie.create_eoa(address, balance, nonce)?;
                }
            }
        }

        assert_eq!(trie.root()?, B256::from_str(STATE_ROOT)?);

        Ok(())
    }
}
