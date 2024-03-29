use std::{

    borrow::{Cow}, 

    fmt::{
    
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 
};

use crate::client::{

    client::{Client, ClientError, ClientResponseError},
    account::{Account},
    
    repository::{

        reference::{
            
            ReferenceError,
            HandleReference,
        },

        commit::{

            CommitError,
            HandleCommit,
        },

        issue::{

            IssueError,
            HandleIssue,
        },
        
        tree::{
    
            TreeError,
            TreeEntry,
            Tree, 
        },

        blob::{

            BlobError,
            Blob,
        },

        sha::{Sha},
    }, 
    
    models::common::repository::{Repository},
    
    GitHubProperties,
};

use serde::{

    Deserialize,
    Serialize,
};

use thiserror::{Error};
use zip::result::{ZipError};

pub mod properties;
pub mod reference;
pub mod commit;
pub mod issue;
pub mod tree;
pub mod blob;
pub mod sha;

use crate::client::{GitHubResult};

use super::{GitHubEndpoint};

#[derive(Clone, Debug)]
pub enum WorkflowStatus {
    ActionRequired,
    Cancelled,
    Completed,
    Failure,
    InProgress,
    Neutral,
    Queued,
    Requested,
    Skipped,
    Stale,
    Success,
    TimedOut,
    Unknown,
    Waiting,
}

impl WorkflowStatus {
    pub fn to_some_str(&self) -> Option<&'static str> {
        match self {
            WorkflowStatus::ActionRequired => Some("action_required"),
            WorkflowStatus::Cancelled => Some("cancelled"),
            WorkflowStatus::Completed => Some("completed"),
            WorkflowStatus::Failure => Some("failure"),
            WorkflowStatus::InProgress => Some("in_progress"),
            WorkflowStatus::Neutral => Some("neutral"),
            WorkflowStatus::Queued => Some("queued"),
            WorkflowStatus::Requested => Some("requested"),
            WorkflowStatus::Skipped => Some("skipped"),
            WorkflowStatus::Stale => Some("stale"),
            WorkflowStatus::Success => Some("success"),
            WorkflowStatus::TimedOut => Some("timed_out"),
            WorkflowStatus::Unknown => None,
            WorkflowStatus::Waiting => Some("waiting"),
        }
    }
}

impl<'a> From<&'a str> for WorkflowStatus {
    fn from(status: &'a str) -> Self {
        match status {
            "action_required" => WorkflowStatus::ActionRequired,
            "cancelled" => WorkflowStatus::Cancelled,
            "completed" => WorkflowStatus::Completed,
            "failure" => WorkflowStatus::Failure,
            "in_progress" => WorkflowStatus::InProgress,
            "neutral" => WorkflowStatus::Neutral,
            "queued" => WorkflowStatus::Queued,
            "requested" => WorkflowStatus::Requested,
            "skipped" => WorkflowStatus::Skipped,
            "stale" => WorkflowStatus::Stale,
            "success" => WorkflowStatus::Success,
            "timed_out" => WorkflowStatus::TimedOut,
            "waiting" => WorkflowStatus::Waiting,
            _ => WorkflowStatus::Unknown,
        }
    }
}

#[derive(Error, Debug)]
pub enum HandleRepositoryError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Reference error!")]
    Reference(#[from] ReferenceError),
    #[error("Commit error!")]
    Commit(#[from] CommitError),
    #[error("Issue error!")]
    Issue(#[from] IssueError),
    #[error("Blob error!")]
    Blob(#[from] BlobError),
    #[error("Tree error!")]
    Tree(#[from] TreeError),
    #[error("Invalid reference: '{name}'")]
    InvalidReference { name: String },
    #[error("Invalid branch: '{name}'")]
    InvalidBranch { name: String },
    #[error("Invalid tag: '{name}'")]
    InvalidTag { name: String },
    #[error("Failed to get default branch: '{name}'")]
    DefaultBranch { name: String },
    #[error("Extraction error!")]
    Archive(#[from] ZipError),
    #[error("Repository not found: '{name}'")]
    Nothing { name: String },
}

#[derive(Clone, Debug)]
pub struct HandleRepository {
    pub(crate) owner: Account,
    pub(crate) name: String,
}

impl HandleRepository {
    pub(crate) fn try_fetch(owner: &Account, name: impl AsRef<str>) -> GitHubResult<HandleRepository, HandleRepositoryError> {
        let name = name.as_ref();

        let components: Vec<_> = name.split('/')
            .collect();

        let name = match components.as_slice() {
            [_, name, _, ..] => name,
            [_, name, ..] => name,
            [name, ..] => name,
            _ => return Err(HandleRepositoryError::Nothing { 
                name: Default::default()
            }),
        };
        
        let response = {

            owner.get_client()
                .get(format!("repos/{owner}/{name}"))?
                .send()?
        };

        if !(response.is_success()) {
            return Err(HandleRepositoryError::Nothing { 
                name: name.to_string() 
            })
        }

        Ok(HandleRepository {
            owner: owner.clone(),
            name: name.to_lowercase(),
        })
    }

    pub(crate) fn try_fetch_all(owner: &Account) -> GitHubResult<Vec<HandleRepository>, HandleRepositoryError> {
        #[derive(Clone, Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            name: String,
        }

        let mut collection = Vec::new();
        let mut page = 0;

        loop {

            page = { page + 1 };

            let capsules: Vec<Capsule> = {
                let ref query = [
                    ("per_page", 100),
                    ("page", page),
                ];

                owner.get_client()
                    .get(format!("users/{owner}/repos"))?
                    .query(query).send()?.json()?
            };

            collection.extend_from_slice({
                capsules.as_slice()
            });

            if capsules.len() < 100 {
                break
            }
        }

        Ok(collection.into_iter().map(|Capsule { name }| HandleRepository { 
            owner: owner.clone(), name: name.to_lowercase()
        }).collect())
    }

    pub fn try_submit_dependency_snapshot(&self, ref payload: impl Serialize) -> GitHubResult<(), HandleRepositoryError> {
        let _ = {

            self.get_client()
                .post(format!("repos/{self}/dependency-graph/snapshots"))?
                .json(payload)
                .send()?
        };

        Ok(())
    }

    /// Gets a list of workflow runs with a given state, returns their run numbers in ascending order.
    pub fn try_get_workflow_runs(&self, status: impl Into<WorkflowStatus>) -> GitHubResult<Vec<usize>, HandleRepositoryError> {
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleRun {
            run_number: usize,
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            total_count: usize,
            workflow_runs: Vec<CapsuleRun>,
        }

        let mut collection = Vec::new();
        let mut page = 0;
        
        let workflow_status: WorkflowStatus = {
            status.into()
        };

        if let Some(status) = workflow_status.to_some_str() {
            
            loop {

                page = { page + 1 };

                #[derive(Serialize)]
                struct Query<'s> {
                    status: &'s str,
                    per_page: usize,
                    page: usize,
                }

                let ref query = Query {
                    status: status.as_ref(),
                    per_page: 100,
                    page,
                };

                let Capsule { total_count, workflow_runs } = {
                    self.get_client().get(format!("repos/{self}/actions/runs"))?
                        .query(query).send()?.json()?
                };

                collection.extend(workflow_runs.iter()
                    .map(|CapsuleRun { run_number }| run_number));

                if total_count < 100 { 
                    break 
                }
            }

            collection.sort();
        }

        Ok(collection)
    }

    /// Gets a list of active workflows, returns their run numbers in ascending order.
    pub fn try_get_active_workflows(&self) -> GitHubResult<Vec<usize>, HandleRepositoryError> {
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleRun {
            run_number: usize,
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            total_count: usize,
            workflow_runs: Vec<CapsuleRun>,
        }

        let mut collection = Vec::new();
        
        collection.extend(self.try_get_workflow_runs(WorkflowStatus::InProgress)?);
        collection.extend(self.try_get_workflow_runs(WorkflowStatus::Requested)?);
        collection.extend(self.try_get_workflow_runs(WorkflowStatus::Waiting)?);
        collection.extend(self.try_get_workflow_runs(WorkflowStatus::Queued)?);

        collection.sort();
        collection.dedup();

        Ok(collection)
    }

    pub fn try_get_issue(&self, id: usize) -> GitHubResult<HandleIssue, HandleRepositoryError> {
        Ok(HandleIssue::try_fetch(self, id)?)
    }

    pub fn try_get_all_issues(&self) -> GitHubResult<Vec<HandleIssue>, HandleRepositoryError> {
        Ok(HandleIssue::try_fetch_all(self)?)
    }

    pub fn try_has_tag(&self, tag: impl AsRef<str>) -> GitHubResult<bool, HandleRepositoryError> {
        Ok(self.try_get_some_tag(tag)?.is_some())
    }

    pub fn try_get_some_tag(&self, tag: impl AsRef<str>) -> GitHubResult<Option<HandleReference>, HandleRepositoryError> {
        let tag = tag.as_ref();

        let candidate = match HandleReference::try_parse(self, tag) {
            Ok(reference) => reference, _ => HandleReference::try_parse(self, {
                format!("tags/{tag}")
            })?,
        };
        
        match self.try_get_some_reference(candidate.to_string())? {
            Some(tag @ HandleReference::Tag { .. }) => Ok(Some(tag)),
            None => Ok(None), _ => Err(HandleRepositoryError::InvalidTag { 
                name: tag.to_owned() 
            })
        }
    }

    pub fn try_get_tag(&self, tag: impl AsRef<str>) -> GitHubResult<HandleReference, HandleRepositoryError> {
        let tag = tag.as_ref();

        let candidate = match HandleReference::try_parse(self, tag) {
            Err(_) => HandleReference::try_parse(self, {
                format!("tags/{tag}")
            })?,
            Ok(reference) => {
                reference
            },
        };

        match self.try_get_reference(candidate.to_string()) {
            Ok(tag @ HandleReference::Tag { .. }) => Ok(tag),
            _ => Err(HandleRepositoryError::InvalidTag { 
                name: tag.to_owned() 
            })
        }
    }

    pub fn try_has_branch(&self, branch: impl AsRef<str>) -> GitHubResult<bool, HandleRepositoryError> {
        Ok(self.try_get_some_branch(branch)?.is_some())
    }

    pub fn try_get_some_branch(&self, branch: impl AsRef<str>) -> GitHubResult<Option<HandleReference>, HandleRepositoryError> {
        let branch = branch.as_ref();

        let candidate = match HandleReference::try_parse(self, branch) {
            Ok(reference) => reference, _ => HandleReference::try_parse(self, {
                format!("heads/{branch}")
            })?,
        };
        
        match self.try_get_some_reference(candidate.to_string())? {
            Some(branch @ HandleReference::Branch { .. }) => Ok(Some(branch)),
            None => Ok(None), _ => Err(HandleRepositoryError::InvalidBranch { 
                name: branch.to_owned() 
            })
        }
    }

    pub fn try_get_branch(&self, branch: impl AsRef<str>) -> GitHubResult<HandleReference, HandleRepositoryError>  {
        let branch = branch.as_ref();

        let candidate = match HandleReference::try_parse(self, branch) {
            Err(_) => HandleReference::try_parse(self, {
                format!("heads/{branch}")
            })?,
            Ok(reference) => {
                reference
            },
        };

        match self.try_get_reference(candidate.to_string()) {
            Ok(branch @ HandleReference::Branch { .. }) => Ok(branch),
            _ => Err(HandleRepositoryError::InvalidBranch { 
                name: branch.to_owned() 
            })
        }
    }

    pub fn try_get_default_branch(&self) -> GitHubResult<HandleReference, HandleRepositoryError>  {
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            default_branch: String,
        }

        let Capsule { default_branch } = self.try_get_properties()?;

        Ok(self.try_get_branch(default_branch.as_str()).map_err(|_| {
            HandleRepositoryError::DefaultBranch { 
                name: default_branch.to_owned() 
            }
        })?)
    }

    pub fn try_has_reference(&self, reference: impl AsRef<str>) -> GitHubResult<bool, HandleRepositoryError> {
        Ok(self.try_get_some_reference(reference)?.is_some())
    }

    pub fn try_get_some_reference(&self, reference: impl AsRef<str>) -> GitHubResult<Option<HandleReference>, HandleRepositoryError> {
        match HandleReference::try_fetch(self, reference) {
            Err(ReferenceError::Nothing { .. }) => Ok(None),
            Err(error) => Err(error.into()),
            Ok(ok) => Ok(Some(ok)),
        }
    }

    pub fn try_get_reference(&self, reference: impl AsRef<str>) -> GitHubResult<HandleReference, HandleRepositoryError> {
        Ok(HandleReference::try_fetch(self, reference)?)
    }

    pub fn try_create_tag(&self, tag: impl AsRef<str>, commit: HandleCommit) -> GitHubResult<HandleReference, HandleRepositoryError>  {
        let tag = tag.as_ref();

        let reference = HandleReference::try_create(self, commit, {
            format!("tags/{tag}")
        })?;
        
        if reference.is_tag() { Ok(reference) } else { 
            Err(HandleRepositoryError::InvalidTag {
                name: tag.to_owned()
            })
        }
    }

    pub fn try_create_branch(&self, branch: impl AsRef<str>, commit: HandleCommit) -> GitHubResult<HandleReference, HandleRepositoryError> {
        let branch = branch.as_ref();

        let reference = HandleReference::try_create(self, commit, {
            format!("heads/{branch}")
        })?;
        
        if reference.is_branch() { Ok(reference) } else { 
            Err(HandleRepositoryError::InvalidBranch {
                name: branch.to_owned()
            })
        }
    }

    pub fn try_create_reference(&self, reference: impl AsRef<str>, commit: HandleCommit) -> GitHubResult<HandleReference, HandleRepositoryError> {
        Ok(HandleReference::try_create(self, commit, reference)?)
    }

    pub fn try_delete_tag(&self, tag: HandleReference) -> GitHubResult<(), HandleRepositoryError> {
        if tag.is_tag() { Ok(tag.try_delete()?) } else {
            Err(HandleRepositoryError::InvalidTag {
                name: tag.to_string()
            })
        }
    }

    pub fn try_delete_branch(&self, branch: HandleReference) -> GitHubResult<(), HandleRepositoryError> {
        if branch.is_branch() { Ok(branch.try_delete()?) } else {
            Err(HandleRepositoryError::InvalidBranch {
                name: branch.to_string()
            })
        }
    }

    pub fn try_delete_reference(&self, reference: HandleReference) -> GitHubResult<(), HandleRepositoryError> {
        Ok(reference.try_delete()?)
    }
    
    pub fn try_get_blob<'a>(&self, sha: impl Into<Sha<'a>>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_fetch(self, sha)?)
    }

    pub fn try_create_binary_blob(&self, content: impl AsRef<[u8]>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_create_binary_blob(self, content)?)
    }

    pub fn try_create_text_blob(&self, content: impl AsRef<str>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_create_text_blob(self, content)?)
    }   

    pub fn try_get_tree<'a>(&self, sha: impl Into<Sha<'a>>, recursive: bool) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_fetch(self, sha, recursive)?)
    }

    pub fn try_create_tree(&self, entries: impl AsRef<[TreeEntry]>) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_create(self, entries)?)
    }

    pub fn try_create_tree_with_base(&self, base: HandleCommit, entries: impl AsRef<[TreeEntry]>) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_create_with_base(self, base, entries)?)
    }

    pub fn try_get_commit<'a>(&self, commit: impl Into<Sha<'a>>) -> GitHubResult<HandleCommit, HandleRepositoryError> {
        Ok(HandleCommit::try_fetch(self, commit)?)
    }

    pub fn try_has_commit<'a>(&self, commit: impl Into<Sha<'a>>) -> GitHubResult<bool, HandleRepositoryError> {
        match HandleCommit::try_fetch(self, commit) {
            Err(CommitError::Client(ClientError::Response(ClientResponseError::Nothing { .. }))) => Ok(false),
            Err(error) => Err(error.into()),
            Ok(_) => Ok(true),
        }
    }

    pub fn try_create_commit(&self, parents: impl AsRef<[HandleCommit]>, tree: Tree, message: impl AsRef<str>) -> GitHubResult<HandleCommit, HandleRepositoryError> { 
        Ok(HandleCommit::try_create(self, parents, tree, message)?) 
    }
}

impl<'a> GitHubEndpoint<'a> for HandleRepository {
    fn get_endpoint(&'a self) -> Cow<'a, str> {
        format!("repos/{self}").into()
    }
}

impl<'a> GitHubProperties<'a> for HandleRepository {
    type Content = Repository;
    type Parent = Account;

    fn get_client(&'a self) -> &'a Client {
        self.get_parent()
            .get_client()
    }
    
    fn get_parent(&'a self) -> &'a Self::Parent {
        &(self.owner)
    }
}

impl FmtDisplay for HandleRepository {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{owner}/{name}", owner = self.owner, name = self.name)
    }
}