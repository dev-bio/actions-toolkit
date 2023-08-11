pub fn debug(message: impl AsRef<str>)  {
    match super::command::issue_message("debug", message.as_ref()) {
        Err(_) => log::debug!("{message}", message = {
            message.as_ref()
        }),
        _ => return
    }
}

pub fn notice(message: impl AsRef<str>) {
    match super::command::issue_message("notice", message.as_ref()) {
        Err(_) => log::info!("{message}", message = {
            message.as_ref()
        }),
        _ => return
    }
}

pub fn warning(message: impl AsRef<str>) {
    match super::command::issue_message("warning", message.as_ref()) {
        Err(_) => log::warn!("{message}", message = {
            message.as_ref()
        }),
        _ => return
    }
}

pub fn error(message: impl AsRef<str>) {
    match super::command::issue_message("error", message.as_ref()) {
        Err(_) => log::error!("{message}", message = {
            message.as_ref()
        }),
        _ => return
    }
}

pub fn begin_group(name: impl AsRef<str>) {
    match super::command::issue_message("group", name.as_ref()) {
        _ => return
    }
}

pub fn end_group() {
    match super::command::issue("endgroup") {
        _ => return
    }
}