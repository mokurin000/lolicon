use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub mod fetch;

pub use bytes;
pub use serde_json;
pub use url;
pub use reqwest;

pub use lolicon_api;
