use std::{
    
    fmt::{

        Display as FmtDisplay,
        Error as FmtError, 
    },

    io::{

        Write as FmtWrite,
        Error as IoError, 
    },

    fs::{OpenOptions},
};

use thiserror::{Error};
use serde::{Serialize};
use uuid::{Uuid};

use super::util::{UtilityError};

#[derive(Error, Debug)]
pub enum CommandFileError {
    #[error("Message encoding failed, reason: '{reason}'")]
    MessageEncoding { reason: String },
    #[error("Construct key / value message failed, reason: '{reason}'")]
    ConstructKeyValueMessage { reason: String },
    #[error("Encode command value failed, reason: '{reason}'")]
    EncodeCommandValue { reason: String },
    #[error("missing variable: {variable}")]
    MissingVariable { variable: String },
    #[error("Illegal key content: '{key}'")]
    IllegalKeyContent { key: String },
    #[error("Illegal value content: '{value}'")]
    IllegalValueContent { value: String },
    #[error("utility error, reason: {0}")]
    Utility(#[from] UtilityError),
    #[error("format error, reason: {0}")]
    Format(#[from] FmtError),
    #[error("file error, reason: {0}")]
    File(#[from] IoError),
}

pub fn issue_file_command(command: impl FmtDisplay, message: impl AsRef<str> + Serialize) -> Result<(), CommandFileError> {
    let path = std::env::var(format!("GITHUB_{command}"))
        .map_err(|_| CommandFileError::MissingVariable{
            variable: format!("GITHUB_{command}")
        })?;

    let mut file = OpenOptions::new()
        .create(false)
        .append(true)
        .write(true)
        .open(path)?;

    let command_value = super::util::to_command_value(message)?;

    Ok(writeln!(file, "{command_value}")?)
}

pub fn construct_key_value_message(key: impl AsRef<str>, value: impl Serialize) -> Result<String, CommandFileError> {
    let delimiter = format!("ghadelimiter_{uuid}", uuid = Uuid::new_v4());

    let key = key.as_ref();
    let value = super::util::to_command_value(value)?;

    if key.contains(delimiter.as_str()) {
        return Err(CommandFileError::IllegalKeyContent {
            key: key.to_owned()
        })
    }

    if value.contains(delimiter.as_str()) {
        return Err(CommandFileError::IllegalValueContent {
            value: value.to_owned()
        })
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