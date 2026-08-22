use thiserror::Error;

#[derive(Debug, Error)]
pub enum LocalDockError {
    #[error("path escapes allowed roots: {0}")]
    PathNotAllowed(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("app not found: {0}")]
    AppNotFound(String),
    #[error("invalid command")]
    InvalidCommand,
    #[error("invalid registry app {app}: {reason}")]
    InvalidRegistryApp { app: String, reason: String },
    #[error("already running")]
    AlreadyRunning,
    #[error("not running")]
    NotRunning,
    #[error("{0}")]
    StartFailed(String),
}
