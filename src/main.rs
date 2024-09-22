use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use reqwest::blocking::get;
use serde_json::Value;

use lolicon_api::Category;
use lolicon_api::Request;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let req = Request::default()
        .num(1)?
        .exclude_ai(true)
        .category(Category::R18)
        .aspect_ratio("lt1")?
        .proxy("i.pixiv.cat");

    let url = String::from(req);
    println!("quering api: {url}");

    let raw_result = get(url)?.text()?;

    let result: Value = serde_json::from_str(&raw_result)?;

    let original = result.pointer("/data/0/urls/original");
    let pid = result.pointer("/data/0/pid");

    if let Some(Value::Number(pid)) = pid {
        eprintln!("pid: {}", pid);
    }

    if let Some(Value::String(ref image_url)) = original {
        fs::create_dir_all("images")?;

        let url = url::Url::from_str(&image_url)?;
        let basename = url.path_segments().unwrap().last().unwrap();
        let target_path = Path::new("images").join(basename);
        let mut metadata_path = target_path.clone();
        metadata_path.set_extension(".json");

        if target_path.exists() {
            println!("skipping existing image.");
            return Ok(());
        }
        println!("downloading {image_url}...",);

        let mut image = Err("download failed. exceeding retry limit");
        for _ in 0..10 {
            let result = get(url.as_str()).and_then(|result| result.bytes());
            if let Ok(bytes) = result {
                if bytes.is_ascii() {
                    Err("Image not found! may removed by its author")?
                }
                image = Ok(bytes);
            }
            sleep(Duration::from_millis(500));
        }
        let image = image?;

        println!("writing image...");
        fs::write(&target_path, &image)?;
        println!("writing metadata...");
        fs::write(&metadata_path, result.to_string())?;
    }

    Ok(())
}
