use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use reqwest::get;
use serde_json::Value;
use tokio::fs;
use url::Url;

use crate::Result;

pub async fn download_image(result: Value, output_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let original = result.pointer("/data/0/urls/original");
    let pid = result.pointer("/data/0/pid");

    if let Some(Value::Number(pid)) = pid {
        eprintln!("pid: {}", pid);
    }

    let Some(Value::String(ref image_url)) = original else {
        Err("failed to parse image_url!")?
    };

    fs::create_dir_all(output_dir.as_ref()).await?;

    let url = url::Url::from_str(&image_url)?;
    let basename = url.path_segments().unwrap().last().unwrap();
    let target_path = Path::new(output_dir.as_ref()).join(basename);
    let mut metadata_path = target_path.clone();
    metadata_path.set_extension(".json");

    println!("writing metadata...");
    fs::write(&metadata_path, result.to_string()).await?;

    if target_path.exists() {
        println!("skipping existing image.");
        return Ok(target_path);
    }
    println!("downloading {image_url}...",);

    let image = download_retry(&url).await?;

    println!("writing image...");
    fs::write(&target_path, &image).await?;

    Ok(target_path)
}

pub async fn download_retry(url: &Url) -> Result<bytes::Bytes> {
    let mut image = Err("download failed. exceeding retry limit");

    for _ in 0..5 {
        let result = get(url.as_str()).await;
        if let Ok(resp) = result {
            let bytes = resp.bytes().await?;
            if bytes.is_ascii() {
                Err("Image not found! may removed by its author")?
            }
            image = Ok(bytes);
            break;
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(image?)
}
