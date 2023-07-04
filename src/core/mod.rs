pub mod file_command;
pub mod command;
pub mod error;
pub mod util;

use std::fmt::Display;

use command::{Command};
use error::{CoreError};

use serde::{Serialize};

use self::command::CommandProperties;

pub fn export_variable(name: impl AsRef<str>, ref value: impl Serialize + Display) -> Result<(), CoreError> {
    let command_message = util::to_command_value(value)?;
    let name = name.as_ref()
        .to_owned();

    std::env::set_var(name.as_str(), {
        command_message.clone()
    });

    if let Ok(variable) = std::env::var("GITHUB_ENV") {
        if !(variable.is_empty()) {
            return file_command::issue_file_command("ENV", {
                file_command::construct_key_value_message(name.as_str(), value)?
            })
        }
    }

    command::issue_command(Command::new("set-env", command_message)?
        .with_property(("name".to_owned(), name)))
}

pub fn set_secret(secret: String) -> Result<(), CoreError> {
    command::issue_command(Command::new("add-mask", secret)?)
}

pub fn add_path(path: impl AsRef<str>) -> Result<(), CoreError> {
    let path = path.as_ref();

    if std::env::var("GITHUB_PATH").is_ok() {
        file_command::issue_file_command("PATH", {
            file_command::construct_key_value_message("path", path)?
        })?;
    }

    command::issue_command(Command::new("add-path", path)?)
}

pub fn get_input(name: impl AsRef<str>) -> Option<String> {
    let tokens: Vec<_> = name.as_ref().split_whitespace()
        .collect();
    
    let name = tokens.join("_")
        .to_uppercase();

    std::env::var(format!("INPUT_{name}")).ok()
}

pub fn get_multiline_input(name: impl AsRef<str>) -> Option<Vec<String>> {
    Some(get_input(name)?.lines().map(|line| {
        line.to_owned()
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

pub fn set_output(name: impl AsRef<str>, ref value: impl Serialize + Display) -> Result<(), CoreError> {
    let name = name.as_ref()
        .to_owned();

    if let Ok(variable) = std::env::var("GITHUB_OUTPUT") {
        if !(variable.is_empty()) {
            return file_command::issue_file_command("OUTPUT", {
                file_command::construct_key_value_message(name, value)?
            })
        }
    }

    util::write_std_eol();
    command::issue_command(Command::new("set-output", util::to_command_value(value)?)?
        .with_property(("name".to_owned(), name)))
}

pub fn set_state(name: impl AsRef<str>, ref value: impl Serialize + Display) -> Result<(), CoreError> {
    let name = name.as_ref()
        .to_owned();

    if let Ok(variable) = std::env::var("GITHUB_STATE") {
        if !(variable.is_empty()) {
            return file_command::issue_file_command("STATE", {
                file_command::construct_key_value_message(name, value)?
            })
        }
    }

    command::issue_command(Command::new("save-state", util::to_command_value(value)?)?
        .with_property(("name".to_owned(), name)))
}

pub fn get_state(name: impl AsRef<str>) -> Option<String> {
    let name = name.as_ref()
        .to_owned();

    std::env::var(format!("STATE_{name}")).ok()
}

pub fn set_command_echo(enabled: bool) -> Result<(), CoreError>  {
    command::issue_message("echo", if enabled { "on" } else { "off" })
}

pub fn fail(message: impl AsRef<str>) -> Result<(), CoreError>  {
    self::error(message)?;

    std::process::exit(1)
}

pub fn info(message: impl Display) {
    println!("{message}")
}

pub fn debug(message: impl AsRef<str>) -> Result<(), CoreError>  {
    command::issue_message("debug", message.as_ref())
}

pub fn debug_with_properties(message: impl AsRef<str>, properties: CommandProperties) -> Result<(), CoreError>  {
    command::issue_command(Command::with_properties("debug", message.as_ref(), properties)?)
}

pub fn notice(message: impl AsRef<str>) -> Result<(), CoreError>  {
    command::issue_message("notice", message.as_ref())
}

pub fn notice_with_properties(message: impl AsRef<str>, properties: CommandProperties) -> Result<(), CoreError>  {
    command::issue_command(Command::with_properties("notice", message.as_ref(), properties)?)
}

pub fn warning(message: impl AsRef<str>) -> Result<(), CoreError>  {
    command::issue_message("warning", message.as_ref())
}

pub fn warning_with_properties(message: impl AsRef<str>, properties: CommandProperties) -> Result<(), CoreError>  {
    command::issue_command(Command::with_properties("warning", message.as_ref(), properties)?)
}

pub fn error(message: impl AsRef<str>) -> Result<(), CoreError>  {
    command::issue_message("error", message.as_ref())
}

pub fn error_with_properties(message: impl AsRef<str>, properties: CommandProperties) -> Result<(), CoreError>  {
    command::issue_command(Command::with_properties("error", message.as_ref(), properties)?)
}

pub fn start_group(name: impl AsRef<str>) -> Result<(), CoreError>  {
    command::issue_message("group", name.as_ref())
}

pub fn end_group() -> Result<(), CoreError>  {
    command::issue("endgroup")
}