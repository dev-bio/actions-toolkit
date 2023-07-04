use std::io::{Error as IoError};
use std::fmt::{Error as FmtError};
use std::env::{VarError};
use serde_json::{Error as SerdeError};

use thiserror::Error;

use super::command::CommandError;
use super::file_command::FileCommandError;
use super::util::UtilityError;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("utility error")]
    Utility(UtilityError),
    #[error("command error")]
    Command(#[from] CommandError),
    #[error("file command error")]
    FileCommand(#[from] FileCommandError),
}