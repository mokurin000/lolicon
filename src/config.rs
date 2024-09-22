use serde::{Deserialize, Serialize};

use lolicon_api::{strum::IntoEnumIterator, Category, ImageSize, Request};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub request: Request,
    pub max_retry: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            request: Request::default()
                .num(1)
                .unwrap()
                .exclude_ai(true)
                .category(Category::R18)
                .aspect_ratio("lt1")
                .unwrap()
                .proxy("i.pixiv.cat")
                .size(ImageSize::iter().collect::<Vec<_>>().as_ref())
                .unwrap(),
            max_retry: 5,
        }
    }
}
