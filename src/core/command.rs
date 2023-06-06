use std::{
    
    fmt::{

        Formatter, 
        Display, 
        Result as FmtResult,
    }
};

use serde::{Serialize};

use super::error::{CoreError};

trait DynamicProperty {
    fn to_string(&self) -> Result<String, CoreError>;
}


#[derive(Debug)]
#[derive(Serialize)]
pub enum Property<T: Serialize + Display> {
    Value(T),
    KeyValue(String, T),
}

impl<T: Serialize + Display> DynamicProperty for Property<T> {
    fn to_string(&self) -> Result<String, CoreError> {
        Ok(match self {
            Self::KeyValue(key, value) => {
                let json = serde_json::to_string(value)?;
                let value = urlencoding::encode(json.as_str());

                format!("{key}={value}")
            },
            Self::Value(value) => {
                let json = serde_json::to_string(value)?;
                let value = urlencoding::encode(json.as_str());

                format!("{value}={value}")
            }
        })
    }
}

pub struct CommandProperties<'a> {
    properties: Vec<Box<dyn DynamicProperty + 'a>>
}

impl<'a> CommandProperties<'a> {
    pub fn new() -> Self {
        Self { properties: Vec::new() }
    }

    pub fn with<T: Serialize + Display + 'a>(mut self, property: Property<T>) -> Self {
        self.properties.push(Box::new(property));
        self
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    fn formatter(properties: &[Box<dyn DynamicProperty + 'a>]) -> Result<String, CoreError> {
        match properties {
            &[] => {
                Ok(format!(""))
            }
            &[ref a] => {
                let a = a.to_string()?;
                
                Ok(format!("{a}"))
            }
            &[ref a, ref b] => {
                let a = a.to_string()?;
                let b = b.to_string()?;
                
                Ok(format!("{a},{b}"))
            }
            &[ref a, ref b, ref c @ .. ] => {
                let a = a.to_string()?;
                let b = b.to_string()?;
                let c = Self::formatter(c)?;

                Ok(format!("{a},{b},{c}"))
            }
        }
    }
}

impl<'a> Display for CommandProperties<'a> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        if let Ok(printed) = Self::formatter(self.properties.as_slice()) {
            write!(fmt, "{printed}")?
        }

        Ok(())
    }
}

pub struct Command<'a> {
    command: String,
    message: String,
    properties: Option<CommandProperties<'a>>,
}

impl<'a> Command<'a> {
    pub fn new(command: impl AsRef<str>, message: impl AsRef<str>) -> Self {
        let properties = None;

        Self { 

            command: command.as_ref()
                .to_owned(), 
            message: message.as_ref()
                .to_string(),
            properties,
        }
    }

    pub fn with_properties(command: impl AsRef<str>, message: impl AsRef<str>, mut properties: Option<CommandProperties<'a>>) -> Self {
        if let Some(ref inner) = properties {
            if inner.is_empty() {
                properties = None
            }
        }

        Self { 

            command: command.as_ref()
                .to_owned(), 
            message: message.as_ref()
                .to_string(),
            properties,
        }
    }

    pub fn with_property<T: Serialize + Display + 'a>(mut self, property: Property<T>) -> Self {
        if let Some(properties) = self.properties {
            self.properties = Some(properties.with(property));
        }
        
        else {

            self.properties = Some(CommandProperties::new()
                .with(property));
        }

        self
    }

    pub fn issue(&self) {
        println!("{self}")
    }
}

fn get_

impl<'a> Display for Command<'a> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        let Self { command, message, properties } = self;
        let message = urlencoding::encode(message);

        if let Some(properties) = properties {
            return write!(fmt, "::{command} {properties}::{message}")
        }
        
        write!(fmt, "::{command}::{message}")
    }
}

pub fn issue_command<'a>(command: Command<'a>) {
    command.issue()
}

pub fn issue<'a>(command: impl AsRef<str>, message: impl AsRef<str>) {
    Command::new(command, message).issue()
}