use reqwest::get;
use serde_json::Value;

use lolicon_api::Category;
use lolicon_api::Request;

use lolicon::fetch;
use lolicon::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let req = Request::default()
        .num(1)?
        .exclude_ai(true)
        .category(Category::R18)
        .aspect_ratio("lt1")?
        .proxy("i.pixiv.cat");

    let url = String::from(req);
    println!("quering api: {url}");

    let raw_result = get(url).await?.text().await?;
    let result: Value = serde_json::from_str(&raw_result)?;

    fetch::download_image(result, "images").await?;

    Ok(())
}
