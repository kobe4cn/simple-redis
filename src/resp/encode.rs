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
use super::{
    BulkString, RespArray, RespEncoder, RespMap, RespNull, RespNullArray, RespNullBulkString,
    RespSet, SimpleError, SimpleString,
};

impl RespEncoder for i64 {
    fn encode(&self) -> Vec<u8> {
        format!(":{}\r\n", self).into_bytes()
    }
}
impl RespEncoder for SimpleString {
    fn encode(&self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

impl RespEncoder for SimpleError {
    fn encode(&self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl RespEncoder for String {
    fn encode(&self) -> Vec<u8> {
        format!("+{}\r\n", self).into_bytes()
    }
}

impl RespEncoder for BulkString {
    fn encode(&self) -> Vec<u8> {
        /*
        第一种写法：
        生成中间的 String，并且对整个数据（包括 \r\n）进行了一次性转换，这会导致在生成中间 String 的时候可能会进行一次额外的内存分配。
        对 self.0 的内容进行了 UTF-8 转换，即使可能不需要，因为 self.0 本身已经是字节数组。
        */
        // format!("${}\r\n{}\r\n", self.len(), String::from_utf8_lossy(&self.0)).into_bytes()

        /*
        第二种写法：
        •预先为 Vec 分配足够的容量，避免重复的内存分配和复制，提高了性能。
        •不进行不必要的 UTF-8 转换，直接将字节数组 self.0 添加到缓冲区，这使得它更高效，尤其是在数据量较大时。
        */
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self.0);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespEncoder for RespNullBulkString {
    fn encode(&self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

impl RespEncoder for RespNull {
    fn encode(&self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}
impl RespEncoder for bool {
    fn encode(&self) -> Vec<u8> {
        format!("#{}\r\n", if *self { "t" } else { "f" }).into_bytes()
    }
}

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
impl RespEncoder for RespArray {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() * 32);
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
        for frame in &self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespEncoder for RespNullArray {
    fn encode(&self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl RespEncoder for RespMap {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() * 32);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        // let mut vec:Vec<_>=self.0.iter().collect();
        // vec.sort_by(|a,b|a.0.cmp(b.0));
        for (key, value) in &self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

impl RespEncoder for RespSet {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() * 32);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in &self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_simple_string_encode() {
        let frame = SimpleString::new("OK");
        assert_eq!(frame.encode(), b"+OK\r\n");
    }

    #[test]
    fn test_simple_error_encode() {
        let frame = SimpleError::new("Error message");
        assert_eq!(frame.encode(), b"-Error message\r\n");
    }

    #[test]
    fn test_integer_encode() {
        let frame = 123;
        assert_eq!(frame.encode(), b":123\r\n");
        let frame = -123;
        assert_eq!(frame.encode(), b":-123\r\n");
    }

    #[test]
    fn test_bulk_string_encode() {
        let frame = BulkString::new(b"hello");
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame = RespNullBulkString;
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_array_encode() {
        let frame = RespArray::new(vec![
            BulkString::new("set").into(),
            BulkString::new("hello").into(),
        ]);
        assert_eq!(frame.encode(), b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");
    }

    #[test]
    fn test_null_array_encode() {
        let frame = RespNullArray;
        assert_eq!(frame.encode(), b"*-1\r\n");
    }
    #[test]
    fn test_null_encode() {
        let frame = RespNull;
        assert_eq!(frame.encode(), b"_\r\n");
    }
    #[test]
    fn test_boolean_encode() {
        let frame = true;
        assert_eq!(frame.encode(), b"#t\r\n");
        let frame = false;
        assert_eq!(frame.encode(), b"#f\r\n");
    }

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
    fn test_map_encode() {
        let mut map = RespMap::new();
        map.insert("age".to_string(), 18.into());
        map.insert("name".to_string(), BulkString::new("zhangsan").into());

        let frame = map;
        assert_eq!(
            frame.encode(),
            b"%2\r\n+age\r\n:18\r\n+name\r\n$8\r\nzhangsan\r\n"
        );
    }

    #[test]
    fn test_set_encode() {
        let mut set = RespSet::new();
        set.insert(BulkString::new("zhangsan").into());
        set.insert(BulkString::new("lisi").into());
        set.insert(123.into());
        set.insert(BulkString::new("lisi").into());
        let frame = set;
        assert_eq!(
            frame.encode(),
            b"~3\r\n$8\r\nzhangsan\r\n$4\r\nlisi\r\n:123\r\n"
        );
    }
}
