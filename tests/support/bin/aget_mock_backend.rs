use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use serde_json::Value;

#[derive(Default)]
struct Args {
    url: String,
    state: PathBuf,
    output: PathBuf,
    metadata: PathBuf,
    format: String,
    selector: Option<String>,
    exclude_selector: Option<String>,
    wait_for: Option<String>,
    extractor_options: Vec<String>,
}

#[derive(Clone)]
struct ParsedUrl {
    host: String,
    port: u16,
    path: String,
    origin: String,
}

struct HttpResponse {
    status: u16,
    final_url: String,
    headers: BTreeMap<String, String>,
    body: String,
}

fn main() {
    if let Some(marker) = descendant_marker_arg() {
        std::thread::sleep(Duration::from_secs(2));
        let _ = fs::write(marker, "survived");
        return;
    }

    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32, String> {
    let args = parse_args(env::args().skip(1).collect())?;
    let config = read_config()?;

    assert_expected_args(&args, &config)?;
    assert_expected_environment(&config)?;
    assert_expected_state(&args, &config)?;

    match config["behavior"].as_str().unwrap_or("http_fetch") {
        "success" => success(&args, &config),
        "http_fetch" => http_fetch(&args, &config),
        "structured_failure" => structured_failure(&args, &config),
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
            success(&args, &config)
        }
        "stdout_logs_success" => {
            for line in config["stdout_lines"].as_array().into_iter().flatten() {
                if let Some(line) = line.as_str() {
                    println!("{line}");
                }
            }
            success(&args, &config)
        }
        "spawn_descendant_and_sleep" => {
            let marker = config["marker"]
                .as_str()
                .ok_or_else(|| "spawn_descendant_and_sleep requires marker".to_string())?;
            Command::new(env::current_exe().map_err(|error| error.to_string())?)
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

fn descendant_marker_arg() -> Option<PathBuf> {
    let mut args = env::args().skip(1);
    if args.next().as_deref() == Some("--descendant-marker") {
        args.next().map(PathBuf::from)
    } else {
        None
    }
}

fn read_config() -> Result<Value, String> {
    let config_path = env::current_exe()
        .map_err(|error| format!("locate mock backend executable: {error}"))?
        .with_extension("json");
    if !config_path.exists() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(
        &fs::read_to_string(&config_path)
            .map_err(|error| format!("read mock backend config: {error}"))?,
    )
    .map_err(|error| format!("parse mock backend config: {error}"))
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
                &BTreeMap::from([("X-Local-Token".to_string(), token)]),
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

fn print_backend_result(
    ok: bool,
    final_url: Option<String>,
    content: Option<String>,
    warnings: Vec<Value>,
    error: Option<String>,
) {
    println!(
        "{}",
        serde_json::json!({
            "ok": ok,
            "final_url": final_url,
            "content": content,
            "warnings": warnings,
            "error": error,
        })
    );
}

fn assert_expected_args(args: &Args, config: &Value) -> Result<(), String> {
    for (field, actual) in [
        ("format", Some(args.format.as_str())),
        ("selector", args.selector.as_deref()),
        ("exclude_selector", args.exclude_selector.as_deref()),
        ("wait_for", args.wait_for.as_deref()),
    ] {
        if let Some(expected) = config[format!("expect_{field}")].as_str() {
            if Some(expected) != actual {
                return Err(format!("expected {field}={expected:?}, got {actual:?}"));
            }
        }
    }
    if let Some(expected) = config["expect_extractor_options"].as_array() {
        let expected = expected
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| "expect_extractor_options entries must be strings".to_string())
                    .map(ToOwned::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if args.extractor_options != expected {
            return Err(format!(
                "expected extractor_options={expected:?}, got {:?}",
                args.extractor_options
            ));
        }
    }
    if config["validate_crawl4ai_options"].as_bool().unwrap_or(false) {
        validate_crawl4ai_options(args)?;
    }
    Ok(())
}

fn validate_crawl4ai_options(args: &Args) -> Result<(), String> {
    const ALLOWED: &[&str] = &[
        "target_elements",
        "excluded_tags",
        "only_text",
        "word_count_threshold",
        "wait_until",
        "page_timeout",
        "wait_for_timeout",
        "delay_before_return_html",
        "wait_for_images",
    ];
    for option in &args.extractor_options {
        let Some((key, _value)) = option.split_once('=') else {
            return structured_validation_error(
                args,
                format!("extractor option must use key=value form: {option}"),
            );
        };
        let key = key.strip_prefix("crawl4ai.").ok_or_else(|| {
            format!("extractor option '{key}' must use the crawl4ai.<key> namespace")
        })?;
        if !ALLOWED.contains(&key) {
            let allowed = ALLOWED.join(", ");
            return structured_validation_error(
                args,
                format!("unsupported extractor option '{key}'; supported keys: {allowed}"),
            );
        }
    }
    if let Some(wait_for) = &args.wait_for {
        let normalized = wait_for.trim().to_ascii_lowercase();
        if normalized.starts_with("js:")
            || ["=>", "function(", "return ", ";"]
                .iter()
                .any(|marker| normalized.contains(marker))
        {
            return structured_validation_error(
                args,
                "--wait-for-selector only supports CSS selectors in v1; JavaScript wait conditions are not allowed".to_string(),
            );
        }
    }
    Ok(())
}

fn structured_validation_error(args: &Args, message: String) -> Result<(), String> {
    print_backend_result(false, None, None, Vec::new(), Some(message.clone()));
    fs::write(
        &args.metadata,
        serde_json::to_vec_pretty(&serde_json::json!({
            "ok": false,
            "final_url": null,
            "content": null,
            "warnings": [],
            "error": message,
        }))
        .map_err(|error| format!("serialize validation metadata: {error}"))?,
    )
    .map_err(|error| format!("write validation metadata: {error}"))?;
    std::process::exit(1);
}

fn assert_expected_environment(config: &Value) -> Result<(), String> {
    for key in config["expect_env_absent"].as_array().into_iter().flatten() {
        let key = key
            .as_str()
            .ok_or_else(|| "expect_env_absent entries must be strings".to_string())?;
        if env::var_os(key).is_some() {
            return Err(format!("expected environment variable {key} to be absent"));
        }
    }
    Ok(())
}

fn assert_expected_state(args: &Args, config: &Value) -> Result<(), String> {
    if config.get("expect_state").is_none() && config.get("expect_state_cookies").is_none() {
        return Ok(());
    }
    let state = read_state(&args.state)?;
    if let Some(expected) = config.get("expect_state") {
        if &state != expected {
            return Err(format!("expected state {expected}, got {state}"));
        }
    }
    if let Some(expected) = config["expect_state_cookies"].as_array() {
        let mut actual = state["cookies"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|cookie| {
                serde_json::json!([
                    cookie["name"].as_str().unwrap_or_default(),
                    cookie["value"].as_str().unwrap_or_default()
                ])
            })
            .collect::<Vec<_>>();
        actual.sort_by_key(|value| value.to_string());
        let mut expected = expected.clone();
        expected.sort_by_key(|value| value.to_string());
        if actual != expected {
            return Err(format!(
                "expected state cookies {expected:?}, got {actual:?}"
            ));
        }
    }
    Ok(())
}

fn read_state(path: &PathBuf) -> Result<Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path).map_err(|error| format!("read state: {error}"))?,
    )
    .map_err(|error| format!("parse state: {error}"))
}

fn expand_state_placeholders(template: &str, state: &Value) -> String {
    template
        .replace(
            "{first_cookie_value}",
            first_cookie_value(state).unwrap_or_default().as_str(),
        )
        .replace(
            "{first_storage_value}",
            first_storage_value(state).unwrap_or_default().as_str(),
        )
}

fn first_cookie_value(state: &Value) -> Option<String> {
    state["cookies"].as_array()?.first()?["value"]
        .as_str()
        .map(ToOwned::to_owned)
}

fn first_storage_value(state: &Value) -> Option<String> {
    state["origins"].as_array()?.first()?["localStorage"]
        .as_array()?
        .first()?["value"]
        .as_str()
        .map(ToOwned::to_owned)
}

fn parse_args(raw: Vec<String>) -> Result<Args, String> {
    let mut args = Args {
        format: "markdown".to_string(),
        ..Args::default()
    };
    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--url" => args.url = next_value(&mut iter, "--url")?,
            "--state" => args.state = PathBuf::from(next_value(&mut iter, "--state")?),
            "--output" => args.output = PathBuf::from(next_value(&mut iter, "--output")?),
            "--metadata" => args.metadata = PathBuf::from(next_value(&mut iter, "--metadata")?),
            "--format" => args.format = next_value(&mut iter, "--format")?,
            "--selector" => args.selector = Some(next_value(&mut iter, "--selector")?),
            "--exclude-selector" => {
                args.exclude_selector = Some(next_value(&mut iter, "--exclude-selector")?)
            }
            "--wait-for" => args.wait_for = Some(next_value(&mut iter, "--wait-for")?),
            "--extractor-option" => args
                .extractor_options
                .push(next_value(&mut iter, "--extractor-option")?),
            _ => return Err(format!("unsupported backend arg: {arg}")),
        }
    }

    if args.url.is_empty()
        || args.state.as_os_str().is_empty()
        || args.output.as_os_str().is_empty()
        || args.metadata.as_os_str().is_empty()
    {
        return Err("missing required backend args".to_string());
    }
    for option in &args.extractor_options {
        if !option.starts_with("crawl4ai.") || !option.contains('=') {
            return Err(format!("unsupported extractor option: {option}"));
        }
    }
    Ok(args)
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn session_headers(state: &Value, parsed: &ParsedUrl) -> BTreeMap<String, String> {
    let cookies = state["cookies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|cookie| {
            let domain = cookie["domain"]
                .as_str()?
                .trim_start_matches('.')
                .to_lowercase();
            if parsed.host == domain || parsed.host.ends_with(&format!(".{domain}")) {
                Some(format!(
                    "{}={}",
                    cookie["name"].as_str()?,
                    cookie["value"].as_str()?
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut headers = BTreeMap::new();
    if !cookies.is_empty() {
        headers.insert("Cookie".to_string(), cookies.join("; "));
    }
    for origin in state["origins"].as_array().into_iter().flatten() {
        if origin["origin"].as_str() == Some(parsed.origin.as_str()) {
            for entry in origin["localStorage"].as_array().into_iter().flatten() {
                if entry["name"].as_str() == Some("local_token") {
                    if let Some(value) = entry["value"].as_str() {
                        headers.insert("X-Local-Token".to_string(), value.to_string());
                    }
                }
            }
        }
    }
    headers
}

fn fetch_following_redirects(
    url: &str,
    headers: &BTreeMap<String, String>,
) -> Result<HttpResponse, String> {
    let mut current = url.to_string();
    for _ in 0..5 {
        let response = fetch_once(&current, headers)?;
        if (300..400).contains(&response.status) {
            if let Some(location) = response.headers.get("location") {
                current = absolutize_url(&current, location)?;
                continue;
            }
        }
        return Ok(response);
    }
    Err("too many redirects".to_string())
}

fn fetch_once(url: &str, headers: &BTreeMap<String, String>) -> Result<HttpResponse, String> {
    let parsed = parse_http_url(url)?;
    let mut stream = TcpStream::connect((parsed.host.as_str(), parsed.port))
        .map_err(|error| format!("connect to {}:{}: {error}", parsed.host, parsed.port))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("set read timeout: {error}"))?;

    write!(
        stream,
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n",
        parsed.path, parsed.host, parsed.port
    )
    .map_err(|error| format!("write request: {error}"))?;
    for (name, value) in headers {
        write!(stream, "{name}: {value}\r\n").map_err(|error| format!("write header: {error}"))?;
    }
    write!(stream, "\r\n").map_err(|error| format!("finish request: {error}"))?;

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|error| format!("read response: {error}"))?;
    parse_response(url, &raw)
}

fn parse_response(url: &str, raw: &[u8]) -> Result<HttpResponse, String> {
    let raw = String::from_utf8_lossy(raw);
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| "malformed HTTP response".to_string())?;
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .ok_or_else(|| "missing HTTP status".to_string())?;
    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let final_url = if (300..400).contains(&status) {
        headers
            .get("location")
            .map(|location| absolutize_url(url, location))
            .transpose()?
            .unwrap_or_else(|| url.to_string())
    } else {
        url.to_string()
    };
    Ok(HttpResponse {
        status,
        final_url,
        headers,
        body: body.to_string(),
    })
}

fn parse_http_url(url: &str) -> Result<ParsedUrl, String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("mock backend only supports http URLs, got {url}"))?;
    let (authority, path) = match rest.split_once('/') {
        Some((authority, path)) => (authority, format!("/{path}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|error| format!("invalid port in {url}: {error}"))?,
        ),
        None => (authority.to_string(), 80),
    };
    Ok(ParsedUrl {
        origin: format!("http://{authority}"),
        host,
        port,
        path,
    })
}

fn absolutize_url(base: &str, location: &str) -> Result<String, String> {
    if location.starts_with("http://") {
        return Ok(location.to_string());
    }
    let parsed = parse_http_url(base)?;
    if location.starts_with('/') {
        Ok(format!("{}{}", parsed.origin, location))
    } else {
        let parent = parsed
            .path
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or("");
        Ok(format!("{}/{location}", parent.trim_end_matches('/')))
    }
}

fn element_body(html: &str, tag: &str) -> Option<String> {
    let start_marker = format!("<{tag}");
    let start = html.find(&start_marker)?;
    let body_start = html[start..].find('>')? + start + 1;
    let end = html[body_start..].find(&format!("</{tag}>"))? + body_start;
    Some(html[body_start..end].to_string())
}

fn remove_elements(html: &str, tag: &str) -> String {
    let mut output = html.to_string();
    while let Some(start) = output.find(&format!("<{tag}")) {
        let Some(after_open) = output[start..].find('>').map(|index| start + index + 1) else {
            break;
        };
        let Some(end) = output[after_open..]
            .find(&format!("</{tag}>"))
            .map(|index| after_open + index + tag.len() + 3)
        else {
            break;
        };
        output.replace_range(start..end, "");
    }
    output
}

fn textify(html: &str) -> String {
    let without_scripts = remove_elements(&remove_elements(html, "script"), "style");
    let mut text = String::new();
    let mut in_tag = false;
    for character in without_scripts.chars() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
