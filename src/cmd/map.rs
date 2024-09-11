use crate::{Backend, RespArray, RespFrame, RespNull};

use super::{extract_args, validate_command, CommandError, CommandExcetor, Get, Set, RESP_OK};

impl CommandExcetor for Get {
    fn execute(&self, backend: &Backend) -> RespFrame {
        match backend.get(&self.key) {
            Some(v) => v,
            None => RespFrame::Null(RespNull),
        }
    }
}
impl CommandExcetor for Set {
    fn execute(&self, backend: &Backend) -> RespFrame {
        backend.set(self.key.clone(), self.value.clone());
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for Get {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;
        let args = extract_args(&value, 1)?;
        match args[0] {
            RespFrame::BulkString(key) => Ok(Get {
                key: String::from_utf8_lossy(key).to_string(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Set {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["set"], 2)?;
        let args = extract_args(&value, 1)?;
        match (args[0], args[1]) {
            (RespFrame::BulkString(key), value) => Ok(Set {
                key: String::from_utf8_lossy(key).to_string(),
                value: value.clone(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::RespDecoder;

    use super::*;

    #[test]
    fn test_get_command() -> Result<(), CommandError> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$3\r\nkey\r\n");
        let frame = RespArray::decode(&mut buf)?;
        // println!("frame output: {:?}", String::from_utf8_lossy(frame));
        let result: Get = frame.try_into()?;

        assert_eq!(result.key, "key");
        Ok(())
    }

    #[test]
    fn test_set_command() -> Result<(), CommandError> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result: Set = frame.try_into()?;
        assert_eq!(result.key, "hello");
        assert_eq!(result.value, RespFrame::BulkString(b"world".into()));
        Ok(())
    }
    #[test]
    fn test_set_ge_command() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let cmd: Set = frame.try_into()?;

        let backend = Backend::new();
        // let cmd = Set {
        //     key: "hello".to_string(),
        //     value: RespFrame::BulkString(b"world".into()),
        // };
        cmd.execute(&backend);
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        // println!("frame output: {:?}", String::from_utf8_lossy(frame));
        let get_cmd: Get = frame.try_into()?;

        // let get_cmd = Get {
        //     key: "hello".to_string(),
        // };
        let result = get_cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));
        Ok(())
    }
}
