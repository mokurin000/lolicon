use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use lolicon_api::{Setu, SetuData};
use reqwest::get;
use tokio::fs;
use url::Url;

use crate::Result;

pub async fn download_image_data(
    data: &SetuData,
    output_dir: &Path,
    size: lolicon_api::ImageSize,
    max_retry: usize,
    results: &mut Vec<PathBuf>,
) -> Result<()> {
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
    let mut metadata_path = target_path.clone();
    metadata_path.set_extension(".json");

    println!("writing metadata...");
    fs::write(&metadata_path, serde_json::to_string(&data)?).await?;

    if target_path.exists() {
        println!("skipping existing image.");
        results.push(target_path);
        return Ok(());
    }
    println!("downloading {image_url}...",);

    match download_retry(&url, max_retry).await {
        Ok(image) => {
            println!("writing image...");
            fs::write(&target_path, &image).await?;
            results.push(target_path);
        }
        Err(reason) => {
            println!("download failed. {reason}");
        }
    }

    Ok(())
}

pub async fn download_images(
    result: Setu,
    output_dir: impl AsRef<Path>,
    size: lolicon_api::ImageSize,
    max_retry: usize,
) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();

    for data in &result.data {
        download_image_data(data, output_dir.as_ref(), size, max_retry, &mut results).await?
    }

    Ok(results)
}

pub async fn download_retry(url: &Url, max_retry: usize) -> Result<bytes::Bytes> {
    let mut image = Err("exceeding retry limit");

    let mut wait_time_ms = 500;
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

        println!("download failed, will retry after {wait_time_ms}ms...");
        tokio::time::sleep(Duration::from_millis(wait_time_ms)).await;
        wait_time_ms *= 2;
    }

    Ok(image?)
}
