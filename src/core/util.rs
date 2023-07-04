use serde_json::{
    error::{Error as JsonError},
    Value as JsonValue,
};

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
    Ok(if let JsonValue::String(value) = serde_json::to_value(value)? { value } else {
        serde_json::to_string(value)?
    })
}

pub fn to_command_message(ref message: impl Serialize) -> Result<String, UtilityError> {
    Ok(self::to_command_value(message)?
        .replace("\r", "%0D")
        .replace("\n", "%0A")
        .replace("%", "%25"))
}

pub fn to_command_property(ref message: impl Serialize) -> Result<String, UtilityError> {
    Ok(self::to_command_value(message)?
        .replace("\r", "%0D")
        .replace("\n", "%0A")
        .replace("%", "%25")
        .replace(":", "%3A")
        .replace(",", "%2C"))
}

pub fn write_std_eol() {
    println!();
}

pub fn write_std_error_eol() {
    eprintln!();
}