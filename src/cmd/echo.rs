use crate::{Backend, BulkString, RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExcetor, Echo};

impl CommandExcetor for Echo {
    fn execute(&self, _backend: &Backend) -> RespFrame {
        RespFrame::BulkString(BulkString::new(self.message.clone().into_bytes()))
    }
}
impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;
        let args = extract_args(&value, 1)?;
        match args[0] {
            RespFrame::BulkString(message) => Ok(Echo {
                message: String::from_utf8_lossy(message).to_string(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}
