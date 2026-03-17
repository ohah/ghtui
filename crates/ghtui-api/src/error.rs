use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error {status}: {message}")]
    GitHub { status: u16, message: String },

    #[error("Rate limit exceeded, resets at {reset_at}")]
    RateLimit { reset_at: i64, remaining: u32 },

    #[error("Authentication required")]
    Unauthorized,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

impl From<ApiError> for ghtui_core::GhtuiError {
    fn from(e: ApiError) -> Self {
        match e {
            ApiError::Unauthorized => ghtui_core::GhtuiError::Auth("Unauthorized".into()),
            ApiError::RateLimit { reset_at, .. } => {
                ghtui_core::GhtuiError::RateLimit { reset_at }
            }
            ApiError::NotFound(msg) => ghtui_core::GhtuiError::NotFound(msg),
            ApiError::GitHub { status, message } => {
                ghtui_core::GhtuiError::Api { status, message }
            }
            ApiError::Http(e) => ghtui_core::GhtuiError::Network(e.to_string()),
            ApiError::Json(e) => ghtui_core::GhtuiError::Parse(e.to_string()),
            ApiError::Other(msg) => ghtui_core::GhtuiError::Other(msg),
        }
    }
}
