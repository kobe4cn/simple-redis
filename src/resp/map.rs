use std::{collections::BTreeMap, ops::Deref};

use bytes::{Buf, BytesMut};

use super::{
    calc_total_length, parse_length, RespDecoder, RespEncoder, RespError, RespFrame, SimpleString,
    CRLF_LEN,
};

#[derive(Debug, PartialEq, Clone)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);
impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
    pub fn insert(&mut self, key: impl Into<String>, value: RespFrame) {
        self.0.insert(key.into(), value);
    }
}
impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}
impl RespEncoder for RespMap {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() * 32);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        // let mut vec:Vec<_>=self.0.iter().collect();
        // vec.sort_by(|a,b|a.0.cmp(b.0));
        for (key, value) in &self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

//- map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespDecoder for RespMap {
    const PREFIX: &'static str = "%";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let (end, len) = parse_length(data, Self::PREFIX)?;
        let total_len = calc_total_length(data, len, end, Self::PREFIX)?;
        if data.len() < total_len {
            return Err(RespError::NotComplete);
        }
        data.advance(end + CRLF_LEN);
        let mut map = RespMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(data)?;
            let value = RespFrame::decode(data)?;
            map.insert(key.0, value);
        }
        Ok(map)
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
    fn test_map_encode() {
        let mut map = RespMap::new();
        map.insert("age".to_string(), 18.into());
        map.insert("name".to_string(), BulkString::new("zhangsan").into());

        let frame = map;
        assert_eq!(
            frame.encode(),
            b"%2\r\n+age\r\n:18\r\n+name\r\n$8\r\nzhangsan\r\n"
        );
    }
}
