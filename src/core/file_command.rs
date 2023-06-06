use std::{
    
    fmt::{Display},
    fs::{OpenOptions},
};

use serde::{Serialize};
use uuid::{Uuid};

use super::{
    
    error::{CoreError},
    util,
};

pub fn issue_file_command(command: impl Display, message: impl AsRef<str> + Serialize) -> Result<(), CoreError> {
    let path = std::env::var(format!("GITHUB_{command}"))?;
    let mut file = OpenOptions::new()
        .create(false)
        .append(true)
        .write(true)
        .open(path)?;

    let command_value = util::to_command_value(message)?;

    use std::io::{Write};

    Ok(write!(file, "{command_value}")?)
}

pub fn prepare_key_value_message(key: impl AsRef<str>, value: impl Serialize) -> Result<String, CoreError> {
    let delimiter = format!("ghadelimiter_{uuid}", uuid = Uuid::new_v4());

    let key = key.as_ref();
    let value = util::to_command_value(value)?;

    if key.contains(delimiter.as_str()) {
        return Err(CoreError::IllegalKeyContent({
            key.to_owned()
        }))
    }

    if value.contains(delimiter.as_str()) {
        return Err(CoreError::IllegalKeyContent({
            value
        }))
    }

    let mut message = String::new();

    use std::fmt::{Write};

    {
        writeln!(message, "{key}<<{delimiter}")?;
        
        {
            writeln!(message, "{value}")?;
        }

        write!(message, "{delimiter}")?;
    }

    Ok(message)
}