use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub mod fetch;

pub use bytes;
pub use serde_json;
pub use url;

pub use lolicon_api;
