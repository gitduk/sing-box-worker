use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Missing field: {0}")]
    MissingField(&'static str),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("{0}")]
    Worker(#[from] worker::Error),
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::MissingField(_) => 400,
            AppError::InvalidFormat(_) => 400,
            AppError::UrlParse(_) => 400,
            AppError::Base64Decode(_) => 400,
            AppError::Utf8(_) => 400,
            AppError::Json(_) => 400,
            AppError::Worker(_) => 500,
        }
    }
}

impl From<AppError> for worker::Error {
    fn from(e: AppError) -> Self {
        worker::Error::RustError(e.to_string())
    }
}
