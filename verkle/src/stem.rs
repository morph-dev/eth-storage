use derive_more::{AsRef, Deref, Index};
use ssz::{Decode, Encode};

use crate::TrieKey;

const STEM_LENGTH: usize = 31;

#[derive(PartialEq, Eq, AsRef, Deref, Index)]
pub struct Stem([u8; STEM_LENGTH]);

impl From<&TrieKey> for Stem {
    fn from(key: &TrieKey) -> Self {
        let mut stem = [0u8; STEM_LENGTH];
        stem.copy_from_slice(&key[..STEM_LENGTH]);
        Stem(stem)
    }
}

impl Encode for Stem {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        STEM_LENGTH
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend(self.as_ref())
    }

    fn ssz_bytes_len(&self) -> usize {
        STEM_LENGTH
    }
}

impl Decode for Stem {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        STEM_LENGTH
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        match <[u8; 31]>::try_from(bytes) {
            Ok(stem) => Ok(Self(stem)),
            Err(_) => Err(ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: STEM_LENGTH,
            }),
        }
    }
}
