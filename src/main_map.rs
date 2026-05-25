use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aget::{Aget, ErrorCode, ErrorResponse, MapCommand, MapOutput, OutputFormat, SessionStore};
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::Value;
use url::Url;

const MAX_MAX_LINKS: usize = 5000;

pub(super) fn run_map(
    command: MapCommand,
    json: bool,
    quiet: bool,
) -> Result<ExitCode, ErrorResponse> {
    let started = Instant::now();
    let store = SessionStore::from_env().map_err(super::io_error)?;
    let input = resolve_input(&command, &store)?;
    if command.max_links == 0 || command.max_links > MAX_MAX_LINKS {
        return Err(usage_error(format!(
            "map --max-links must be between 1 and {MAX_MAX_LINKS}"
        )));
    }

    let output_dir = store.home().join("runs").join(map_run_id());
    fs::create_dir_all(&output_dir).map_err(super::io_error)?;

    let source = match input {
        MapInput::Url(url) => fetch_source(&command, &output_dir, url)?,
        MapInput::Artifact(run_id) => artifact_source(&store, &run_id)?,
    };
    let (links, discovered) = extract_links(&source, &command)?;
    let summary = MapSummary {
        discovered,
        emitted: links.len(),
        truncated: links.len() == command.max_links,
    };
    let result = MapResult {
        summary,
        source: MapSource {
            url: source.url,
            final_url: source.final_url,
            artifact: source.artifact,
        },
        artifacts: MapArtifacts {
            json: output_dir.join("links.json"),
            markdown: output_dir.join("links.md"),
        },
        links,
    };
    write_outputs(&result)?;

    if json {
        super::print_success_envelope(
            "map",
            serde_json::to_value(&result).map_err(super::io_error)?,
            Vec::new(),
            elapsed_timing(started),
        )?;
    } else if !quiet {
        match command.output {
            MapOutput::Markdown => print!("{}", map_markdown(&result)),
            MapOutput::Json => println!(
                "{}",
                serde_json::to_string_pretty(&result).map_err(super::io_error)?
            ),
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn resolve_input(command: &MapCommand, store: &SessionStore) -> Result<MapInput, ErrorResponse> {
    match (&command.url, &command.artifact) {
        (Some(url), None) => Ok(MapInput::Url(url.clone())),
        (None, Some(run_id)) => {
            validate_run_id(run_id)?;
            let run_dir = store.home().join("runs").join(run_id);
            if !run_dir.is_dir() {
                return Err(usage_error(format!("artifact run not found: {run_id}")));
            }
            Ok(MapInput::Artifact(run_id.clone()))
        }
        (None, None) => Err(usage_error(
            "map requires exactly one URL or --artifact <run-id>",
        )),
        (Some(_), Some(_)) => Err(usage_error(
            "map requires exactly one input; do not combine URL and --artifact",
        )),
    }
}

fn fetch_source(
    command: &MapCommand,
    output_dir: &Path,
    url: String,
) -> Result<SourceDocument, ErrorResponse> {
    let content_path = output_dir.join("source.html");
    let mut request = Aget::from_env()
        .map_err(super::io_error)?
        .get(url)
        .content_format(OutputFormat::Html)
        .output(&content_path);
    for session in &command.session {
        request = request.session(session.clone());
    }
    if let Some(selector) = &command.selector {
        request = request.selector(selector.clone());
    }
    if let Some(exclude_selector) = &command.exclude_selector {
        request = request.exclude_selector(exclude_selector.clone());
    }
    if let Some(wait_for_selector) = &command.wait_for_selector {
        request = request.wait_for_selector(wait_for_selector.clone());
    }
    for option in &command.backend_options {
        request = request.backend_option(option.key.clone(), option.value.clone());
    }

    let success = request.run().map_err(super::error_response)?;
    Ok(SourceDocument {
        url: success.url,
        final_url: success.final_url,
        content: success.content,
        artifact: None,
    })
}

fn artifact_source(store: &SessionStore, run_id: &str) -> Result<SourceDocument, ErrorResponse> {
    let run_dir = store.home().join("runs").join(run_id);
    let metadata_path = run_dir.join("metadata.json");
    let metadata = read_json(&metadata_path)?;
    if metadata.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(usage_error("map --artifact requires a successful get run"));
    }
    let content_path = metadata
        .pointer("/artifacts/content")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| run_dir.join("content.md"));
    if !content_path.starts_with(&run_dir) {
        return Err(usage_error(
            "map --artifact refuses external caller-owned content paths",
        ));
    }
    let content = fs::read_to_string(&content_path).map_err(super::io_error)?;
    Ok(SourceDocument {
        url: metadata
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        final_url: metadata
            .get("final_url")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                metadata
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
            })
            .to_string(),
        content,
        artifact: Some(run_id.to_string()),
    })
}

fn extract_links(
    source: &SourceDocument,
    command: &MapCommand,
) -> Result<(Vec<MapLink>, usize), ErrorResponse> {
    let document = Html::parse_document(&source.content);
    let selector = Selector::parse("a[href]").map_err(super::io_error)?;
    let base = document_base_url(&document)
        .or_else(|| Url::parse(&source.final_url).ok())
        .or_else(|| Url::parse(&source.url).ok())
        .ok_or_else(|| usage_error("map source URL cannot be used as a link base"))?;
    let path_prefix = path_prefix(&base);
    let filter_same_origin = !command.any_origin;
    let filter_same_path = !command.any_path;

    let mut seen = BTreeSet::new();
    let mut links = Vec::new();
    let mut discovered = 0usize;

    for element in document.select(&selector) {
        let Some(href) = element.value().attr("href") else {
            continue;
        };
        let Some(url) = resolve_link(&base, href) else {
            continue;
        };
        discovered += 1;
        let normalized = normalize_url(url);
        if !seen.insert(normalized.clone()) {
            continue;
        }
        let same_origin = same_origin(&base, &normalized);
        let same_path = normalized.path().starts_with(&path_prefix);
        let content_type = infer_content_type(&normalized);
        if filter_same_origin && !same_origin {
            continue;
        }
        if filter_same_path && !same_path {
            continue;
        }
        if !command.include.is_empty()
            && !command
                .include
                .iter()
                .any(|pattern| simple_match(pattern, normalized.as_str()))
        {
            continue;
        }
        if command
            .exclude
            .iter()
            .any(|pattern| simple_match(pattern, normalized.as_str()))
        {
            continue;
        }
        if !command.content_types.is_empty()
            && !command
                .content_types
                .iter()
                .any(|expected| Some(expected.as_str()) == content_type.as_deref())
        {
            continue;
        }
        links.push(MapLink {
            url: normalized.to_string(),
            text: link_text(&element),
            source_url: source.final_url.clone(),
            same_origin,
            same_path,
            content_type,
        });
        if links.len() == command.max_links {
            break;
        }
    }

    links.sort_by(|left, right| left.url.cmp(&right.url));
    Ok((links, discovered))
}

fn resolve_link(base: &Url, href: &str) -> Option<Url> {
    let href = href.trim();
    if href.is_empty()
        || href.starts_with('#')
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("javascript:")
    {
        return None;
    }
    base.join(href).ok()
}

fn document_base_url(document: &Html) -> Option<Url> {
    let selector = Selector::parse("base[href]").ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("href"))
        .and_then(|href| Url::parse(href).ok())
}

fn normalize_url(mut url: Url) -> Url {
    url.set_fragment(None);
    url
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn path_prefix(url: &Url) -> String {
    let path = url.path();
    if path.ends_with('/') {
        path.to_string()
    } else {
        match path.rsplit_once('/') {
            Some((prefix, _)) if !prefix.is_empty() => format!("{prefix}/"),
            _ => "/".to_string(),
        }
    }
}

fn link_text(element: &scraper::ElementRef<'_>) -> String {
    let text = element
        .text()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() {
        element.value().attr("aria-label").unwrap_or("").to_string()
    } else {
        text
    }
}

fn infer_content_type(url: &Url) -> Option<String> {
    let path = url.path().to_ascii_lowercase();
    if path.ends_with(".pdf") {
        Some("application/pdf".to_string())
    } else if path.ends_with(".json") {
        Some("application/json".to_string())
    } else if path.ends_with(".txt") || path.ends_with(".md") {
        Some("text/plain".to_string())
    } else if path.ends_with(".html")
        || path.ends_with(".htm")
        || path.ends_with('/')
        || !path.rsplit('/').next().unwrap_or("").contains('.')
    {
        Some("text/html".to_string())
    } else {
        None
    }
}

fn simple_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" || pattern.is_empty() {
        return true;
    }
    let original = value;
    let parts = pattern.split('*').collect::<Vec<_>>();
    if parts.len() == 1 {
        return value.contains(pattern);
    }
    let mut remaining = value;
    if !pattern.starts_with('*') {
        let Some(stripped) = remaining.strip_prefix(parts[0]) else {
            return false;
        };
        remaining = stripped;
    }
    for part in parts.iter().filter(|part| !part.is_empty()) {
        let Some(index) = remaining.find(part) else {
            return false;
        };
        remaining = &remaining[index + part.len()..];
    }
    pattern.ends_with('*')
        || parts
            .iter()
            .rev()
            .find(|part| !part.is_empty())
            .map_or(true, |part| original.ends_with(part))
}

fn write_outputs(result: &MapResult) -> Result<(), ErrorResponse> {
    let metadata = serde_json::json!({
        "ok": true,
        "command": "map",
        "url": result.source.url,
        "final_url": result.source.final_url,
        "artifacts": result.artifacts,
        "summary": result.summary,
    });
    let metadata_path = result
        .artifacts
        .json
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("metadata.json");
    fs::write(
        metadata_path,
        serde_json::to_vec_pretty(&metadata).map_err(super::io_error)?,
    )
    .map_err(super::io_error)?;
    fs::write(
        &result.artifacts.json,
        serde_json::to_vec_pretty(result).map_err(super::io_error)?,
    )
    .map_err(super::io_error)?;
    fs::write(&result.artifacts.markdown, map_markdown(result)).map_err(super::io_error)
}

fn map_markdown(result: &MapResult) -> String {
    let mut markdown = format!(
        "# aget map\n\nSource: {}\nLinks: {}\n\n",
        result.source.final_url, result.summary.emitted
    );
    for link in &result.links {
        let label = if link.text.is_empty() {
            link.url.as_str()
        } else {
            link.text.as_str()
        };
        markdown.push_str(&format!("- [{}]({})\n", label, link.url));
    }
    markdown
}

fn read_json(path: &Path) -> Result<Value, ErrorResponse> {
    let text = fs::read_to_string(path).map_err(super::io_error)?;
    serde_json::from_str(&text).map_err(super::io_error)
}

fn validate_run_id(run_id: &str) -> Result<(), ErrorResponse> {
    if run_id.is_empty()
        || run_id == "."
        || run_id == ".."
        || !run_id.starts_with("run-")
        || !run_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(usage_error(format!("invalid artifact run id '{run_id}'")));
    }
    Ok(())
}

fn map_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("run-{}-{nanos}-map", std::process::id())
}

fn usage_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::UsageError, message)
}

fn elapsed_timing(started: Instant) -> aget::TimingMs {
    aget::TimingMs {
        total: started.elapsed().as_millis(),
    }
}

enum MapInput {
    Url(String),
    Artifact(String),
}

struct SourceDocument {
    url: String,
    final_url: String,
    content: String,
    artifact: Option<String>,
}

#[derive(Debug, Serialize)]
struct MapResult {
    summary: MapSummary,
    source: MapSource,
    artifacts: MapArtifacts,
    links: Vec<MapLink>,
}

#[derive(Debug, Serialize)]
struct MapSummary {
    discovered: usize,
    emitted: usize,
    truncated: bool,
}

#[derive(Debug, Serialize)]
struct MapSource {
    url: String,
    final_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact: Option<String>,
}

#[derive(Debug, Serialize)]
struct MapArtifacts {
    json: PathBuf,
    markdown: PathBuf,
}

#[derive(Debug, Serialize)]
struct MapLink {
    url: String,
    text: String,
    source_url: String,
    same_origin: bool,
    same_path: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
}
