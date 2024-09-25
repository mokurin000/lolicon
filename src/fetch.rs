use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use bytes::Bytes;
use lolicon_api::{ImageSize, Setu, SetuData};
use reqwest::Client;
use tokio::{fs, task::JoinSet};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::error::LoliconError;
use crate::Result;

#[derive(Debug, Clone, Default)]
pub struct Downloaded {
    pub path: PathBuf,
    pub raw_image: Option<Bytes>,
    pub data: SetuData,
}

pub fn get_target_path(output_dir: impl AsRef<Path>, url: impl AsRef<str>) -> Result<PathBuf> {
    let url = Url::from_str(url.as_ref())?;
    let basename = url.path_segments().unwrap().last().unwrap();
    let target_path = output_dir.as_ref().join(basename);

    Ok(target_path)
}

pub fn get_url_by_size(data: &SetuData, size: ImageSize) -> Result<&str> {
    match size {
        lolicon_api::ImageSize::Original => &data.urls.original,
        lolicon_api::ImageSize::Regular => &data.urls.regular,
        lolicon_api::ImageSize::Small => &data.urls.small,
        lolicon_api::ImageSize::Thumb => &data.urls.thumb,
        lolicon_api::ImageSize::Mini => &data.urls.mini,
    }
    .as_deref()
    .ok_or(LoliconError::SizeNotFound)
}

/// returns path to the downloaded image
pub async fn download_image_data(
    data: SetuData,
    output_dir: &Path,
    size: ImageSize,
    max_retry: usize,
    client: &Client,
    to_skip: Option<impl Fn(&SetuData) -> bool>,
) -> Result<Downloaded> {
    let pid = data.pid;
    let image_url = get_url_by_size(&data, size)?;

    fs::create_dir_all(output_dir).await?;

    let target_path = get_target_path(output_dir, image_url)?;
    let skip_setu = to_skip.is_some_and(|call| call(&data));
    if target_path.exists() || skip_setu {
        if skip_setu {
            debug!("skip {pid}: filtered");
        } else {
            debug!("skip {pid}: existing {}", target_path.to_string_lossy());
        }
        return Ok(Downloaded {
            data,
            path: target_path,
            ..Default::default()
        });
    }
    info!("url: {image_url}",);

    let url = Url::from_str(image_url)?;
    let image = download_retry(&url, max_retry, 500, client, pid as _).await?;

    Ok(Downloaded {
        data,
        path: target_path,
        raw_image: Some(image),
    })
}

/// download each image in data from `setu`
pub async fn download_images(
    setu: Setu,
    output_dir: impl AsRef<Path>,
    size: lolicon_api::ImageSize,
    max_retry: usize,
    client: &Client,
    to_skip: Option<&'static (impl Fn(&SetuData) -> bool + Send + Sync)>,
) -> Vec<Result<PathBuf>> {
    let mut results = Vec::new();

    let mut tasks = JoinSet::new();
    let mut write_tasks = JoinSet::new();

    for data in setu.data {
        let client = client.clone();
        let output_dir = output_dir.as_ref().to_path_buf();
        tasks.spawn(async move {
            download_image_data(data, output_dir.as_ref(), size, max_retry, &client, to_skip).await
        });
    }

    while let Some(result) = tasks.join_next().await {
        let Ok(result) = result else {
            continue;
        };

        match result {
            Ok(d) => {
                results.push(Ok(d.path.clone()));
                write_tasks.spawn_blocking(move || {
                    let target_path = d.path;

                    if let Some(bytes) = d.raw_image {
                        let _ = std::fs::write(&target_path, bytes);
                        info!("saved {}", target_path.to_string_lossy());
                    }
                });
            }
            Err(e) => {
                error!("download failed: {e}");
                results.push(Err(e));
            }
        }
    }
    let _ = write_tasks.join_all().await;
    results
}

/// download an image to bytes, return error on 404 page
///
/// if `max_retry` was set to zero, it will never download.
pub async fn download_retry(
    url: &Url,
    max_retry: usize,
    initial_wait_ms: u64,
    client: &Client,
    pid: u64,
) -> Result<Bytes> {
    let mut image = Err(LoliconError::RetryExceed);

    let mut wait_time_ms = initial_wait_ms;
    for _ in 0..max_retry {
        let result = client
            .get(url.as_str())
            .timeout(Duration::from_secs(10))
            .send()
            .await;
        if let Ok(resp) = result {
            if let Ok(bytes) = resp.bytes().await {
                if bytes.is_ascii() {
                    Err(LoliconError::NotFound(pid))?
                }
                image = Ok(bytes);
                break;
            }
        }

        warn!("download {pid} failed, will retry after {wait_time_ms}ms...");
        tokio::time::sleep(Duration::from_millis(wait_time_ms)).await;
        wait_time_ms *= 2;
    }

    Ok(image?)
}
