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
// mod decode;
mod encode;

use std::collections::BTreeMap;
use std::ops::Deref;

use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait RespEncoder {
    fn encode(&self) -> Vec<u8>;
}

pub trait RespDecoder {
    fn decode(data: &[u8]) -> Result<RespFrame, String>;
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
