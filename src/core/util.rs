use serde_json::error::{Error as JsonError};
use serde::{Serialize};
use thiserror::{Error};

#[derive(Error, Debug)]
pub enum UtilityError {
    #[error("encoding command value failed, reason: {0}")]
    EncodeCommandValue(String),
    #[error("encoding escaped command value failed, reason: {0}")]
    EncodeEscapedCommandValue(String),
    #[error("json serialization failed, reason: {0}")]
    SerializeJson(#[from] JsonError),
}

pub fn to_command_value(ref value: impl Serialize) -> Result<String, UtilityError> {
    Ok(serde_json::to_string(value)?)
}

pub fn to_command_value_escaped(ref value: impl Serialize) -> Result<String, UtilityError> {
    Ok(urlencoding::encode(serde_json::to_string(value)?.as_str()).into_owned())
}

pub fn write_std_eol() {
    println!();
}

pub fn write_std_error_eol() {
    eprintln!();
}