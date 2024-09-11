mod hmap;
mod map;

use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK".to_string()).into();
}
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command:{0}")]
    InvalidCommand(String),
    #[error("Invalid argument:{0}")]
    InvalidArgument(String),
    #[error("{0}")]
    RespError(#[from] RespError),
}

#[enum_dispatch]
pub trait CommandExcetor {
    fn execute(&self, backend: &Backend) -> RespFrame;
}

#[enum_dispatch(CommandExcetor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    Hset(HSet),
    HgetAll(HGetAll),
    Unrecognized(Unrecognized),
}
#[derive(Debug)]
pub struct Get {
    key: String,
}
#[derive(Debug)]
pub struct Unrecognized;

#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}
#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}
#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}
#[derive(Debug)]
pub struct HGetAll {
    key: String,
}
// impl From<Get> for Command {
//     fn from(command: Get) -> Self {
//         Command::Get(command)
//     }
// }
// impl From<Set> for Command {
//     fn from(command: Set) -> Self {
//         Command::Set(command)
//     }
// }
// impl From<HGet> for Command {
//     fn from(command: HGet) -> Self {
//         Command::HGet(command)
//     }
// }
// impl From<HSet> for Command {
//     fn from(command: HSet) -> Self {
//         Command::Hset(command)
//     }
// }
// impl From<HGetAll> for Command {
//     fn from(command: HGetAll) -> Self {
//         Command::HgetAll(command)
//     }
// }

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(frame: RespFrame) -> Result<Self, Self::Error> {
        match frame {
            RespFrame::Array(value) => value.try_into(),
            _ => Err(CommandError::InvalidCommand("Invalid command".to_string())),
        }
    }
}
impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(frame: RespArray) -> Result<Self, Self::Error> {
        match frame.first() {
            Some(RespFrame::BulkString(ref command)) => {
                match command.to_ascii_lowercase().as_slice() {
                    b"get" => Ok(Get::try_from(frame)?.into()),
                    b"set" => Ok(Set::try_from(frame)?.into()),
                    b"hget" => Ok(HGet::try_from(frame)?.into()),
                    b"hset" => Ok(HSet::try_from(frame)?.into()),
                    b"hgetall" => Ok(HGetAll::try_from(frame)?.into()),
                    _ => Ok(Unrecognized.into()),
                }
            }
            _ => Err(CommandError::InvalidCommand("Invalid command".to_string())),
        }
    }
}
impl CommandExcetor for Unrecognized {
    fn execute(&self, _backend: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

// impl CommandExcetor for Command {
//     fn execute(&self, backend: &Backend) -> RespFrame {
//         match self {
//             Command::Get(command) => command.execute(backend),
//             Command::Set(command) => command.execute(backend),
//             Command::HGet(command) => command.execute(backend),
//             Command::Hset(command) => command.execute(backend),
//             Command::HgetAll(command) => command.execute(backend),
//         }
//     }
// }

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have {} argument",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref command) => {
                if command.to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "{} is not a valid command,expected {}",
                        String::from_utf8_lossy(command.as_ref()),
                        name
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(format!(
                    "{} command must have {} argument",
                    name, n_args
                )));
            }
        }
    }
    Ok(())
}

fn extract_args(value: &RespArray, start: usize) -> Result<Vec<&RespFrame>, CommandError> {
    Ok(value.iter().skip(start).collect::<Vec<&RespFrame>>())
}
