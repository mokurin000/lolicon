use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoliconError {
    #[error("404 not found")]
    NotFound,
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
}
