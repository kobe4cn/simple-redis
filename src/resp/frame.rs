use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

use super::{
    BulkString, RespArray, RespDecoder, RespError, RespMap, RespNull, RespNullArray,
    RespNullBulkString, RespSet, SimpleError, SimpleString,
};

#[enum_dispatch(RespEncoder)]
#[derive(Debug, PartialEq, Clone)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

impl RespDecoder for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = data.iter().peekable();
        let frame = match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(data)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(data)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(data)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(data)?;
                Ok(frame.into())
            }
            Some(b'$') => match RespNullBulkString::decode(data) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => {
                    let frame = BulkString::decode(data)?;
                    Ok(frame.into())
                }
            },
            Some(b'*') => match RespNullArray::decode(data) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => {
                    let frame = RespArray::decode(data)?;
                    Ok(frame.into())
                }
            },
            Some(b',') => {
                let frame = f64::decode(data)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(data)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(data)?;
                Ok(frame.into())
            }
            Some(b'_') => {
                let frame = RespNull::decode(data)?;
                Ok(frame.into())
            }
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrameType(format!(
                "Decode unknown frame type {:?}",
                data
            ))),
        };
        frame
    }
    fn expect_length(data: &[u8]) -> anyhow::Result<usize, RespError> {
        let mut iter = data.iter().peekable();
        match iter.peek() {
            Some(b'$') => BulkString::expect_length(data),
            Some(b'*') => RespArray::expect_length(data),
            Some(b'%') => RespMap::expect_length(data),
            Some(b'~') => RespSet::expect_length(data),
            Some(b',') => f64::expect_length(data),
            Some(b':') => i64::expect_length(data),
            Some(b'#') => bool::expect_length(data),
            Some(b'+') => SimpleString::expect_length(data),
            Some(b'-') => SimpleError::expect_length(data),
            Some(b'_') => RespNull::expect_length(data),
            _ => Err(RespError::NotComplete),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::RespEncoder;

    use super::*;
    #[test]
    fn test_null_encode() {
        let frame = RespNull;
        assert_eq!(frame.encode(), b"_\r\n");
    }
}
