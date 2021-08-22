use lolicon_api::Request;
use lolicon_api::R18;
use reqwest::blocking::get;
use serde_json::Value;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let req = Request::default()
        .r18(R18::Mixin)
        // .tag(&["泳装".into()])?
        .uid(&[16731])?;

    let url = String::from(req);

    let raw_result = get(url)?.text()?;

    let result: Value = serde_json::from_str(&raw_result)?;

    let original = result.pointer("/data/0/urls/original");
    let pid = result.pointer("/data/0/pid");
    let r18 = result.pointer("/data/0/r18");

    if let Some(Value::Number(pid)) = pid {
        eprintln!("pid: {}", pid);
    }
    if let Some(Value::Bool(r18)) = r18 {
        eprintln!("r18: {}", r18);
    }

    if let Some(Value::String(ref image_url)) = original {
        let image_req = get(image_url)?;
        let file_name = image_req.url()
            .path_segments()
            .and_then(|path| path.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("temp.bin").to_string();
        let mut file = std::fs::File::create(&file_name)?;
        file.write_all(image_req.bytes()?.as_ref())?;

        eprintln!("saved as {}.", file_name);
    }

    Ok(())
}
