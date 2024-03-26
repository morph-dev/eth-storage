use alloy_primitives::B256;

pub mod account;
pub mod mpt;
pub mod nibbles;
pub mod nodes;

type Db = dyn db::Db<B256, Vec<u8>>;
