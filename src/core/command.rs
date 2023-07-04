use serde::{Serialize};

use super::util::{self, 
    
    UtilityError
};

use thiserror::{Error};

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("message encoding failed, reason: {0}")]
    MessageEncoding(String),
    #[error("property construction failed, reason: {0}")]
    ConstructProperty(String),
    #[error("command construction failed, reason: {0}")]
    Construct(String),
}

trait Construct {
    fn construct(&self) -> Result<String, CommandError>;
}

impl<T: Serialize> Construct for (String, T) {
    fn construct(&self) -> Result<String, CommandError> {
        let (key, value) = self;

        let json = serde_json::to_string(value)
            .map_err(|_| CommandError::ConstructProperty(format! {
                "failed to serialize value for key: {key}"
            }))?;

        let value = urlencoding::encode({
            json.as_str()
        });

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
            message: util::to_command_value_escaped(message).map_err(|error| match error {
                UtilityError::EncodeEscapedCommandValue(message) => CommandError::MessageEncoding(message),
                _ => CommandError::MessageEncoding(format! {
                    "unknown!"
                }),
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

        fn construct_properties(properties: &[String]) -> Result<String, CommandError> {
            match properties {
                [ref a, ref b, ref c @ .. ] => {
                    Ok(format!("{a},{b},{c}", c = {
                        construct_properties(c)?
                    }))
                }
                [ref a, ref b] => {
                    Ok(format!("{a},{b}"))
                }
                [ref a] => {
                    Ok(format!("{a}"))
                }
            }
        }

        if properties.is_empty() {

            Ok(format!("::{command}::{message}"))
        }

        else {

            let properties = construct_properties({
                self.properties.as_slice()
            })?;

            Ok(format!("::{command} {properties}::{message}"))
        }
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
