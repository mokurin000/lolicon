use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::RwLock;

use lolicon_api::{Setu, SetuData};
use reqwest::Client;
use tokio::fs;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::FmtSubscriber;

use lolicon::error::LoliconError;
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
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config;

    if !fs::try_exists(CONFIG_FILE).await? {
        config = Config::default();
        fs::write(CONFIG_FILE, toml::to_string(&config)?).await?;
    } else {
        config = toml::from_str(&fs::read_to_string(CONFIG_FILE).await?)?;
    }

    let req = config.request;

    let url = String::from(req);
    info!("quering api: {url}");

    let client = Client::new();
    let raw_result = client.get(url).send().await?.text().await?;
    let result: Setu = serde_json::from_str(&raw_result)?;

    if !result.error.is_empty() {
        error!("请求失败：{}", result.error);
        std::process::exit(1);
    }

    std::fs::create_dir_all(&config.output_dir)?;

    if config.save_metadata {
        info!("saving metadata...");
        for data in &result.data {
            let image_url = fetch::get_url_by_size(data, config.target_size)?;
            let mut metadata_path = fetch::get_target_path(&config.output_dir, image_url)?;
            metadata_path.set_extension("json");

            fs::write(metadata_path, serde_json::to_string_pretty(data)?).await?;
        }
    }

    if !config.save_images {
        info!("skip downloading images...");
        return Ok(());
    }

    let results = fetch::download_images(
        result.clone(),
        &config.output_dir,
        config.target_size,
        config.max_retry,
        &client,
        Some(&|data: &SetuData| {
            let pid = data.pid as u64;
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
        guard.write_file(STORAGE_FILE)?;
    }

    Ok(())
}
