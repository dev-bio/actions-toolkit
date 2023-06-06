use serde::{Serialize};

use super::error::{CoreError};

pub fn to_command_value(ref value: impl Serialize) -> Result<String, CoreError> {
    Ok(serde_json::to_string(value)?)
}