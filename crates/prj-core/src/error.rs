use std::path::PathBuf;

/// Errors produced by core `prj` operations.
#[derive(Debug, thiserror::Error)]
pub enum PrjError {
    #[error("project not found: {0}")]
    ProjectNotFound(String),

    #[error("project already registered: {0}")]
    ProjectAlreadyExists(String),

    #[error("path does not exist: {}", .0.display())]
    PathNotFound(PathBuf),

    #[error("path is not a directory: {}", .0.display())]
    NotADirectory(PathBuf),

    #[error("failed to read database: {0}")]
    DatabaseRead(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("failed to write database: {0}")]
    DatabaseWrite(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("git clone failed: {0}")]
    CloneFailed(String),

    #[error("could not determine clone destination from args: {0}")]
    CloneDestUnknown(String),

    #[error("no target projects specified (use --project, --tag, or --all)")]
    NoTargetProjects,

    #[error("manifest error: {0}")]
    Manifest(String),
}
