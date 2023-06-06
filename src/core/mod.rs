use self::{
    
    command::{Command, Property},
    error::{CoreError}, 
};

use serde::{Serialize};

pub mod file_command;
pub mod command;
pub mod error;
pub mod util;

pub fn export_variable(name: impl AsRef<str>, ref value: impl Serialize) -> Result<(), CoreError> {
    let command_message = util::to_command_value(value)?;

    std::env::set_var(name.as_ref(), {
        command_message.clone()
    });

    if let Ok(variable) = std::env::var("GITHUB_ENV") {
        if !(variable.is_empty()) {
            return file_command::issue_file_command("ENV", {
                file_command::prepare_key_value_message(name.as_ref(), value)?
            })
        }
    }

    Ok(command::issue_command(Command::new("set-env", command_message)
        .with_property(Property::Value(name.as_ref()))))
}

pub fn set_secret(secret: String) {
    command::issue_command(Command::new("add-mask", secret))
}

pub fn add_path(path: impl AsRef<str>) -> Result<(), CoreError> {
    let path = path.as_ref();

    if std::env::var("GITHUB_PATH").is_ok() {
        file_command::issue_file_command("PATH", {
            file_command::prepare_key_value_message("path", path)?
        })?;
    }

    Ok(command::issue_command(Command::new("add-path", path)))
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

pub fn set_output(name: impl AsRef<str>, ref value: impl Serialize) -> Result<(), CoreError> {
    if let Ok(variable) = std::env::var("GITHUB_OUTPUT") {
        if !(variable.is_empty()) {
            return file_command::issue_file_command("OUTPUT", {
                file_command::prepare_key_value_message(name.as_ref(), value)?
            })
        }
    }

    Ok(command::issue_command(Command::new("set-output", util::to_command_value(value)?)
        .with_property(Property::Value(name.as_ref()))))
}

pub fn set_command_echo(enabled: bool) {
    command::issue("echo", if enabled { "on" } else { "off" })
}