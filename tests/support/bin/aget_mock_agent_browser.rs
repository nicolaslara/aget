use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

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
    append_log(&config, &format_args(&args))?;

    match config["behavior"].as_str().unwrap_or("generic") {
        "profile_lock" => {
            eprintln!("Please quit Chrome before importing this profile");
            Ok(2)
        }
        behavior => run_generic(behavior, args, &config),
    }
}

fn format_args(args: &[String]) -> String {
    let quoted = args
        .iter()
        .map(|arg| serde_json::to_string(arg).unwrap())
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{quoted}]")
}

fn run_generic(behavior: &str, args: Vec<String>, config: &Value) -> Result<i32, String> {
    if args.len() == 6 && args[0] == "--profile" && args[2] == "--session" && args[4] == "open" {
        if args[5].starts_with("https://")
            || args[5].starts_with("http://")
            || args[5] == "about:blank"
        {
            return Ok(0);
        }
        return Err(format!("unexpected open target: {}", args[5]));
    }

    if args.len() == 7
        && args[0] == "--profile"
        && args[2] == "--session"
        && args[4] == "state"
        && args[5] == "load"
    {
        return Ok(0);
    }

    if args.len() == 7
        && args[0] == "--profile"
        && args[2] == "--session"
        && args[4] == "get"
        && args[5] == "html"
        && args[6] == "body"
    {
        println!(
            "{}",
            config["html"]
                .as_str()
                .unwrap_or("<main><h1>Fallback Title</h1><p>Useful &amp; local content</p></main>")
        );
        return Ok(0);
    }

    if (args.len() == 3 && args[0] == "--session" && args[2] == "close")
        || (args.len() == 5
            && args[0] == "--profile"
            && args[2] == "--session"
            && args[4] == "close")
    {
        if behavior == "close_failure" {
            eprintln!("{}", config["close_stderr"].as_str().unwrap_or("close failed"));
            return Ok(config["close_exit_code"].as_i64().unwrap_or(1) as i32);
        }
        return Ok(0);
    }

    if args.len() == 5 && args[0] == "--session" && args[2] == "state" && args[3] == "save" {
        let state_path = Path::new(&args[4]);
        append_log(&config, &format!("STATE_PATH={}", state_path.display()))?;
        if behavior == "malformed_state" {
            fs::write(state_path, "{bad json").map_err(|error| format!("write state: {error}"))?;
            return Ok(0);
        }
        let state = config
            .get("state")
            .cloned()
            .unwrap_or_else(|| default_state_for_session(&args[1], state_path));
        fs::write(
            state_path,
            serde_json::to_vec(&state).map_err(|error| format!("serialize state: {error}"))?,
        )
        .map_err(|error| format!("write state: {error}"))?;
        return Ok(0);
    }

    Err(format!("unexpected mock agent-browser args: {args:?}"))
}

fn read_config() -> Result<Value, String> {
    let config_path = env::current_exe()
        .map_err(|error| format!("locate mock agent-browser executable: {error}"))?
        .with_extension("json");
    if !config_path.exists() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(
        &fs::read_to_string(&config_path)
            .map_err(|error| format!("read mock agent-browser config: {error}"))?,
    )
    .map_err(|error| format!("parse mock agent-browser config: {error}"))
}

fn append_log(config: &Value, line: &str) -> Result<(), String> {
    let path = config["log_path"]
        .as_str()
        .map(ToOwned::to_owned)
        .or_else(|| env::var("AGET_FAKE_AGENT_BROWSER_LOG").ok())
        .or_else(|| env::var("AGENT_BROWSER_LOG").ok());
    let Some(path) = path else {
        return Ok(());
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("open log {path}: {error}"))?;
    writeln!(file, "{line}").map_err(|error| format!("write log {path}: {error}"))
}

fn default_state_for_session(agent_session: &str, state_path: &Path) -> Value {
    let domain = if let Some(session_name) = agent_session.strip_prefix("aget-login-") {
        state_path
            .parent()
            .and_then(|parent| {
                let pending_path = parent.join(format!("login-{session_name}.json"));
                fs::read_to_string(pending_path).ok()
            })
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .and_then(|pending| {
                pending["allowed_domains"]
                    .as_array()
                    .and_then(|domains| domains.first())
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| "127.0.0.1".to_string())
    } else {
        "127.0.0.1".to_string()
    };

    serde_json::json!({
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
    })
}
