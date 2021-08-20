use lolicon_api::Request;
use lolicon_api::R18;
use reqwest::blocking::get;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let req = Request::default()
        .r18(R18::R18)
        .num(1)?
        .size(vec!["original".into()])?;
    let url = String::from(req);
    let raw_result = get(url)?.text()?;
    let result: Value = serde_json::from_str(&raw_result)?;
    let original = result.pointer("/data/0/urls/original");
    if let Some(Value::String(ref image_url)) = original {
        println!("{}", image_url);
    }

    Ok(())
}
