use serde::{Serialize};
use thiserror::{Error};

#[derive(Error, Debug)]
pub enum UtilityError {
    #[error("encoding command value failed, reason: {0}")]
    EncodeCommandValue(String),
    #[error("encoding escaped command value failed, reason: {0}")]
    EncodeEscapedCommandValue(String),
}

pub fn to_command_value(ref value: impl Serialize) -> Result<String, UtilityError> {
    Ok(serde_json::to_string(value).map_err(|_| UtilityError::EncodeCommandValue(format! {
        "failed to serialize value!"
    }))?)
}

pub fn to_command_value_escaped(ref value: impl Serialize) -> Result<String, UtilityError> {
    Ok(urlencoding::encode(serde_json::to_string(value).map_err(|_| UtilityError::EncodeEscapedCommandValue(format! {
        "failed to serialize value!"
    }))?.as_str()).into_owned())
}

pub fn write_std_eol() {
    println!();
}

pub fn write_std_error_eol() {
    eprintln!();
}