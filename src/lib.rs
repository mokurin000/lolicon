use std::error::Error;

pub type AnyResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub type Result<T> = std::result::Result<T, error::LoliconError>;

pub mod fetch;

pub use bytes;
pub use reqwest;
pub use serde_json;
pub use url;

pub use lolicon_api;

pub mod error;
