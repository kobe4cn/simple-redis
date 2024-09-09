use bytes::BytesMut;
use simple_redis::{RespArray, RespDecoder};

fn main() {
    // println!("Hello, world!");
    let mut buf = BytesMut::new();
    // buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
    buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

    let frame = RespArray::decode(&mut buf);
    println!("{:?}", frame);
}
