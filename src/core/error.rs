use thiserror::{Error};

use super::file_command::{FileCommandError};
use super::command::{CommandError};
use super::util::{UtilityError};
use super::log::{LogError};

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("utility error, reason: {0}")]
    Utility(#[from] UtilityError),
    #[error("command error, reason: {0}")]
    Command(#[from] CommandError),
    #[error("file command error, reason: {0}")]
    FileCommand(#[from] FileCommandError),
    #[error("log error, reason: {0}")]
    Log(#[from] LogError),
}