use std::{

    collections::{HashSet},
    
    fmt::{
    
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    },
};

use thiserror::{Error};
use serde::{Deserialize};

use crate::client::{

    repository::{

        commit::{HandleCommit},
        sha::{Sha},

        HandleRepositoryError,
        HandleRepository,
    },

    client::{

        ClientResponseError,
        ClientError,
        Client,
    },
    
    Number, GitHubProperties,
};

use crate::client::{GitHubResult};

#[derive(Debug, Error)]
pub enum ReferenceError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Invalid reference: '{reference}'")]
    Invalid { reference: String },
    #[error("Reference not found: '{reference}'")]
    Nothing { reference: String },
    #[error("Circular reference: '{reference}'")]
    Circular { reference: String },
    #[error("Reference is deleted!")]
    Delete,
}

#[derive(Clone, Debug)]
pub enum HandleReference {
    PullRequest { repository: HandleRepository, branch: String, issue: Number },
    Branch { repository: HandleRepository, branch: String },
    Tag { repository: HandleRepository, tag: String },
}

impl HandleReference {
    pub(crate) fn try_parse(repository: &HandleRepository, reference: impl AsRef<str>) -> GitHubResult<HandleReference, ReferenceError> {
        let reference = reference.as_ref();

        let tokens: Vec<_> = reference.split('/')
            .collect();

        let kind = match tokens.as_slice() {
            ["refs", "pull", issue, branch] |
            ["pull", issue, branch] => HandleReference::PullRequest {
                repository: repository.clone(), branch: branch.to_string(), issue: issue.parse().map_err(|_| {
                    ReferenceError::Invalid { reference: reference.to_owned() }
                })?,
            },
            ["refs", "pull", issue, branch @ ..] |
            ["pull", issue, branch @ ..] => HandleReference::PullRequest {
                repository: repository.clone(), branch: branch.join("/"), issue: issue.parse().map_err(|_| {
                    ReferenceError::Invalid { reference: reference.to_owned() }
                })?,
            },
            ["refs", "heads", branch] |
            ["heads", branch] => HandleReference::Branch {
                repository: repository.clone(), branch: branch.to_string(),
            },
            ["refs", "heads", branch @ ..] |
            ["heads", branch @ ..] => HandleReference::Branch {
                repository: repository.clone(), branch: branch.join("/"),
            },
            ["refs", "tags", tag] |
            ["tags", tag] => HandleReference::Tag {
                repository: repository.clone(), tag: tag.to_string(),
            },
            ["refs", "tags", tag @ ..] |
            ["tags", tag @ ..] => HandleReference::Tag {
                repository: repository.clone(), tag: tag.join("/"),
            },
            _ => return Err(ReferenceError::Invalid {
                reference: reference.to_owned()
            })
        };

        Ok(kind)
    }

    pub(crate) fn try_fetch(repository: &HandleRepository, reference: impl AsRef<str>)  -> GitHubResult<HandleReference, ReferenceError> {
        let reference = reference.as_ref();

        let parsed = Self::try_parse(repository, {
            reference
        })?;

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            #[serde(rename = "ref")]
            name: String,
        }

        let Capsule { name } = { 

            let result = {

                repository.get_client()
                    .get(format!("repos/{repository}/git/ref/{parsed}"))?
                    .send()
            };

            match result {
                Err(ClientError::Response(ClientResponseError::Nothing { .. })) => return Err(ReferenceError::Nothing {
                    reference: reference.to_string()
                }), 
                Err(error) => return Err(ReferenceError::Client({
                    error
                })),
                Ok(response) => response.json()?
            }
        };

        if name.ends_with(reference) { Ok(parsed) } else { 
            Err(ReferenceError::Nothing {
                reference: reference.to_string()
            })
        }
    }

    pub(crate) fn try_create(repository: &HandleRepository, commit: HandleCommit, reference: impl AsRef<str>) -> GitHubResult<HandleReference, ReferenceError> {
        let reference = reference.as_ref();
        let parsed = Self::try_parse(repository, {
            reference
        })?;
        
        let ref payload = serde_json::json!({
            "ref": format!("refs/{parsed}"),
            "sha": commit.get_sha(),
        });

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            #[serde(rename = "ref")]
            name: String,
        }

        let Capsule { name } = { 

            let result = {

                repository.get_client()
                    .post(format!("repos/{repository}/git/refs"))?
                    .json(payload)
                    .send()
            };

            match result {
                Err(ClientError::Response(ClientResponseError::Nothing { .. })) => return Err(ReferenceError::Nothing {
                    reference: reference.to_string()
                }), 
                Err(error) => return Err(ReferenceError::Client({
                    error
                })),
                Ok(response) => response.json()?
            }
        };

        if name.ends_with(reference) { Ok(parsed) } else { 
            Err(ReferenceError::Nothing {
                reference: reference.to_string()
            })
        }
    }

    pub fn try_set_commit<'a>(&self, force: bool, commit: impl Into<Sha<'a>>) -> GitHubResult<(), HandleRepositoryError> {
        let repository = self.get_repository();

        let ref payload = serde_json::json!({
            "sha": commit.into(),
            "force": force,
        });
        
        repository.get_client()
            .patch(format!("repos/{repository}/git/refs/{self}"))?
            .json(payload)
            .send()?;

        Ok(())
    }

    pub fn try_get_commit(&self) -> GitHubResult<HandleCommit, HandleRepositoryError> {
        let repository = self.get_repository();
        let client = self.get_client();

        #[derive(Debug)]
        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum CapsuleReference {
            #[serde(rename = "commit")]
            Commit { sha: Sha<'static> },
            #[serde(rename = "tag")]
            Tag { sha: Sha<'static> },
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            object: CapsuleReference
        }

        let mut visited = HashSet::new();
        let mut capsule = {
            client.get(format!("repos/{repository}/git/ref/{self}"))?
                .send()?.json()?
        };

        let sha = loop {
            let Capsule { object } = {
                capsule
            };

            match object {
                CapsuleReference::Commit { sha } => {
                    break sha
                },
                CapsuleReference::Tag { ref sha } => {
                    if visited.contains(sha) { 
                        return Err(HandleRepositoryError::Reference({
                            ReferenceError::Circular {
                                reference: self.to_string()
                            }
                        }))
                    }
                    
                    visited.insert(sha.clone());
                    capsule = client.get(format!("repos/{repository}/git/tags/{sha}"))?
                        .send()?.json()?;
                },
            }
        };

        repository.try_get_commit(sha)
    }

    pub(crate) fn try_delete(&self) -> GitHubResult<(), ReferenceError> {
        let repository = self.get_repository();

        repository.get_client()
            .delete(format!("repos/{repository}/git/refs/{self}"))?
            .send()?;

        Ok(())
    }

    pub(crate) fn get_client(&self) -> &Client {
        match self {
            HandleReference::PullRequest { repository, .. } => repository.get_client(),
            HandleReference::Branch { repository, .. } => repository.get_client(),
            HandleReference::Tag { repository, .. } => repository.get_client(),
        }
    }

    pub fn get_repository(&self) -> HandleRepository {
        match self {
            HandleReference::PullRequest { repository, .. } => repository.clone(),
            HandleReference::Branch { repository, .. } => repository.clone(),
            HandleReference::Tag { repository, .. } => repository.clone(),
        }
    }

    pub fn is_pull_request(&self) -> bool {
        match self {
             HandleReference::PullRequest { .. } => true,
             _ => false,
        }
    }

    pub fn is_branch(&self) -> bool {
        match self {
             HandleReference::Branch { .. } => true,
             _ => false,
        }
    }

    pub fn is_tag(&self) -> bool {
        match self {
             HandleReference::Tag { .. } => true,
             _ => false,
        }
    }
}

impl FmtDisplay for HandleReference {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        match self {
            HandleReference::PullRequest { branch, issue, .. } => write!(fmt, "pull/{issue}/{branch}"),
            HandleReference::Branch { branch, .. } => write!(fmt, "heads/{branch}"),
            HandleReference::Tag { tag, .. } => write!(fmt, "tags/{tag}"),
        }
    }
}