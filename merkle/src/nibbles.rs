use derive_more::{Deref, Index};

#[derive(Clone, Deref, Index)]
pub struct Nibbles {
    nibbles: Vec<u8>,
}

impl<T: AsRef<[u8]>> From<T> for Nibbles {
    fn from(data: T) -> Self {
        return Nibbles {
            nibbles: Vec::from(data.as_ref()),
        };
    }
}

impl Nibbles {
    pub fn first(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self[0] as usize)
        }
    }

    pub fn skip_head(&self, count: usize) -> Self {
        Nibbles {
            nibbles: self.nibbles[count..].into(),
        }
    }

    pub fn unpack<T: AsRef<[u8]>>(path: T) -> Self {
        Self {
            nibbles: path
                .as_ref()
                .iter()
                .flat_map(|&b| vec![b >> 4, b & 0b1111])
                .collect(),
        }
    }

    pub fn common_prefix(&self, other: &Nibbles) -> usize {
        let len = std::cmp::min(self.len(), other.len());
        for i in 0..len {
            if self[i] != other[i] {
                return i;
            }
        }
        len
    }

    const LEAF_FLAG: u8 = 0b10;
    const ODD_LEN_FLAG: u8 = 0b01;

    pub fn compact_len(&self) -> usize {
        1 + self.len() / 2
    }

    pub fn to_compact(&self, is_leaf: bool) -> Vec<u8> {
        let leaf_flag: u8 = if is_leaf { Self::LEAF_FLAG } else { 0 };

        let start;
        let first_byte;

        if self.len() % 2 == 0 {
            // Even length
            let flags = leaf_flag;
            first_byte = flags << 4;
            start = 0;
        } else {
            // Odd length
            let flags = leaf_flag | Self::ODD_LEN_FLAG;
            first_byte = (flags << 4) | (self[0]);
            start = 1;
        }

        let mut result = Vec::with_capacity(1 + self.len() / 2);
        result.push(first_byte);
        for i in (start..self.len()).step_by(2) {
            result.push(self[i] << 4 | self[i + 1]);
        }
        result
    }

    pub fn from_compact(bytes: &[u8]) -> (Self, bool) {
        let first = bytes[0];
        let flags = first >> 4;

        let mut nibbles: Vec<u8>;
        if flags & Self::ODD_LEN_FLAG == 0 {
            // eve length
            nibbles = Vec::with_capacity(2 * (bytes.len() - 1));
        } else {
            // odd length
            nibbles = Vec::with_capacity(2 * bytes.len() - 1);
            nibbles.push(first & 0xF);
        }
        for byte in &bytes[1..] {
            nibbles.push(byte >> 4);
            nibbles.push(byte & 0xF);
        }

        (Self::from(nibbles), flags & Self::LEAF_FLAG != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::Nibbles;

    #[test]
    fn unpack() {
        for (input, expected) in [
            (vec![], vec![]),
            (vec![0xa0], vec![0xa, 0x0]),
            (vec![0x0a], vec![0x0, 0xa]),
            (vec![0xab], vec![0xa, 0xb]),
            (vec![0x13, 0x37], vec![0x1, 0x3, 0x3, 0x7]),
        ] {
            let nibbles = Nibbles::unpack(input);
            assert_eq!(nibbles[..], expected);
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
            let nibbles1 = Nibbles::unpack(input1);
            let nibbles2 = Nibbles::unpack(input2);
            assert_eq!(nibbles1.common_prefix(&nibbles2), expected);
        }
    }

    #[test]
    fn compact() {
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
            let nibbles = Nibbles::from(&nibbles_vec);
            assert_eq!(
                nibbles.compact_len(),
                compact.len(),
                ", we are testing nibbles: {:02x?}",
                nibbles_vec
            );
            assert_eq!(nibbles.to_compact(is_leaf), compact);

            // from compact
            let (nibbles_from_compact, is_leaf_from_compact) = Nibbles::from_compact(&compact);
            assert_eq!(nibbles_from_compact.nibbles, nibbles_vec);
            assert_eq!(is_leaf_from_compact, is_leaf);
        }
    }
}
