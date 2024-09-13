use std::ops::Deref;

use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecoder, RespEncoder, RespError, CRLF_LEN};

#[derive(Debug, PartialEq, Clone)]
pub struct SimpleString(pub(crate) String);

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl RespEncoder for SimpleString {
    fn encode(&self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}
impl RespEncoder for String {
    fn encode(&self) -> Vec<u8> {
        format!("+{}\r\n", self).into_bytes()
    }
}
//simple string:00 "+OK\r\n"
impl RespDecoder for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let end = extract_simple_frame_data(data, Self::PREFIX, 1)?;
        let frame = SimpleString::new(String::from_utf8_lossy(&data[Self::PREFIX.len()..end]));
        Ok(frame)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, 1)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use super::*;

    #[test]
    fn test_simple_string_encode() {
        let frame = SimpleString::new("OK");
        assert_eq!(frame.encode(), b"+OK\r\n");
    }
    #[test]
    fn test_simple_string_decode() -> anyhow::Result<()> {
        let mut data = BytesMut::new();
        data.extend_from_slice(b"+OK\r\n");
        let frame = SimpleString::new("OK".to_string());
        let result = SimpleString::decode(&mut data)?;
        assert_eq!(result, frame);

        let mut data = BytesMut::new();
        data.extend_from_slice(b"+hello\r");
        let result = SimpleString::decode(&mut data);

        assert_eq!(result.unwrap_err(), RespError::NotComplete);

        data.put_u8(b'\n');
        let result = SimpleString::decode(&mut data)?;
        assert_eq!(result, SimpleString::new("hello".to_string()));

        Ok(())
    }
}
