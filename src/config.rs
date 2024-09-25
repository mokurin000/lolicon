use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use lolicon_api::{strum::IntoEnumIterator, Category, ImageSize, Request};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub target_size: ImageSize,
    pub output_dir: PathBuf,
    pub save_metadata: bool,
    pub save_images: bool,
    pub max_retry: usize,
    pub request: Request,
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
            save_metadata: true,
            target_size: ImageSize::Original,
            output_dir: PathBuf::from("images"),
            save_images: true,
        }
    }
}
