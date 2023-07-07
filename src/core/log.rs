use std::{

    sync::{
        
        Mutex, 
        Arc,
    },

    cell::{RefCell},
};

use once_cell::sync::{Lazy};
use thiserror::{Error};

use super::error::{CoreError};

static LOG_WRITER: Lazy<Mutex<LogWriter>> = Lazy::new(|| {
    Mutex::new(LogWriter)
});

thread_local!{
    
    static LOG_STATE: RefCell<LogState> = RefCell::new(LogState::new());

}


#[derive(Error, Debug)]
pub enum LogError {
    #[error("failed to write log, reason: {0}")]
    Write(String),
}

struct LogWriter;

impl LogWriter {
    pub fn write(&self, ref group: Log) -> Result<(), CoreError> {
        match group {
            Log::Group(LogGroup { group, lines }) => {
                super::command::issue_message("group", group)?;

                for line in lines.iter() {
                    self.write(Log::Line({
                        line.clone()
                    }))?;
                }

                super::command::issue("endgroup")?
            },
            Log::Line(line) => {
                match line {
                    LogLine::Debug(message) => {
                        super::command::issue_message("debug", message)?
                    },
                    LogLine::Notice(message) => {
                        super::command::issue_message("notice", message)?
                    },
                    LogLine::Warning(message) => {
                        super::command::issue_message("warning", message)?
                    },
                    LogLine::Error(message) => {
                        super::command::issue_message("error", message)?
                    },
                }
            },
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
enum LogLine {
    Debug(String),
    Notice(String),
    Warning(String),
    Error(String),
}

#[derive(Clone, Debug)]
struct LogGroup {
    group: String,
    lines: Vec<LogLine>,
}

impl LogGroup {
    fn new(name: impl AsRef<str>) -> Self {
        let name = name.as_ref();

        Self {
            group: name.to_owned(),
            lines: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
enum Log {
    Group(LogGroup),
    Line(LogLine),
}

#[derive(Debug)]
struct LogState {
    state: Arc<Mutex<Option<LogGroup>>>,
}

impl LogState {
    fn new() -> Self {
        Self {
            state: Arc::new({
                Mutex::new(None)
            }),
        }
    }

    fn begin_group(&self, name: impl AsRef<str>) -> Result<(), CoreError> {
        let mut state = self.state.lock().map_err(|_| {
            CoreError::Log(LogError::Write(format! {
                "failed to lock log state!"
            }))
        })?;

        if let Some(state) = state.take() {
            let writer = LOG_WRITER.lock().map_err(|_| {
                CoreError::Log(LogError::Write(format! {
                    "failed to lock log group!"
                }))
            })?;

            writer.write(Log::Group(state))?
        }

        let _ = state.insert({
            LogGroup::new(name)
        });

        Ok(())
    }

    fn end_group(&self) -> Result<(), CoreError> {
        let mut state = self.state.lock().map_err(|_| {
            CoreError::Log(LogError::Write(format! {
                "failed to lock log state!"
            }))
        })?;

        if let Some(state) = state.take() {
            let writer = LOG_WRITER.lock().map_err(|_| {
                CoreError::Log(LogError::Write(format! {
                    "failed to lock log group!"
                }))
            })?;

            writer.write(Log::Group(state))?
        }

        Ok(())
    }

    fn log(&self, line: LogLine) -> Result<(), CoreError> {

        {
            let mut state = self.state.lock().map_err(|_| {
                CoreError::Log(LogError::Write(format! {
                    "failed to lock log state!"
                }))
            })?;

            if let Some(state) = state.as_mut() {
                return Ok(state.lines.push(line));
            }
        }

        let writer = LOG_WRITER.lock().map_err(|_| {
            CoreError::Log(LogError::Write(format! {
                "failed to lock log writer!"
            }))
        })?;

        Ok(writer.write(Log::Line(line))?)
    }
}

impl Drop for LogState {
    fn drop(&mut self) {
        let _ = self.end_group();
    }
}

pub fn debug(message: impl AsRef<str>) -> Result<(), CoreError>  {
    Ok(LOG_STATE.with(|state| state.borrow().log(LogLine::Debug({
        message.as_ref().to_owned()
    })))?)
}

pub fn notice(message: impl AsRef<str>) -> Result<(), CoreError>  {
    Ok(LOG_STATE.with(|state| state.borrow().log(LogLine::Notice({
        message.as_ref().to_owned()
    })))?)
}

pub fn warning(message: impl AsRef<str>) -> Result<(), CoreError>  {
    Ok(LOG_STATE.with(|state| state.borrow().log(LogLine::Warning({
        message.as_ref().to_owned()
    })))?)
}

pub fn error(message: impl AsRef<str>) -> Result<(), CoreError>  {
    Ok(LOG_STATE.with(|state| state.borrow().log(LogLine::Error({
        message.as_ref().to_owned()
    })))?)
}

pub fn begin_group(name: impl AsRef<str>) -> Result<(), CoreError>  {
    Ok(LOG_STATE.with(|state| state.borrow().begin_group(name))?)
    
}

pub fn end_group() -> Result<(), CoreError>  {
    Ok(LOG_STATE.with(|state| state.borrow().end_group())?)
}