use strum::EnumIs;
use thiserror::Error;

#[derive(Debug, Error, EnumIs)]
pub enum LoliconError {
    #[error("404 not found {0}")]
    NotFound(u64),
    #[error("size not found")]
    SizeNotFound,
    #[error("exceeded max retry times!")]
    RetryExceed,
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Url(#[from] url::ParseError),
    #[error("Invalid JSON")]
    ParseError(#[from] serde_json::Error),
}
