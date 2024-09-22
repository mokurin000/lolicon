use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use lolicon_api::{Setu, SetuData};
use reqwest::get;
use tokio::{fs, task::JoinSet};
use url::Url;

use crate::Result;

/// returns path to the downloaded image
pub async fn download_image_data(
    data: &SetuData,
    output_dir: &Path,
    size: lolicon_api::ImageSize,
    max_retry: usize,
    save_metadata: bool,
) -> Result<PathBuf> {
    let pid = data.pid;
    eprintln!("pid: {pid}");

    let image_url = match size {
        lolicon_api::ImageSize::Original => &data.urls.original,
        lolicon_api::ImageSize::Regular => &data.urls.regular,
        lolicon_api::ImageSize::Small => &data.urls.small,
        lolicon_api::ImageSize::Thumb => &data.urls.thumb,
        lolicon_api::ImageSize::Mini => &data.urls.mini,
    }
    .as_deref()
    .ok_or(format!("missing size {size}!"))?;

    fs::create_dir_all(output_dir).await?;

    let url = url::Url::from_str(&image_url)?;
    let basename = url.path_segments().unwrap().last().unwrap();
    let target_path = output_dir.join(basename);

    if save_metadata {
        let mut metadata_path = target_path.clone();
        metadata_path.set_extension("json");
        eprintln!("writing metadata...");
        fs::write(&metadata_path, serde_json::to_string(&data)?).await?;
    }

    if target_path.exists() {
        println!("skipping existing image.");
        return Ok(target_path);
    }
    eprintln!("downloading {image_url}...",);

    let image = download_retry(&url, max_retry, 500).await?;
    eprintln!("writing image...");
    fs::write(&target_path, &image).await?;
    Ok(target_path)
}

/// download each image in data from `setu`
pub async fn download_images(
    setu: Setu,
    output_dir: impl AsRef<Path>,
    size: lolicon_api::ImageSize,
    max_retry: usize,
    save_metadata: bool,
) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();

    let mut tasks = JoinSet::new();
    for data in &setu.data {
        let data = data.clone();
        let output_dir = output_dir.as_ref().to_path_buf();
        tasks.spawn(async move {
            download_image_data(&data, output_dir.as_ref(), size, max_retry, save_metadata).await
        });
    }

    while let Some(result) = tasks.join_next().await {
        let Ok(result) = result else {
            continue;
        };

        match result {
            Ok(path) => results.push(path),
            Err(e) => {
                eprintln!("download failed: {e}");
            }
        }
    }

    Ok(results)
}

/// download an image to bytes, return error on 404 page
///
/// if `max_retry` was set to zero, it will never download.
pub async fn download_retry(
    url: &Url,
    max_retry: usize,
    initial_wait_ms: u64,
) -> Result<bytes::Bytes> {
    let mut image = Err("exceeding retry limit");

    let mut wait_time_ms = initial_wait_ms;
    for _ in 0..max_retry {
        let result = get(url.as_str()).await;
        if let Ok(resp) = result {
            let bytes = resp.bytes().await?;
            if bytes.is_ascii() {
                Err("Image not found! may removed by its author")?
            }
            image = Ok(bytes);
            break;
        }

        eprintln!("download failed, will retry after {wait_time_ms}ms...");
        tokio::time::sleep(Duration::from_millis(wait_time_ms)).await;
        wait_time_ms *= 2;
    }

    Ok(image?)
}
