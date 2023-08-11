use thiserror::{Error};

use super::command_file::{CommandFileError};
use super::command::{CommandError};
use super::util::{UtilityError};

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Utility error!")]
    Utility(#[from] UtilityError),
    #[error("Command error!")]
    Command(#[from] CommandError),
    #[error("File command error!")]
    FileCommand(#[from] CommandFileError),
}