use std::io::{Error as IoError};
use std::fmt::{Error as FmtError};
use std::env::{VarError};
use serde_json::{Error as SerdeError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("serialization")]
    Serialize(#[from] SerdeError),
    #[error("environment variable")]
    EnvironmentVariable(#[from] VarError),
    #[error("file")]
    File(#[from] IoError),
    #[error("format")]
    Format(#[from] FmtError),
    #[error("illegal key content: {0}")]
    IllegalKeyContent(String),
    #[error("illegal value content: {0}")]
    IllegalValueContent(String),
}