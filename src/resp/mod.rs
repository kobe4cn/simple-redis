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
mod array;
mod bool;
mod bulk_string;
mod double;
mod frame;
mod integer;
mod map;
mod null;
mod set;
mod simple_error;
mod simple_string;

use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;

pub use self::{
    array::{RespArray, RespNullArray},
    bulk_string::{BulkString, RespNullBulkString},
    frame::RespFrame,
    map::RespMap,
    null::RespNull,
    set::RespSet,
    simple_error::SimpleError,
    simple_string::SimpleString,
};

use thiserror::Error;

const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();
#[enum_dispatch]
pub trait RespEncoder {
    fn encode(&self) -> Vec<u8>;
}

pub trait RespDecoder: Sized {
    const PREFIX: &'static str;
    fn decode(data: &mut BytesMut) -> anyhow::Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RespError {
    #[error("Invalid frame :{0}")]
    InvalidFrame(String),
    #[error("Invalid frame type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length: {0}")]
    InvalidFrameLength(isize),
    #[error("Frame is not complete")]
    NotComplete,
    #[error("parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("CommandError: {0}")]
    InvalidCommand(String),
}

fn extract_simple_frame_data(
    buf: &[u8],
    prefix: &str,
    nth: usize,
) -> anyhow::Result<usize, RespError> {
    if buf.len() < 3 {
        return Err(RespError::NotComplete);
    }
    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect:SimpleString({}), got {:?} ",
            prefix, buf
        )));
    }
    //search for \r\n
    let end = find_crlf(buf, nth).ok_or(RespError::NotComplete)?;
    Ok(end)
}

fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }
    None
}

fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    except_type: &str,
) -> anyhow::Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }
    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect:{}, got {:?} ",
            except_type, buf
        )));
    }
    buf.advance(expect.len());
    Ok(())
}

fn parse_length(buf: &[u8], prefix: &str) -> anyhow::Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix, 1)?;
    let len = String::from_utf8_lossy(&buf[prefix.len()..end]).parse()?;
    Ok((end, len))
}

fn calc_total_length(
    buf: &[u8],
    len: usize,
    end: usize,
    prefix: &str,
) -> anyhow::Result<usize, RespError> {
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];
    // println!("calc_total_length: {:?}", String::from_utf8(data.to_vec()));
    match prefix {
        "*" | "~" => {
            for _ in 0..len {
                // println!("for start : {:?}", String::from_utf8(data.to_vec()));
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            for _ in 0..len {
                let len1 = SimpleString::expect_length(data)?;
                data = &data[len1..];
                total += len1;

                let len2 = RespFrame::expect_length(data)?;
                data = &data[len2..];
                total += len2;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}
