use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::resp::{parse_length, CRLF_LEN};

use super::{extract_fixed_data, RespDecoder, RespEncoder, RespError, RespFrame};

#[derive(Debug, PartialEq, Clone)]
pub struct BulkString(pub(crate) Vec<u8>);
#[derive(Debug, PartialEq, Clone)]
pub struct RespNullBulkString;

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(v: &[u8; N]) -> Self {
        BulkString(v.to_vec()).into()
    }
}
impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(v: &[u8; N]) -> Self {
        BulkString(v.to_vec())
    }
}

impl RespEncoder for BulkString {
    fn encode(&self) -> Vec<u8> {
        /*
        第一种写法：
        生成中间的 String，并且对整个数据（包括 \r\n）进行了一次性转换，这会导致在生成中间 String 的时候可能会进行一次额外的内存分配。
        对 self.0 的内容进行了 UTF-8 转换，即使可能不需要，因为 self.0 本身已经是字节数组。
        */
        // format!("${}\r\n{}\r\n", self.len(), String::from_utf8_lossy(&self.0)).into_bytes()

        /*
        第二种写法：
        •预先为 Vec 分配足够的容量，避免重复的内存分配和复制，提高了性能。
        •不进行不必要的 UTF-8 转换，直接将字节数组 self.0 添加到缓冲区，这使得它更高效，尤其是在数据量较大时。
        */
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self.0);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}
impl RespEncoder for RespNullBulkString {
    fn encode(&self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

//bulk string: "$<length>\r\n<data>\r\n"
impl RespDecoder for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let (end, len) = parse_length(data, Self::PREFIX)?;
        //left data after slice the data prefix CRLF(\r\n) and the length of the data
        let remained = &data[end + CRLF_LEN..];

        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }
        data.advance(end + CRLF_LEN);
        // println!("data after advance :::::{:?}", data);
        let data = data.split_to(len + CRLF_LEN);
        println!(
            "data after split_to :::::{:?}",
            String::from_utf8(data.to_vec())
        );
        let frame = BulkString::new(data[..len].to_vec());
        Ok(frame)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

//$-1\r\n
impl RespDecoder for RespNullBulkString {
    const PREFIX: &'static str = "$";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        extract_fixed_data(data, "$-1\r\n", "null bulk string")?;
        Ok(RespNullBulkString)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bulk_string_encode() {
        let frame = BulkString::new(b"hello");
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame = RespNullBulkString;
        assert_eq!(frame.encode(), b"$-1\r\n");
    }
    #[test]
    fn test_bulk_string_decode() -> anyhow::Result<()> {
        let mut data = BytesMut::new();
        data.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = "hello".to_string();
        let result = BulkString::decode(&mut data)?;
        assert_eq!(result, BulkString::new(frame));

        Ok(())
    }
}
