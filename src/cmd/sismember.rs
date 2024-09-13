use crate::{Backend, RespArray, RespFrame};

use super::{
    extract_args, extract_args_hmget, validate_command, CommandError, CommandExcetor, Sadd,
    Sismember,
};

impl TryFrom<RespArray> for Sadd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let n_args = if value.len() > 3 { value.len() - 1 } else { 2 };
        validate_command(&value, &["sadd"], n_args)?;
        let args = extract_args_hmget(&value)?;
        match (args.0, args.1) {
            (RespFrame::BulkString(key), members) => {
                let members = members
                    .iter()
                    .map(|m| match m {
                        RespFrame::BulkString(member) => {
                            Ok(String::from_utf8_lossy(member).to_string())
                        }

                        _ => Err(CommandError::InvalidArgument(
                            "Invalid argument".to_string(),
                        )),
                    })
                    .collect::<Result<Vec<String>, CommandError>>()?;
                Ok(Sadd {
                    key: String::from_utf8_lossy(key).to_string(),
                    members,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}
impl TryFrom<RespArray> for Sismember {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], 2)?;
        let args = extract_args(&value, 1)?;
        match (args[0], args[1]) {
            (RespFrame::BulkString(key), RespFrame::BulkString(member)) => Ok(Sismember {
                key: String::from_utf8_lossy(key).to_string(),
                member: String::from_utf8_lossy(member).to_string(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}
impl CommandExcetor for Sismember {
    fn execute(&self, backend: &Backend) -> RespFrame {
        let is_member = backend.sismember(self.key.clone(), self.member.clone());
        RespFrame::Integer(is_member)
    }
}

impl CommandExcetor for Sadd {
    fn execute(&self, backend: &Backend) -> RespFrame {
        let added = backend.sadd(self.key.clone(), &self.members);
        RespFrame::Integer(added)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::RespDecoder;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_sadd() -> Result<()> {
        let input = "*4\r\n$4\r\nsadd\r\n$5\r\nmyset\r\n$4\r\nfour\r\n$4\r\nfive\r\n".as_bytes();
        let mut buf = BytesMut::with_capacity(input.len());
        buf.extend_from_slice(input);
        let frame = RespArray::decode(&mut buf)?;
        let sadd: Sadd = frame.try_into()?;
        assert_eq!(sadd.key, "myset");
        assert_eq!(sadd.members, vec!["four", "five"]);
        assert_eq!(sadd.execute(&Backend::new()), RespFrame::Integer(2));
        Ok(())
    }
    #[test]
    fn test_sismember() -> Result<()> {
        let input = "*3\r\n$9\r\nsismember\r\n$5\r\nmyset\r\n$4\r\nfour\r\n".as_bytes();
        let mut buf = BytesMut::with_capacity(input.len());
        buf.extend_from_slice(input);
        let frame = RespArray::decode(&mut buf)?;
        let sismember: Sismember = frame.try_into()?;
        assert_eq!(sismember.key, "myset");
        assert_eq!(sismember.member, "four");
        assert_eq!(sismember.execute(&Backend::new()), RespFrame::Integer(0));
        Ok(())
    }
}
