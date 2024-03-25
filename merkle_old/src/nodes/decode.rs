use alloy_rlp::{Decodable, Header, EMPTY_STRING_CODE};
use bytes::{Buf, Bytes};

#[derive(Debug)]
pub(crate) enum RlpStructure {
    List(Vec<Bytes>),
    Value(Bytes),
}

impl Decodable for RlpStructure {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        if buf[0] < EMPTY_STRING_CODE {
            return Ok(Self::Value(buf.copy_to_bytes(1)));
        }
        let header = Header::decode(buf)?;

        if header.list {
            let mut bytes_vec: Vec<Bytes> = vec![];
            let mut consumed: usize = 0;
            while consumed < header.payload_length {
                let total_len = if buf[0] < EMPTY_STRING_CODE {
                    1
                } else {
                    let mut tmp_buf = *buf;
                    let tmp_header = Header::decode(&mut tmp_buf)?;

                    tmp_header.length() + tmp_header.payload_length
                };

                bytes_vec.push(buf.copy_to_bytes(total_len));
                consumed += total_len;
            }
            Ok(RlpStructure::List(bytes_vec))
        } else {
            Ok(RlpStructure::Value(
                buf.copy_to_bytes(header.payload_length),
            ))
        }
    }
}
