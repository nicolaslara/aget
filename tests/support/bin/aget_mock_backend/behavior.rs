use std::fs;
use std::process::Command;
use std::time::Duration;

use serde_json::Value;

use crate::args::Args;
use crate::config::{expand_state_placeholders, read_state};
use crate::http::{
    element_body, fetch_following_redirects, parse_http_url, remove_elements, session_headers,
    textify,
};
use crate::output::print_backend_result;

pub(crate) fn run_behavior(args: &Args, config: &Value) -> Result<i32, String> {
    match config["behavior"].as_str().unwrap_or("http_fetch") {
        "success" => success(args, config),
        "http_fetch" => http_fetch(args, config),
        "structured_failure" => structured_failure(args, config),
        "exit" => {
            if let Some(stderr) = config["stderr"].as_str() {
                eprintln!("{stderr}");
            }
            Ok(config["exit_code"].as_i64().unwrap_or(2) as i32)
        }
        "malformed" => {
            println!("{}", config["stdout"].as_str().unwrap_or("not json"));
            Ok(config["exit_code"].as_i64().unwrap_or(0) as i32)
        }
        "sleep" => {
            std::thread::sleep(Duration::from_secs(
                config["seconds"].as_u64().unwrap_or(3),
            ));
            Ok(0)
        }
        "noisy_success" => {
            eprint!(
                "{}",
                "noise".repeat(config["noise_repetitions"].as_u64().unwrap_or(20000) as usize)
            );
            success(args, config)
        }
        "stdout_logs_success" => {
            for line in config["stdout_lines"].as_array().into_iter().flatten() {
                if let Some(line) = line.as_str() {
                    println!("{line}");
                }
            }
            success(args, config)
        }
        "spawn_descendant_and_sleep" => {
            let marker = config["marker"]
                .as_str()
                .ok_or_else(|| "spawn_descendant_and_sleep requires marker".to_string())?;
            Command::new(std::env::current_exe().map_err(|error| error.to_string())?)
                .arg("--descendant-marker")
                .arg(marker)
                .spawn()
                .map_err(|error| format!("spawn descendant: {error}"))?;
            std::thread::sleep(Duration::from_secs(
                config["seconds"].as_u64().unwrap_or(5),
            ));
            Ok(0)
        }
        other => Err(format!("unknown mock backend behavior: {other}")),
    }
}

fn success(args: &Args, config: &Value) -> Result<i32, String> {
    let content = config["content"].as_str().unwrap_or("# Fake");
    fs::write(&args.output, content).map_err(|error| format!("write output: {error}"))?;
    if let Some(metadata) = config.get("backend_metadata") {
        fs::write(
            &args.metadata,
            serde_json::to_vec(metadata).map_err(|error| format!("serialize metadata: {error}"))?,
        )
        .map_err(|error| format!("write metadata: {error}"))?;
    }
    let final_url = config["final_url"]
        .as_str()
        .map(ToOwned::to_owned)
        .or_else(|| {
            config["final_url_suffix"]
                .as_str()
                .map(|suffix| format!("{}{suffix}", args.url))
        })
        .unwrap_or_else(|| args.url.clone());
    let warnings = config["warnings"].as_array().cloned().unwrap_or_default();
    print_backend_result(true, Some(final_url), Some(content.to_string()), warnings, None);
    Ok(config["exit_code"].as_i64().unwrap_or(0) as i32)
}

fn structured_failure(args: &Args, config: &Value) -> Result<i32, String> {
    let state = read_state(&args.state)?;
    let error = expand_state_placeholders(
        config["error"]
            .as_str()
            .unwrap_or("structured backend failure"),
        &state,
    );
    if let Some(stderr) = config["stderr"].as_str() {
        eprintln!("{}", expand_state_placeholders(stderr, &state));
    }
    print_backend_result(false, None, None, Vec::new(), Some(error));
    Ok(config["exit_code"].as_i64().unwrap_or(0) as i32)
}

fn http_fetch(args: &Args, config: &Value) -> Result<i32, String> {
    let parsed = parse_http_url(&args.url)?;
    let state = read_state(&args.state)?;
    let mut headers = session_headers(&state, &parsed);
    if let Some(cookie_header) = config["cookie_header"].as_str() {
        headers.insert("Cookie".to_string(), cookie_header.to_string());
    }
    if !headers.contains_key("Cookie") {
        if let Some(expected) = config["expect_state_cookies"].as_array() {
            let cookies = expected
                .iter()
                .filter_map(|pair| {
                    let pair = pair.as_array()?;
                    Some(format!("{}={}", pair.first()?.as_str()?, pair.get(1)?.as_str()?))
                })
                .collect::<Vec<_>>();
            if !cookies.is_empty() {
                headers.insert("Cookie".to_string(), cookies.join("; "));
            }
        }
    }
    let mut response = fetch_following_redirects(&args.url, &headers)?;

    if parsed.path == "/storage-protected" {
        if let Some(token) = headers.remove("X-Local-Token") {
            response = fetch_following_redirects(
                &format!("{}/storage-api", parsed.origin),
                &std::collections::BTreeMap::from([("X-Local-Token".to_string(), token)]),
            )?;
        }
    }

    let mut body = response.body;
    if args.wait_for.as_deref() == Some("#ready") && parsed.path == "/delayed" {
        body.push_str(r#"<div id="ready">Delayed Ready</div>"#);
    }
    if args.exclude_selector.as_deref() == Some("nav") {
        body = remove_elements(&body, "nav");
    }
    if args.selector.as_deref() == Some("main") {
        if let Some(main) = element_body(&body, "main") {
            body = main;
        }
    }

    let mut content = match args.format.as_str() {
        "html" => body,
        "json" => serde_json::json!({
            "url": response.final_url,
            "content": textify(&body),
        })
        .to_string(),
        _ => textify(&body),
    };
    if let Some(override_content) = config["content"].as_str() {
        content = override_content.to_string();
    }
    if let Some(prefix) = config["content_prefix"].as_str() {
        content = format!("{prefix}{content}");
    }

    fs::write(&args.output, &content).map_err(|error| format!("write output: {error}"))?;
    fs::write(
        &args.metadata,
        serde_json::to_vec(&serde_json::json!({
            "mock_site": true,
            "url": args.url,
            "status": response.status,
        }))
        .map_err(|error| format!("serialize metadata: {error}"))?,
    )
    .map_err(|error| format!("write metadata: {error}"))?;

    let warnings = if parsed.path == "/warning" {
        vec![serde_json::json!("mock warning")]
    } else {
        config["warnings"].as_array().cloned().unwrap_or_default()
    };
    let final_url = config["final_url"]
        .as_str()
        .map(ToOwned::to_owned)
        .or_else(|| {
            config["final_url_suffix"]
                .as_str()
                .map(|suffix| format!("{}{suffix}", args.url))
        })
        .unwrap_or(response.final_url);
    print_backend_result(true, Some(final_url), Some(content), warnings, None);
    Ok(0)
}
