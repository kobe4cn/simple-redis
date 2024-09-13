use bytes::BytesMut;

use super::{extract_fixed_data, RespDecoder, RespEncoder, RespError};

#[derive(Debug, PartialEq, Clone)]
pub struct RespNull;

impl RespNull {
    pub fn new() -> Self {
        RespNull
    }
}
impl Default for RespNull {
    fn default() -> Self {
        Self::new()
    }
}

impl RespEncoder for RespNull {
    fn encode(&self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}
//- null: "_\r\n"
impl RespDecoder for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        extract_fixed_data(data, "_\r\n", "null")?;
        Ok(RespNull)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        // extract_fixed_data(buf, "_\r\n", "null")?;
        Ok(3)
    }
}
