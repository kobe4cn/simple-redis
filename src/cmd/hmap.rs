use super::{
    extract_args, extract_args_hmget, validate_command, CommandError, CommandExcetor, HGet,
    HGetAll, HMget, HSet, RESP_OK,
};
use crate::{Backend, BulkString, RespArray, RespFrame, RespNull, SimpleString};
use anyhow::Result;

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], 2)?;
        let args = extract_args(&value, 1)?;
        match (args[0], args[1]) {
            (RespFrame::BulkString(key), RespFrame::BulkString(field)) => Ok(HGet {
                key: String::from_utf8_lossy(key).to_string(),
                field: String::from_utf8_lossy(field).to_string(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hgetall"], 1)?;
        let args = extract_args(&value, 1)?;
        match args[0] {
            RespFrame::BulkString(key) => Ok(HGetAll {
                key: String::from_utf8_lossy(key).to_string(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hset"], 3)?;
        let args = extract_args(&value, 1)?;
        match (args[0], args[1], args[2]) {
            (RespFrame::BulkString(key), RespFrame::BulkString(field), value) => Ok(HSet {
                key: String::from_utf8_lossy(key).to_string(),
                field: String::from_utf8_lossy(field).to_string(),
                value: value.clone(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}
impl TryFrom<RespArray> for HMget {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let n_args = if value.len() > 3 { value.len() - 1 } else { 2 };
        validate_command(&value, &["hmget"], n_args)?;
        let args = extract_args_hmget(&value)?;
        match (&args.0, &args.1) {
            (RespFrame::BulkString(key), fields) => {
                let fields = fields
                    .iter()
                    .map(|v| match v {
                        RespFrame::BulkString(v) => Ok(String::from_utf8_lossy(v).to_string()),
                        _ => Err(CommandError::InvalidArgument(
                            "Invalid argument".to_string(),
                        )),
                    })
                    .collect::<Result<Vec<String>, CommandError>>()?;
                Ok(HMget {
                    key: String::from_utf8_lossy(key).to_string(),
                    fields,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}
impl CommandExcetor for HMget {
    fn execute(&self, backend: &Backend) -> RespFrame {
        let mut result = Vec::new();
        for field in self.fields.iter() {
            match backend.hget(&self.key, field) {
                Some(v) => result.push(v),
                None => result.push(RespFrame::SimpleString(SimpleString::new("(nil)"))),
            }
        }
        RespArray::new(result).into()
    }
}
impl CommandExcetor for HGet {
    fn execute(&self, backend: &Backend) -> RespFrame {
        match backend.hget(&self.key, &self.field) {
            Some(v) => v,
            None => RespFrame::Null(RespNull),
        }
    }
}
impl CommandExcetor for HGetAll {
    fn execute(&self, backend: &Backend) -> RespFrame {
        let hmap = backend.hgetall(&self.key);
        match hmap {
            Some(v) => {
                let mut result = Vec::new();
                for v in v.iter() {
                    result.push(RespFrame::BulkString(BulkString::new(v.key().to_owned())));
                    result.push(v.value().clone());
                }
                // if self.sort {
                //     result.sort_by(|a, b| a.0.cmp(&b.0));
                // }
                // let ret = result
                //     .into_iter()
                //     .flat_map(|(k, v)| {
                //         vec![RespFrame::BulkString(BulkString::new(k.clone())), v.clone()]
                //     })
                //     .collect::<Vec<RespFrame>>();
                RespArray::new(result).into()
            }
            None => RespFrame::Null(RespNull),
        }
    }
}
impl CommandExcetor for HSet {
    fn execute(&self, backend: &Backend) -> RespFrame {
        backend.hset(self.key.clone(), self.field.clone(), self.value.clone());
        RESP_OK.clone()
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::RespDecoder;

    use super::*;

    #[test]
    fn test_hget_command() -> Result<(), CommandError> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nkey\r\n$5\r\nfield\r\n");
        let frame = RespArray::decode(&mut buf)?;
        // println!("frame output: {:?}", String::from_utf8_lossy(frame));
        let result: HGet = frame.try_into()?;

        assert_eq!(result.key, "key");
        assert_eq!(result.field, "field");
        Ok(())
    }

    #[test]
    fn test_hgetall_command() -> Result<(), CommandError> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$7\r\nhgetall\r\n$3\r\nkey\r\n");
        let frame = RespArray::decode(&mut buf)?;
        // println!("frame output: {:?}", String::from_utf8_lossy(frame));
        let result: HGetAll = frame.try_into()?;

        assert_eq!(result.key, "key");
        Ok(())
    }

    #[test]
    fn test_hset_command() -> Result<(), CommandError> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n");
        let frame = RespArray::decode(&mut buf)?;
        // println!("frame output: {:?}", String::from_utf8_lossy(frame));
        let result: HSet = frame.try_into()?;

        assert_eq!(result.key, "key");
        assert_eq!(result.field, "field");
        assert_eq!(result.value, RespFrame::BulkString(b"value".into()));
        Ok(())
    }
    #[test]
    fn test_hmget_command() -> Result<(), CommandError> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$5\r\nhmget\r\n$3\r\nsay\r\n$5\r\nhello\r\n$6\r\nhello1\r\n");
        let frame = RespArray::decode(&mut buf)?;
        println!("frame output: {:?}", frame);
        let result: HMget = frame.try_into()?;

        assert_eq!(result.key, "say");
        assert_eq!(result.fields, vec!["hello", "hello1"]);
        Ok(())
    }
}
