use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Systemd error: {0}")]
    Systemd(String),

    #[error("ACME error: {0}")]
    Acme(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Subscription error: {0}")]
    Subscription(String),

    #[error("Journal error: {0}")]
    Journal(String),

    #[error("Command error: {0}")]
    Command(String),
}
