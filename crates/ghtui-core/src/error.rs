use thiserror::Error;

#[derive(Error, Debug)]
pub enum GhtuiError {
    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("API error: {status} {message}")]
    Api { status: u16, message: String },

    #[error("Rate limit exceeded, resets at {reset_at}")]
    RateLimit { reset_at: i64 },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}
