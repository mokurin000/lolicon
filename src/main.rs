use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::RwLock;

use lolicon::error::LoliconError;
use lolicon_api::Setu;
use reqwest::Client;
use tokio::fs;

use lolicon::fetch;
use lolicon::AnyResult;

mod config;
use config::Config;
mod storage;
use storage::Storage;

const CONFIG_FILE: &str = "config.toml";
const STORAGE_FILE: &str = "storage.json";

static STORAGE: LazyLock<Arc<RwLock<Storage>>> = LazyLock::new(|| {
    let storage = if std::fs::exists(STORAGE_FILE).unwrap() {
        Storage::from_file(STORAGE_FILE).unwrap()
    } else {
        Storage::new()
    };
    Arc::new(RwLock::new(storage))
});

#[tokio::main]
async fn main() -> AnyResult<()> {
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

    let client = Client::new();
    let raw_result = client.get(url).send().await?.text().await?;
    let result: Setu = serde_json::from_str(&raw_result)?;

    if !result.error.is_empty() {
        eprintln!("错误：{}", result.error);
        std::process::exit(1);
    }

    let results = fetch::download_images(
        result,
        config.output_dir,
        config.target_size,
        config.max_retry,
        config.save_metadata,
        &client,
        Some(&|pid| {
            if let Ok(guard) = STORAGE.read() {
                guard.contains(&pid)
            } else {
                false
            }
        }),
    )
    .await;

    {
        let mut guard = STORAGE.write().expect("write storage failed");
        for error in results.into_iter().filter_map(Result::err) {
            let LoliconError::NotFound(pid) = error else {
                continue;
            };
            guard.store(pid);
        }
    }

    let guard = STORAGE.read().expect("read storage failed");
    guard.write_file(STORAGE_FILE)?;

    Ok(())
}
