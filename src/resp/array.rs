use std::ops::Deref;

use bytes::{Buf, BytesMut};

use super::{
    calc_total_length, extract_fixed_data, parse_length, RespDecoder, RespEncoder, RespError,
    RespFrame, CRLF_LEN,
};

#[derive(Debug, PartialEq, Clone)]
pub struct RespArray(pub(crate) Vec<RespFrame>);
#[derive(Debug, PartialEq, Clone)]
pub struct RespNullArray;

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}
impl From<Vec<RespFrame>> for RespArray {
    fn from(v: Vec<RespFrame>) -> Self {
        RespArray(v)
    }
}

impl RespEncoder for RespArray {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() * 32);
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
        for frame in &self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespEncoder for RespNullArray {
    fn encode(&self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

//null array: "*-1\r\n"
impl RespDecoder for RespNullArray {
    const PREFIX: &'static str = "*-1";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        extract_fixed_data(data, "*-1\r\n", "null array")?;
        Ok(RespNullArray)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(5)
    }
}

//- array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//b"*2\r\n$3\r\nset\r\n"
impl RespDecoder for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let (end, len) = parse_length(data, Self::PREFIX)?;
        let total_len = calc_total_length(data, len, end, Self::PREFIX)?;
        if data.len() < total_len {
            return Err(RespError::NotComplete);
        }
        data.advance(end + CRLF_LEN);
        //b"$3\r\nset\r\n
        let mut frames = Vec::with_capacity(len);

        // move the *<number-of-elements>\r\n

        for _ in 0..len {
            println!("data :::::{:?}", String::from_utf8(data.to_vec()));
            frames.push(RespFrame::decode(data)?);
        }
        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, len, end, Self::PREFIX)?;
        Ok(total_len)
    }
}

#[cfg(test)]
mod tests {
    use crate::BulkString;

    use super::*;

    #[test]
    fn test_array_encode() {
        let frame = RespArray::new(vec![
            BulkString::new("set").into(),
            BulkString::new("hello").into(),
        ]);
        assert_eq!(frame.encode(), b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");
    }

    #[test]
    fn test_null_array_encode() {
        let frame = RespNullArray;
        assert_eq!(frame.encode(), b"*-1\r\n");
    }
    #[test]
    fn test_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");

        let frame = RespArray::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);
        Ok(())
    }
}
