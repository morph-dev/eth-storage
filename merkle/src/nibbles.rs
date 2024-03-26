use std::{cmp, fmt::Write, ops::Deref};

use anyhow::{bail, Result};
use derive_more::{Deref, Index, LowerHex, UpperHex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deref, LowerHex, UpperHex)]
pub struct Nibble(u8);

impl Nibble {
    pub fn split(byte: u8) -> [Self; 2] {
        [Nibble(byte >> 4), Nibble(byte & 0xF)]
    }

    pub fn join<A, B>(first: A, second: B) -> u8
    where
        A: Deref<Target = u8>,
        B: Deref<Target = u8>,
    {
        *first << 4 | *second
    }
}

impl TryFrom<u8> for Nibble {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            0..=15 => Ok(Nibble(value)),
            _ => bail!("Invalid nibble value: {value}"),
        }
    }
}

#[derive(Clone, Deref, Index, PartialEq, Eq)]
pub struct Nibbles(Vec<Nibble>);

impl Nibbles {
    pub fn from_packed<T: AsRef<[u8]>>(packed_nibbles: T) -> Self {
        let packed_nibbles = packed_nibbles.as_ref();
        let mut nibbles = Vec::with_capacity(packed_nibbles.len());
        for &b in packed_nibbles {
            nibbles.extend(Nibble::split(b));
        }
        Self(nibbles)
    }

    pub fn from_slice<T: AsRef<[Nibble]>>(nibbles: T) -> Self {
        Self(nibbles.as_ref().to_vec())
    }

    // Public util functions

    pub fn common_prefix(&self, other: &[Nibble]) -> usize {
        let len = cmp::min(self.len(), other.len());
        for i in 0..len {
            if self[i] != other[i] {
                return i;
            }
        }
        len
    }

    // Compact

    const LEAF_FLAG: u8 = 0b10;
    const ODD_LEN_FLAG: u8 = 0b01;

    pub fn compact_len(&self) -> usize {
        1 + self.len() / 2
    }

    pub fn to_compact(&self, is_leaf: bool) -> Vec<u8> {
        let start;
        let mut flags = if is_leaf { Self::LEAF_FLAG } else { 0 };
        let first_byte_nibble;

        if self.len() % 2 == 0 {
            // Even length
            first_byte_nibble = Nibble(0);
            start = 0;
        } else {
            // Odd length
            flags |= Self::ODD_LEN_FLAG;
            first_byte_nibble = self[0];
            start = 1;
        }

        let mut result = Vec::with_capacity(1 + self.len() / 2);
        result.push(Nibble::join(&flags, first_byte_nibble));
        for i in (start..self.len()).step_by(2) {
            result.push(Nibble::join(self[i], self[i + 1]));
        }
        result
    }

    pub fn from_compact(bytes: &[u8]) -> Result<(Self, bool)> {
        let Some((&first, bytes)) = bytes.split_first() else {
            bail!("Can't create from compact that is empty");
        };
        let [flags, first_byte_nibble] = Nibble::split(first);
        let flags = *flags;
        if flags > Self::ODD_LEN_FLAG | Self::LEAF_FLAG {
            bail!("Invalid first byte ({first:#04X}): invalid flags")
        }

        let mut nibbles: Vec<Nibble>;
        if flags & Self::ODD_LEN_FLAG != 0 {
            // odd length
            nibbles = Vec::with_capacity(1 + 2 * bytes.len());
            nibbles.push(first_byte_nibble);
        } else {
            // even length
            if *first_byte_nibble != 0 {
                bail!("Invalid first byte ({first:#04X}): even lenght and non-zero low bits")
            }
            nibbles = Vec::with_capacity(2 * bytes.len());
        }
        for &byte in bytes {
            nibbles.extend(Nibble::split(byte));
        }

        Ok((Self(nibbles), flags & Self::LEAF_FLAG != 0))
    }
}

impl FromIterator<Nibble> for Nibbles {
    fn from_iter<T: IntoIterator<Item = Nibble>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl std::fmt::Debug for Nibbles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str = self.iter().fold(String::from("0x"), |mut output, nibble| {
            let _ = write!(output, "{nibble:X}");
            output
        });

        f.debug_struct("Nibbles")
            .field("unpacked_nibbles", &as_str)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack() {
        for (input, expected) in [
            (vec![], vec![]),
            (vec![0xa0], vec![0xa, 0x0]),
            (vec![0x0a], vec![0x0, 0xa]),
            (vec![0xab], vec![0xa, 0xb]),
            (vec![0x13, 0x37], vec![0x1, 0x3, 0x3, 0x7]),
        ] {
            let nibbles = Nibbles::from_packed(input);
            assert_eq!(
                *nibbles,
                expected.into_iter().map(Nibble).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn common_prefix() {
        for (input1, input2, expected) in [
            (vec![0x12], vec![0xab], 0),
            (vec![0x12, 0x34, 0x56], vec![0x12, 0x3a, 0xbc], 3),
            (vec![0x12, 0x34, 0x56], vec![0x12, 0x34, 0x56], 6),
            (vec![0x12, 0x34, 0x56], vec![0x12, 0x34], 4),
            (vec![0x12, 0x34], vec![0x12, 0x34, 0x56], 4),
        ] {
            let nibbles1 = Nibbles::from_packed(input1);
            let nibbles2 = Nibbles::from_packed(input2);
            assert_eq!(nibbles1.common_prefix(&nibbles2), expected);
        }
    }

    #[test]
    fn compact() -> Result<()> {
        for (compact, nibbles_vec, is_leaf) in [
            // manual
            (vec![0x00], vec![], false),
            (vec![0x20], vec![], true),
            (vec![0x1a], vec![0xa], false),
            (vec![0x3a], vec![0xa], true),
            (vec![0x00, 0xab], vec![0xa, 0xb], false),
            (vec![0x20, 0xab], vec![0xa, 0xb], true),
            (vec![0x1a, 0xbc], vec![0xa, 0xb, 0xc], false),
            (vec![0x3a, 0xbc], vec![0xa, 0xb, 0xc], true),
            // from https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie
            (vec![0x11, 0x23, 0x45], vec![0x1, 0x2, 0x3, 0x4, 0x5], false),
            (
                vec![0x00, 0x01, 0x23, 0x45],
                vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5],
                false,
            ),
            (
                vec![0x20, 0x0f, 0x1c, 0xb8],
                vec![0x0, 0xf, 0x1, 0xc, 0xb, 0x8],
                true,
            ),
            (vec![0x3f, 0x1c, 0xb8], vec![0xf, 0x1, 0xc, 0xb, 0x8], true),
        ] {
            // to compact
            let nibbles: Nibbles = nibbles_vec.iter().cloned().map(Nibble).collect();
            assert_eq!(
                nibbles.compact_len(),
                compact.len(),
                ", we are testing nibbles: {:02x?}",
                nibbles_vec
            );
            assert_eq!(nibbles.to_compact(is_leaf), compact);

            // from compact
            let (nibbles_from_compact, is_leaf_from_compact) = Nibbles::from_compact(&compact)?;
            assert_eq!(nibbles_from_compact, nibbles);
            assert_eq!(is_leaf_from_compact, is_leaf);
        }

        Ok(())
    }
}
