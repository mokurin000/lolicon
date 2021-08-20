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
    if let Value::Object(map) = result {
        if let Value::Array(ref setu) = map["data"] {
            let map = &setu[0];
            if let Value::Object(ref map) = map["urls"] {
                if let Value::String(ref image_url) = map["original"] {
                    println!("{}", image_url);
                }
            }
        }
    }

    Ok(())
}
