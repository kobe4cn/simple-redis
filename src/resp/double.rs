use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecoder, RespEncoder, RespError, CRLF_LEN};

impl RespEncoder for f64 {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:+e}\r\n", self)
        } else {
            // let sign = if self < &0.0 { "" } else { "+" };
            format!(",{}\r\n", self)
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

//- double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespDecoder for f64 {
    const PREFIX: &'static str = ",";
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
    use super::*;
    #[test]
    fn test_double_encode() {
        let frame = 123.456;
        assert_eq!(frame.encode(), b",123.456\r\n");
        let frame = -123.456;
        assert_eq!(frame.encode(), b",-123.456\r\n");
        let frame = 1.23456e+8;
        assert_eq!(frame.encode(), b",+1.23456e8\r\n");
        let frame = -1.23456e-9;
        assert_eq!(&frame.encode(), b",-1.23456e-9\r\n");
    }
    #[test]
    fn test_double_decode() -> anyhow::Result<()> {
        let mut data = BytesMut::new();
        data.extend_from_slice(b",1.23\r\n");
        let frame = 1.23;
        let result = f64::decode(&mut data)?;
        assert_eq!(result, frame);

        let mut data = BytesMut::new();
        data.extend_from_slice(b",+1.23456e8\r\n");
        let frame = 1.23456e+8;
        let result = f64::decode(&mut data)?;
        assert_eq!(result, frame);

        let mut data = BytesMut::new();
        data.extend_from_slice(b",-1.23456e-9\r\n");
        let result = f64::decode(&mut data)?;
        assert_eq!(result, -1.23456e-9);

        Ok(())
    }
}
