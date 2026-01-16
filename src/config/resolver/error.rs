use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("'{0}' not found")]
    NotFound(String),
    #[error("No groups defined in config")]
    NoGroups,
    #[error("Group '{0}' references '{1}' which is not found")]
    GroupMemberNotFound(String, String),
    #[error("Group '{0}' failed to load member: {1}")]
    GroupMemberError(String, Box<ResolveError>),
    #[error("Found no name to resolve")]
    NoNameToResolve,
}
