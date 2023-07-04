use std::{
    
    fmt::{Error as FmtError, Display},
    io::{Error as IoError, Write},
    fs::{OpenOptions},
};

use serde::{Serialize};
use uuid::{Uuid};

use super::util::{UtilityError};

use thiserror::{Error};

#[derive(Error, Debug)]
pub enum FileCommandError {
    #[error("message encoding failed, reason: {0}")]
    MessageEncoding(String),
    #[error("construct key / value message failed, reason: {0}")]
    ConstructKeyValueMessage(String),
    #[error("encode command value failed, reason: {0}")]
    EncodeCommandValue(String),
    #[error("missing variable: {0}")]
    MissingVariable(String),
    #[error("illegal key content for key: {0}")]
    IllegalKeyContent(String),
    #[error("utility error, reason: {0}")]
    Utility(#[from] UtilityError),
    #[error("format error, reason: {0}")]
    Format(#[from] FmtError),
    #[error("file error, reason: {0}")]
    File(#[from] IoError),
}

pub fn issue_file_command(command: impl Display, message: impl AsRef<str> + Serialize) -> Result<(), FileCommandError> {
    let path = std::env::var(format!("GITHUB_{command}")).map_err(|_| FileCommandError::MissingVariable(format! {
        "GITHUB_{command}"
    }))?;

    let mut file = OpenOptions::new()
        .create(false)
        .append(true)
        .write(true)
        .open(path)?;

    let command_value = super::util::to_command_value(message)?;

    Ok(writeln!(file, "{command_value}")?)
}

pub fn construct_key_value_message(key: impl AsRef<str>, value: impl Serialize) -> Result<String, FileCommandError> {
    let delimiter = format!("ghadelimiter_{uuid}", uuid = Uuid::new_v4());

    let key = key.as_ref();
    let value = super::util::to_command_value(value)?;

    if key.contains(delimiter.as_str()) {
        return Err(FileCommandError::IllegalKeyContent({
            key.to_owned()
        }))
    }

    if value.contains(delimiter.as_str()) {
        return Err(FileCommandError::IllegalKeyContent({
            value
        }))
    }

    let mut message = String::new();

    {
        use std::fmt::{Write};

        writeln!(message, "{key}<<{delimiter}")?;
        writeln!(message, "{value}")?;
        write!(message, "{delimiter}")?;
    }

    Ok(message)
}