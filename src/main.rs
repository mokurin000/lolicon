use lolicon_api::Request;
use lolicon_api::R18;
use serde_json::Value;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let req = Request::default()
        // .tag(&["泳装".into()])?
        .r18(R18::Mixin)
        .proxy("i.pixiv.re");

    let url = String::from(req);

    let raw_result = get(url)?.text()?;

    let result: Value = serde_json::from_str(&raw_result)?;

    let original = result.pointer("/data/0/urls/original");
    let pid = result.pointer("/data/0/pid");

    if let Some(Value::Number(pid)) = pid {
        eprintln!("pid: {}", pid);
    }

    if let Some(Value::String(ref image_url)) = original {
        println!("{image_url}");
    }

    Ok(())
}
