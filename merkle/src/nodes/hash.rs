use alloy_primitives::B256;
use derive_more::Deref;

#[derive(Clone, Deref, derive_more::From)]
pub struct HashNode(B256);
