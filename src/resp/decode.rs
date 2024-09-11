/*
   - simple string: "+OK\r\n"
   - error: "-Error message\r\n"
   - bulk error: "!<length>\r\n<error>\r\n"
   - integer: ":[<+|->]<value>\r\n"
   - bulk string: "$<length>\r\n<data>\r\n"
   - null bulk string: "$-1\r\n"
   - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
   - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
   - null array: "*-1\r\n"
   - null: "_\r\n"
   - boolean: "#<t|f>\r\n"
   - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
   - big number: "([+|-]<number>\r\n"
   - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
   - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
*/

use bytes::{Buf, BytesMut};

use super::{
    calc_total_length, extract_fixed_data, extract_simple_frame_data, parse_length, BulkString,
    RespArray, RespDecoder, RespError, RespFrame, RespMap, RespNull, RespNullArray,
    RespNullBulkString, RespSet, SimpleError, SimpleString, CRLF_LEN,
};

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
//bulk string: "$<length>\r\n<data>\r\n"
impl RespDecoder for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let (end, len) = parse_length(data, Self::PREFIX)?;
        //left data after slice the data prefix CRLF(\r\n) and the length of the data
        let remained = &data[end + CRLF_LEN..];

        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }
        data.advance(end + CRLF_LEN);
        // println!("data after advance :::::{:?}", data);
        let data = data.split_to(len + CRLF_LEN);
        println!(
            "data after split_to :::::{:?}",
            String::from_utf8(data.to_vec())
        );
        let frame = BulkString::new(data[..len].to_vec());
        Ok(frame)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}
//$-1\r\n
impl RespDecoder for RespNullBulkString {
    const PREFIX: &'static str = "$";
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        extract_fixed_data(data, "$-1\r\n", "null bulk string")?;
        Ok(RespNullBulkString)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
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

    use bytes::{BufMut, BytesMut};

    use crate::{BulkString, RespArray, RespError, SimpleError};

    use super::{RespDecoder, SimpleString};

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

    #[test]
    fn test_double_encode() -> anyhow::Result<()> {
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
    #[test]
    fn test_bulk_string_decode() -> anyhow::Result<()> {
        let mut data = BytesMut::new();
        data.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = "hello".to_string();
        let result = BulkString::decode(&mut data)?;
        assert_eq!(result, BulkString::new(frame));

        Ok(())
    }
}
