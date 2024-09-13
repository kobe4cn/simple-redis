use std::ops::Deref;

use bytes::{Buf, BytesMut};

use super::{
    calc_total_length, parse_length, RespDecoder, RespEncoder, RespError, RespFrame, CRLF_LEN,
};

#[derive(Debug, PartialEq, Clone)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl Deref for RespSet {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for RespSet {
    fn default() -> Self {
        Self::new()
    }
}
impl RespSet {
    pub fn new() -> Self {
        RespSet(Vec::new())
    }
    pub fn insert(&mut self, value: RespFrame) {
        if !self.0.contains(&value) {
            self.0.push(value);
        }
    }
}
impl RespEncoder for RespSet {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() * 32);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in &self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}
//- set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespDecoder for RespSet {
    const PREFIX: &'static str = "~";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let prefix = "~";
        let (end, len) = parse_length(data, prefix)?;
        let total_len = calc_total_length(data, len, end, prefix)?;
        if data.len() < total_len {
            return Err(RespError::NotComplete);
        }
        data.advance(end + CRLF_LEN);
        let mut set = RespSet::new();
        for _ in 0..len {
            let element = RespFrame::decode(data)?;
            set.insert(element);
        }
        Ok(set)
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
    fn test_set_encode() {
        let mut set = RespSet::new();
        set.insert(BulkString::new("zhangsan").into());
        set.insert(BulkString::new("lisi").into());
        set.insert(123.into());
        set.insert(BulkString::new("lisi").into());
        let frame = set;
        assert_eq!(
            frame.encode(),
            b"~3\r\n$8\r\nzhangsan\r\n$4\r\nlisi\r\n:123\r\n"
        );
    }
}
