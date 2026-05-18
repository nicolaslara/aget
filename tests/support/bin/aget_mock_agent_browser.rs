use std::env;
use std::fs;
use std::path::Path;

use serde_json::Value;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    if args.len() == 6 && args[0] == "--profile" && args[2] == "--session" && args[4] == "open" {
        if args[5].starts_with("https://") || args[5] == "about:blank" {
            return Ok(());
        }
        return Err(format!("unexpected open target: {}", args[5]));
    }

    if args.len() == 3 && args[0] == "--session" && args[2] == "close" {
        return Ok(());
    }

    if args.len() == 5 && args[0] == "--session" && args[2] == "state" && args[3] == "save" {
        let agent_session = &args[1];
        let state_path = Path::new(&args[4]);
        let domain = if let Some(session_name) = agent_session.strip_prefix("aget-login-") {
            let pending_path = state_path
                .parent()
                .ok_or_else(|| "state path has no parent".to_string())?
                .join(format!("login-{session_name}.json"));
            let pending: Value = serde_json::from_str(
                &fs::read_to_string(&pending_path)
                    .map_err(|error| format!("read pending login metadata: {error}"))?,
            )
            .map_err(|error| format!("parse pending login metadata: {error}"))?;
            pending["allowed_domains"]
                .as_array()
                .and_then(|domains| domains.first())
                .and_then(Value::as_str)
                .ok_or_else(|| "pending login metadata has no allowed domain".to_string())?
                .to_string()
        } else if agent_session.starts_with("aget-import-") {
            "127.0.0.1".to_string()
        } else {
            return Err(format!("unexpected session name: {agent_session}"));
        };

        let state = serde_json::json!({
            "cookies": [{
                "name": "app_session",
                "value": "valid-app",
                "domain": domain,
                "path": "/",
                "httpOnly": true,
                "secure": false,
                "sameSite": "Lax"
            }],
            "origins": []
        });
        fs::write(
            state_path,
            serde_json::to_vec(&state).map_err(|error| format!("serialize state: {error}"))?,
        )
        .map_err(|error| format!("write state: {error}"))?;
        return Ok(());
    }

    Err(format!("unexpected mock agent-browser args: {args:?}"))
}
