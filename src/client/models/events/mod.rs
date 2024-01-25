use serde::{
    
    Deserialize,
    Serialize, 
};

pub mod payloads;
pub use payloads::{
    
    EventIssueComment,
    EventIssue, 
};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "event_name", content = "event")]
pub enum Event {
    #[serde(rename = "issue_comment")]
    IssueComment(EventIssueComment),
    #[serde(rename = "issues")]
    Issue(EventIssue),
    #[serde(rename = "schedule")]
    Schedule,
}

impl Event {
    pub fn is_bot_generated(&self) -> bool {
        match self {
            Event::IssueComment(comment) => match comment {
                EventIssueComment::Deleted { comment, .. } |
                EventIssueComment::Created { comment, .. } |
                EventIssueComment::Edited { comment, .. } => {
                    comment.get_author()
                        .is_bot()
                },
            },
            Event::Issue(issue) => match issue {
                EventIssue::Reopened { issue, .. } |
                EventIssue::Deleted { issue, .. } |
                EventIssue::Opened { issue, .. } |
                EventIssue::Closed { issue, .. } |
                EventIssue::Edited { issue, .. } => {
                    issue.get_author()
                        .is_bot()
                },
            },
            _ => false,
        }
    }
}