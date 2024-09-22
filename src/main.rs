use config::Config;
use reqwest::get;

use lolicon_api::ImageSize;
use lolicon_api::Setu;

use lolicon::fetch;
use lolicon::Result;
use tokio::fs;

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
    println!("quering api: {url}");

    let raw_result = get(url).await?.text().await?;
    let result: Setu = serde_json::from_str(&raw_result)?;

    fetch::download_images(result, "images", ImageSize::Original, config.max_retry).await?;

    Ok(())
}
