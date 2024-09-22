use std::time::Duration;

use config::Config;
use reqwest::get;

use lolicon_api::ImageSize;
use lolicon_api::Setu;

use lolicon::fetch;
use lolicon::Result;
use tokio::fs;
use tokio::time;

mod config;

const CONFIG_FILE: &str = "config.toml";

#[tokio::main]
async fn main() -> Result<()> {
    let config;

    if !fs::try_exists(CONFIG_FILE).await? {
        config = Config::default();
        fs::write(CONFIG_FILE, toml::to_string(&config)?).await?;
    } else {
        config = toml::from_str(&fs::read_to_string(CONFIG_FILE).await?)?;
    }

    let req = config.request;

    let url = String::from(req);
    eprintln!("quering api: {url}");

    let raw_result = get(url).await?.text().await?;
    let result: Setu = serde_json::from_str(&raw_result)?;

    if !result.error.is_empty() {
        eprintln!("错误：{}", result.error);
        std::process::exit(1);
    }

    fetch::download_images(
        result,
        "images",
        ImageSize::Original,
        config.max_retry,
        config.save_metadata,
    )
    .await?;

    Ok(())
}
