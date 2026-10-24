use thiserror::Error;

/// Provider-specific errors
#[derive(Error)]
pub enum ProviderError {
    #[error("HTTP request failed")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON serialization failed")]
    SerializationError(#[from] serde_json::Error),

    #[error("Model not supported by provider")]
    ModelNotSupported(String),

    #[error("Provider API error: {status}")]
    ApiError { status: u16, message: String },

    #[error("Provider configuration error")]
    ConfigError(String),

    #[error("Provider authentication error")]
    AuthError(String),
}

impl std::fmt::Debug for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
