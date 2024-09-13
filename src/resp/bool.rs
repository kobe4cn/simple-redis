use bytes::BytesMut;

use super::{extract_fixed_data, RespDecoder, RespEncoder, RespError};

impl RespEncoder for bool {
    fn encode(&self) -> Vec<u8> {
        format!("#{}\r\n", if *self { "t" } else { "f" }).into_bytes()
    }
}
//- boolean: "#<t|f>\r\n"
impl RespDecoder for bool {
    const PREFIX: &'static str = "#";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        match extract_fixed_data(data, "#t\r\n", "bool") {
            Ok(_) => Ok(true),
            Err(_) => match extract_fixed_data(data, "#f\r\n", "bool") {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}
#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use super::*;

    #[test]
    fn test_boolean_encode() {
        let frame = true;
        assert_eq!(frame.encode(), b"#t\r\n");
        let frame = false;
        assert_eq!(frame.encode(), b"#f\r\n");
    }
    #[test]
    fn test_bool_decode() -> anyhow::Result<()> {
        let mut data = BytesMut::new();
        data.extend_from_slice(b"#t\r\n");
        let frame = true;
        let result = bool::decode(&mut data)?;
        assert_eq!(result, frame);

        let mut data = BytesMut::new();
        data.extend_from_slice(b"#f\r\n");
        let frame = false;
        let result = bool::decode(&mut data)?;
        assert_eq!(result, frame);

        let mut data = BytesMut::new();
        data.extend_from_slice(b"#t\r");
        let result = bool::decode(&mut data);

        assert_eq!(result.unwrap_err(), RespError::NotComplete);

        data.put_u8(b'\n');
        let result = bool::decode(&mut data)?;
        assert!(result);

        Ok(())
    }
}
