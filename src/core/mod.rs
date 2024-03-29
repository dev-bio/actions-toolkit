use std::fmt::{Display as FmtDisplay};

use serde::{Serialize};

pub mod command_file;
pub mod command;
pub mod error;
pub mod util;
pub mod log;

use command::{Command};
use error::{CoreError};

pub fn export_variable(name: impl AsRef<str>, ref value: impl Serialize + FmtDisplay) -> Result<(), CoreError> {
    let command_message = util::to_command_value(value)?;
    let name = name.as_ref()
        .to_owned();

    std::env::set_var(name.as_str(), {
        command_message.clone()
    });

    if let Ok(variable) = std::env::var("GITHUB_ENV") {
        if !(variable.is_empty()) {
            return Ok(command_file::issue_file_command("ENV", {
                command_file::construct_key_value_message(name.as_str(), value)?
            })?)
        }
    }

    Ok(command::issue_command(Command::new("set-env", command_message)?
        .with_property(("name".to_owned(), name))?)?)
}

pub fn add_secret_mask(secret: impl AsRef<str>) -> Result<(), CoreError> {
    Ok(command::issue_command(Command::new("add-mask", secret.as_ref())?)?)
}

pub fn add_path(path: impl AsRef<str>) -> Result<(), CoreError> {
    let path = path.as_ref();

    if std::env::var("GITHUB_PATH").is_ok() {
        command_file::issue_file_command("PATH", {
            command_file::construct_key_value_message("path", path)?
        })?;
    }

    Ok(command::issue_command(Command::new("add-path", path)?)?)
}

pub fn get_input(name: impl AsRef<str>) -> Option<String> {
    let tokens: Vec<_> = name.as_ref().split_whitespace()
        .collect();
    
    let name = tokens.join("_")
        .to_uppercase();

    std::env::var(format!("INPUT_{name}")).ok().map(|input| {
        let trimmed = input.trim();
        
        if trimmed.is_empty() { None } else { 
            Some(trimmed.to_owned()) 
        }
    })?
}

pub fn get_multiline_input(name: impl AsRef<str>) -> Option<Vec<String>> {
    get_input(name).map(|input| input.lines().filter_map(|line| {
        let trimmed = line.trim();

        if trimmed.is_empty() { None } else { 
            Some(trimmed.to_owned()) 
        }
    }).collect())
}

pub fn get_boolean_input(name: impl AsRef<str>) -> Option<bool> {
    let input = get_input(name)?;

    for accepted in ["true", "True", "TRUE"] {
        if input.contains(accepted) {
            return Some(true)
        }
    }

    for accepted in ["false", "False", "FALSE"] {
        if input.contains(accepted) {
            return Some(false)
        }
    }

    None
}

pub fn set_output(name: impl AsRef<str>, ref value: impl Serialize + FmtDisplay) -> Result<(), CoreError> {
    let name = name.as_ref()
        .to_owned();

    if let Ok(variable) = std::env::var("GITHUB_OUTPUT") {
        if !(variable.is_empty()) {
            return Ok(command_file::issue_file_command("OUTPUT", {
                command_file::construct_key_value_message(name, value)?
            })?)
        }
    }

    util::write_std_eol();
    Ok(command::issue_command(Command::new("set-output", util::to_command_value(value)?)?
        .with_property(("name".to_owned(), name))?)?)
}

pub fn is_debug() -> bool {
    if let Ok(variable) = std::env::var("RUNNER_DEBUG") {
        return match variable.as_str() {
            "1" => true,
            _ => false,
        }
    }

    false
}

pub fn set_state(name: impl AsRef<str>, ref value: impl Serialize + FmtDisplay) -> Result<(), CoreError> {
    let name = name.as_ref()
        .to_owned();

    if let Ok(variable) = std::env::var("GITHUB_STATE") {
        if !(variable.is_empty()) {
            return Ok(command_file::issue_file_command("STATE", {
                command_file::construct_key_value_message(name, value)?
            })?)
        }
    }

    Ok(command::issue_command(Command::new("save-state", util::to_command_value(value)?)?
        .with_property(("name".to_owned(), name))?)?)
}

pub fn get_state(name: impl AsRef<str>) -> Option<String> {
    let name = name.as_ref()
        .to_owned();

    if let Some(state) = std::env::var(format!("STATE_{name}")).ok() {
        if !(state.is_empty()) {
            return Some(state)
        }
    }

    None
}

pub fn set_command_echo(enabled: bool) -> Result<(), CoreError>  {
    Ok(command::issue_message("echo", if enabled { "on" } else { "off" })?)
}