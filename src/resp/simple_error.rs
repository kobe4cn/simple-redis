use std::ops::Deref;

use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecoder, RespEncoder, RespError, CRLF_LEN};

#[derive(Debug, PartialEq, Clone)]
pub struct SimpleError(pub(crate) String);

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl RespEncoder for SimpleError {
    fn encode(&self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}
//- error: "-Error message\r\n"
impl RespDecoder for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let end = extract_simple_frame_data(data, Self::PREFIX, 1)?;
        let frame = SimpleError::new(String::from_utf8_lossy(&data[Self::PREFIX.len()..end]));
        Ok(frame)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, 1)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use crate::{RespDecoder, RespEncoder, RespError, SimpleError};

    #[test]
    fn test_simple_error_encode() {
        let frame = SimpleError::new("Error message");
        assert_eq!(frame.encode(), b"-Error message\r\n");
    }
    #[test]
    fn test_simple_error_decode() -> anyhow::Result<()> {
        let mut data = BytesMut::new();
        data.extend_from_slice(b"-Error message\r\n");
        let frame = SimpleError::new("Error message".to_string());
        let result = SimpleError::decode(&mut data)?;
        assert_eq!(result, frame);

        let mut data = BytesMut::new();
        data.extend_from_slice(b"-hello\r");
        let result = SimpleError::decode(&mut data);

        assert_eq!(result.unwrap_err(), RespError::NotComplete);

        data.put_u8(b'\n');
        let result = SimpleError::decode(&mut data)?;
        assert_eq!(result, SimpleError::new("hello".to_string()));

        Ok(())
    }
}
