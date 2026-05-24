use std::env;
use std::fs;

use serde_json::Value;

fn main() {
    match run(env::args().skip(1).collect()) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

fn run(args: Vec<String>) -> Result<i32, String> {
    let config = read_config()?;
    if args.len() != 8
        || args[0] != "--json"
        || args[1] != "browser"
        || args[2] != "--surface"
        || args[4] != "cookies"
        || args[5] != "get"
        || args[6] != "--domain"
    {
        return Err(format!("unexpected cmux args: {args:?}"));
    }
    if let Some(expected_surface) = config["surface"].as_str() {
        if args[3] != expected_surface {
            return Err(format!(
                "expected surface {expected_surface:?}, got {:?}",
                args[3]
            ));
        }
    }

    let domain = &args[7];
    let cookies = config["cookies"].as_array().cloned().unwrap_or_else(|| {
        vec![
            serde_json::json!({
                "name": "sid",
                "value": "allowed-secret",
                "domain": domain,
                "path": "/",
                "secure": true,
                "session_only": false,
                "expires": 1910000000_i64
            }),
            serde_json::json!({
                "name": "wide",
                "value": "suffix-secret",
                "domain": format!(".{domain}"),
                "path": "/",
                "secure": false,
                "session_only": true,
                "expires": null
            }),
            serde_json::json!({
                "name": "evil",
                "value": "blocked-secret",
                "domain": format!("{domain}.evil"),
                "path": "/",
                "secure": false,
                "session_only": true,
                "expires": null
            }),
        ]
    });
    println!("{}", serde_json::json!({ "cookies": cookies }));
    Ok(0)
}

fn read_config() -> Result<Value, String> {
    let config_path = env::current_exe()
        .map_err(|error| format!("locate mock cmux executable: {error}"))?
        .with_extension("json");
    if !config_path.exists() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(
        &fs::read_to_string(&config_path)
            .map_err(|error| format!("read mock cmux config: {error}"))?,
    )
    .map_err(|error| format!("parse mock cmux config: {error}"))
}
