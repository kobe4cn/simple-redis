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
mod decode;
mod encode;

use std::collections::BTreeMap;
use std::ops::Deref;

use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;

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
}

#[enum_dispatch(RespEncoder)]
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub struct SimpleString(String);
#[derive(Debug, PartialEq)]
pub struct SimpleError(String);
#[derive(Debug, PartialEq)]
pub struct RespNullArray;
#[derive(Debug, PartialEq)]
pub struct RespNull;
#[derive(Debug, PartialEq)]
pub struct RespNullBulkString;
#[derive(Debug, PartialEq)]
pub struct BulkString(Vec<u8>);
#[derive(Debug, PartialEq)]
pub struct RespArray(Vec<RespFrame>);
#[derive(Debug, PartialEq)]
pub struct RespMap(BTreeMap<String, RespFrame>);

#[derive(Debug, PartialEq)]
pub struct RespSet(Vec<RespFrame>);

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
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
impl Default for RespSet {
    fn default() -> Self {
        Self::new()
    }
}
impl RespSet {
    pub fn new() -> Self {
        RespSet(Vec::new())
    }
    pub fn insert(&mut self, value: RespFrame) {
        if !self.0.contains(&value) {
            self.0.push(value);
        }
    }
}
impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}
impl From<Vec<RespFrame>> for RespArray {
    fn from(v: Vec<RespFrame>) -> Self {
        RespArray(v)
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(v: &[u8; N]) -> Self {
        BulkString(v.to_vec()).into()
    }
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
