use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Email sending failed: {0}")]
    EmailError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Server error: {0}")]
    ServerError(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
