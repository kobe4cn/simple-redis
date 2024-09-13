use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecoder, RespEncoder, RespError, CRLF_LEN};

impl RespEncoder for i64 {
    fn encode(&self) -> Vec<u8> {
        format!(":{}\r\n", self).into_bytes()
    }
}
//integer: ":[<+|->]<value>\r\n"
impl RespDecoder for i64 {
    const PREFIX: &'static str = ":";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let end = extract_simple_frame_data(data, Self::PREFIX, 1)?;
        let frame = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]).parse()?;
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
    fn test_integer_encode() {
        let frame = 123;
        assert_eq!(frame.encode(), b":123\r\n");
        let frame = -123;
        assert_eq!(frame.encode(), b":-123\r\n");
    }
    #[test]
    fn test_integer_decode() -> anyhow::Result<()> {
        let mut data = BytesMut::new();
        data.extend_from_slice(b":100\r\n");
        let frame = 100;
        let result = i64::decode(&mut data)?;
        assert_eq!(result, frame);

        let mut data = BytesMut::new();
        data.extend_from_slice(b":100\r");
        let result = i64::decode(&mut data);

        assert_eq!(result.unwrap_err(), RespError::NotComplete);

        data.put_u8(b'\n');
        let result = i64::decode(&mut data)?;
        assert_eq!(result, 100);

        Ok(())
    }
}
