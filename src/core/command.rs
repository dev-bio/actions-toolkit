use serde::{Serialize};

use super::util::{self, 
    
    UtilityError
};

use thiserror::{Error};

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Utility error!")]
    Utility(#[from] UtilityError),
    #[error("Encoding message failed, reason: {reason}")]
    MessageEncoding { reason: String },
    #[error("property construction failed, reason: {reason}")]
    ConstructProperty { reason: String },
    #[error("command construction failed, reason: {reason}")]
    Construct { reason: String },
}

trait Construct {
    fn construct(&self) -> Result<String, CommandError>;
}

impl<T: Serialize> Construct for (String, T) {
    fn construct(&self) -> Result<String, CommandError> {
        let (key, value) = self;

        let json = serde_json::to_string(value)
            .map_err(|_| CommandError::ConstructProperty {
                reason: format!("failed to serialize value for key: {key}")
            })?;

        let value = util::to_command_property({
            json.as_str()
        })?;

        Ok(format!("{key}={value}"))
    }
}

pub struct Command {
    command: String,
    message: String,
    properties: Vec<String>,
}

impl Command {
    pub fn new(command: impl AsRef<str>, message: impl Serialize) -> Result<Self, CommandError> {
        Ok(Self { 

            properties: Vec::new(),
            message: util::to_command_message(message).map_err(|error| match error {
                UtilityError::EncodeEscapedCommandValue(message) => CommandError::MessageEncoding {
                    reason: message
                },
                _ => CommandError::MessageEncoding {
                    reason: "unknown!".to_owned()
                },
            })?,
            command: command.as_ref()
                .to_owned(), 
        })
    }

    pub fn with_property<T: Serialize>(mut self, property: (String, T)) -> Result<Self, CommandError> {
        self.properties.push(property.construct()?);

        Ok(self)
    }

    pub fn construct(&self) -> Result<String, CommandError> {
        let Self { 
            
            properties,
            command, 
            message,
            
        } = self;

        fn construct_properties(properties: &[String]) -> Result<Option<String>, CommandError> {
            match properties {
                [] => {
                    Ok(None)
                }
                [ref a] => {
                    Ok(Some(format!("{a}")))
                }
                [ref a, ref b] => {
                    Ok(Some(format!("{a},{b}")))
                }
                [ref a, ref b, ref c @ .. ] => {
                    if let Some(c) = construct_properties(c)? {
                        return Ok(Some(format!("{a},{b},{c}")))
                    }

                    Ok(Some(format!("{a},{b}")))
                }
            }
        }

        if let Some(properties) = construct_properties(properties.as_slice())? {
            if !(properties.is_empty()) { 
                return Ok(format!("::{command} {properties}::{message}")) 
            }
        }

        Ok(format!("::{command}::{message}"))
    }

    pub fn issue(&self) -> Result<(), CommandError> {
        let command = self.construct()?;

        Ok(println!("{command}"))
    }
}

pub fn issue(command: impl AsRef<str>) -> Result<(), CommandError> {
    Command::new(command, "")?.issue()
}

pub fn issue_message(command: impl AsRef<str>, message: impl Serialize) -> Result<(), CommandError>  {
    Command::new(command, message)?.issue()
}

pub fn issue_command(command: Command) -> Result<(), CommandError> {
    command.issue()
}
