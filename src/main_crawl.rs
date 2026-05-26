use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aget::error::ErrorBody;
use aget::{
    Aget, CacheMetadata, CrawlCommand, ErrorCode, ErrorResponse, GetSuccess, OutputFormat,
    UsageMetrics,
};
use scraper::{Html, Selector};
use serde::Serialize;
use url::Url;

const MAX_CRAWL_LIMIT: usize = 100;
const MAX_CRAWL_DEPTH: usize = 5;
const MAX_CRAWL_CONCURRENCY: usize = 8;

pub(super) fn run_crawl(
    command: CrawlCommand,
    json: bool,
    quiet: bool,
) -> Result<ExitCode, ErrorResponse> {
    let started = Instant::now();
    validate_command(&command)?;
    let start_url = Url::parse(&command.url)
        .map_err(|_| usage_error(format!("crawl start URL is invalid: {}", command.url)))?;
    let output_dir = resolve_output_dir(command.output_dir.clone())?;
    fs::create_dir_all(output_dir.join("items")).map_err(super::io_error)?;

    let mut queue = VecDeque::from([CrawlCandidate {
        url: normalize_url(start_url.clone()).to_string(),
        depth: 0,
        source_url: None,
    }]);
    let mut scheduled = BTreeSet::new();
    let mut items = Vec::new();
    let filter = CrawlFilter::from_command(&command, &start_url);

    while !queue.is_empty() && items.len() < command.limit {
        let mut pending = Vec::new();
        while pending.len() < command.concurrency && items.len() + pending.len() < command.limit {
            let Some(candidate) = queue.pop_front() else {
                break;
            };
            if !scheduled.insert(candidate.url.clone()) {
                continue;
            }
            let index = items.len() + pending.len();
            pending.push((index, candidate));
        }
        if pending.is_empty() {
            continue;
        }

        let mut results = Vec::new();
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (index, candidate) in pending {
                let command = &command;
                let output_dir = &output_dir;
                handles.push(
                    scope.spawn(move || fetch_crawl_item(index, candidate, command, output_dir)),
                );
            }
            for handle in handles {
                results.push(handle.join().expect("crawl worker panicked"));
            }
        });

        for item in results {
            if item.status == CrawlItemStatus::Ok && item.depth < command.max_depth {
                for link in discover_links(&item, &filter) {
                    if items.len() + queue.len() >= command.limit {
                        break;
                    }
                    if !scheduled.contains(&link.url) {
                        queue.push_back(CrawlCandidate {
                            url: link.url,
                            depth: item.depth + 1,
                            source_url: Some(item.url.clone()),
                        });
                    }
                }
            }
            items.push(item);
        }
    }

    let summary = CrawlSummary::from_items(&items, command.limit);
    let manifest = CrawlManifest {
        summary,
        artifacts: CrawlArtifacts {
            manifest: output_dir.join("manifest.json"),
            markdown: output_dir.join("manifest.md"),
        },
        start_url: command.url,
        items,
    };
    write_manifest(&manifest)?;

    if json {
        print_crawl_envelope(&manifest, started)?;
    } else if !quiet {
        println!(
            "Crawl: {} fetched, {} succeeded, {} failed, {} limit",
            manifest.summary.fetched,
            manifest.summary.succeeded,
            manifest.summary.failed,
            manifest.summary.limit
        );
        println!("Manifest: {}", manifest.artifacts.manifest.display());
    }

    Ok(if manifest.summary.failed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn validate_command(command: &CrawlCommand) -> Result<(), ErrorResponse> {
    if command.limit == 0 || command.limit > MAX_CRAWL_LIMIT {
        return Err(usage_error(format!(
            "crawl --limit must be between 1 and {MAX_CRAWL_LIMIT}"
        )));
    }
    if command.max_depth > MAX_CRAWL_DEPTH {
        return Err(usage_error(format!(
            "crawl --max-depth must be at most {MAX_CRAWL_DEPTH}"
        )));
    }
    if command.concurrency == 0 || command.concurrency > MAX_CRAWL_CONCURRENCY {
        return Err(usage_error(format!(
            "crawl --concurrency must be between 1 and {MAX_CRAWL_CONCURRENCY}"
        )));
    }
    Ok(())
}

fn resolve_output_dir(output_dir: Option<PathBuf>) -> Result<PathBuf, ErrorResponse> {
    if let Some(output_dir) = output_dir {
        return Ok(output_dir);
    }
    let store = aget::SessionStore::from_env().map_err(super::io_error)?;
    Ok(store.home().join("runs").join(crawl_run_id()))
}

fn fetch_crawl_item(
    index: usize,
    candidate: CrawlCandidate,
    command: &CrawlCommand,
    output_dir: &Path,
) -> CrawlItem {
    let item_dir = output_dir.join("items").join(format!("{index:06}"));
    if let Err(error) = fs::create_dir_all(&item_dir) {
        return CrawlItem::failed(
            index,
            candidate,
            ErrorBody {
                code: ErrorCode::IoError,
                message: error.to_string(),
                retry: None,
            },
        );
    }

    let html_result = run_get(
        &candidate.url,
        OutputFormat::Html,
        item_dir.join("source.html"),
        command,
    );
    let content_result = if command.content_format == OutputFormat::Html {
        html_result.clone()
    } else {
        run_get(
            &candidate.url,
            command.content_format,
            item_dir.join("content.md"),
            command,
        )
    };

    match (html_result, content_result) {
        (Ok(html), Ok(content)) => CrawlItem::ok(index, candidate, html, content),
        (Err(error), _) | (_, Err(error)) => CrawlItem::failed(index, candidate, error),
    }
}

fn run_get(
    url: &str,
    format: OutputFormat,
    output: PathBuf,
    command: &CrawlCommand,
) -> Result<GetSuccess, ErrorBody> {
    let request = Aget::from_env()
        .map_err(|error| ErrorBody {
            code: ErrorCode::IoError,
            message: error.to_string(),
            retry: None,
        })
        .map(|aget| {
            let mut request = aget
                .get(url.to_string())
                .content_format(format)
                .output(output);
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
            if let Some(max_chars) = command.max_chars {
                request = request.max_chars(max_chars);
            }
            request = request
                .cache_policy(command.cache.policy())
                .cache_ttl(command.cache.cache_ttl);
            for option in &command.backend_options {
                request = request.backend_option(option.key.clone(), option.value.clone());
            }
            request
        })?;

    request.run().map_err(|error| match error {
        aget::AgetError::Stable { code, message } => ErrorBody {
            code,
            message,
            retry: None,
        },
    })
}

fn discover_links(item: &CrawlItem, filter: &CrawlFilter) -> Vec<CrawlDiscoveredLink> {
    let Some(html_path) = item
        .artifacts
        .as_ref()
        .map(|artifacts| artifacts.source_html.as_path())
    else {
        return Vec::new();
    };
    let Ok(html) = fs::read_to_string(html_path) else {
        return Vec::new();
    };
    let document = Html::parse_document(&html);
    let Ok(selector) = Selector::parse("a[href]") else {
        return Vec::new();
    };
    let base = document_base_url(&document)
        .or_else(|| Url::parse(&item.final_url).ok())
        .or_else(|| Url::parse(&item.url).ok());
    let Some(base) = base else {
        return Vec::new();
    };

    let mut seen = BTreeSet::new();
    let mut links = Vec::new();
    for element in document.select(&selector) {
        let Some(href) = element.value().attr("href") else {
            continue;
        };
        let Some(url) = resolve_link(&base, href) else {
            continue;
        };
        let normalized = normalize_url(url);
        if !filter.allows(&normalized) || !seen.insert(normalized.to_string()) {
            continue;
        }
        links.push(CrawlDiscoveredLink {
            url: normalized.to_string(),
        });
    }
    links
}

fn document_base_url(document: &Html) -> Option<Url> {
    let selector = Selector::parse("base[href]").ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("href"))
        .and_then(|href| Url::parse(href).ok())
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

fn normalize_url(mut url: Url) -> Url {
    url.set_fragment(None);
    url
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

fn write_manifest(manifest: &CrawlManifest) -> Result<(), ErrorResponse> {
    let metadata = serde_json::json!({
        "ok": manifest.summary.failed == 0,
        "command": "crawl",
        "url": manifest.start_url,
        "artifacts": manifest.artifacts,
        "summary": manifest.summary,
    });
    let metadata_path = manifest
        .artifacts
        .manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("metadata.json");
    fs::write(
        metadata_path,
        serde_json::to_vec_pretty(&metadata).map_err(super::io_error)?,
    )
    .map_err(super::io_error)?;
    fs::write(
        &manifest.artifacts.manifest,
        serde_json::to_vec_pretty(manifest).map_err(super::io_error)?,
    )
    .map_err(super::io_error)?;
    fs::write(&manifest.artifacts.markdown, crawl_markdown(manifest)).map_err(super::io_error)
}

fn print_crawl_envelope(manifest: &CrawlManifest, started: Instant) -> Result<(), ErrorResponse> {
    let envelope = serde_json::json!({
        "ok": manifest.summary.failed == 0,
        "schema_version": aget::ENVELOPE_SCHEMA_VERSION,
        "command": "crawl",
        "data": manifest,
        "warnings": [],
        "timing_ms": {"total": started.elapsed().as_millis()},
    });
    println!(
        "{}",
        serde_json::to_string(&envelope).map_err(super::io_error)?
    );
    Ok(())
}

fn crawl_markdown(manifest: &CrawlManifest) -> String {
    let mut markdown = format!(
        "# aget crawl\n\nStart: {}\nFetched: {}\nSucceeded: {}\nFailed: {}\n\n",
        manifest.start_url,
        manifest.summary.fetched,
        manifest.summary.succeeded,
        manifest.summary.failed
    );
    for item in &manifest.items {
        markdown.push_str(&format!(
            "- {} depth={} `{}`\n",
            item.status.as_str(),
            item.depth,
            item.url
        ));
    }
    markdown
}

fn crawl_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("run-{}-{nanos}-crawl", std::process::id())
}

fn usage_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::UsageError, message)
}

#[derive(Clone)]
struct CrawlCandidate {
    url: String,
    depth: usize,
    source_url: Option<String>,
}

struct CrawlDiscoveredLink {
    url: String,
}

struct CrawlFilter {
    start_origin: Url,
    path_prefix: String,
    any_origin: bool,
    any_path: bool,
    allow_domains: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
}

impl CrawlFilter {
    fn from_command(command: &CrawlCommand, start_url: &Url) -> Self {
        Self {
            start_origin: start_url.clone(),
            path_prefix: path_prefix(start_url),
            any_origin: command.any_origin,
            any_path: command.any_path,
            allow_domains: command.allow_domains.clone(),
            include: command.include.clone(),
            exclude: command.exclude.clone(),
        }
    }

    fn allows(&self, url: &Url) -> bool {
        if !self.any_origin
            && !same_origin(&self.start_origin, url)
            && !self
                .allow_domains
                .iter()
                .any(|domain| host_matches(url, domain))
        {
            return false;
        }
        if !self.any_path && !url.path().starts_with(&self.path_prefix) {
            return false;
        }
        if !self.include.is_empty()
            && !self
                .include
                .iter()
                .any(|pattern| simple_match(pattern, url.as_str()))
        {
            return false;
        }
        if self
            .exclude
            .iter()
            .any(|pattern| simple_match(pattern, url.as_str()))
        {
            return false;
        }
        true
    }
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn host_matches(url: &Url, domain: &str) -> bool {
    url.host_str()
        .is_some_and(|host| host == domain || host.ends_with(&format!(".{domain}")))
}

#[derive(Debug, Serialize)]
struct CrawlManifest {
    summary: CrawlSummary,
    artifacts: CrawlArtifacts,
    start_url: String,
    items: Vec<CrawlItem>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct CrawlSummary {
    limit: usize,
    fetched: usize,
    succeeded: usize,
    failed: usize,
}

impl CrawlSummary {
    fn from_items(items: &[CrawlItem], limit: usize) -> Self {
        Self {
            limit,
            fetched: items.len(),
            succeeded: items
                .iter()
                .filter(|item| item.status == CrawlItemStatus::Ok)
                .count(),
            failed: items
                .iter()
                .filter(|item| item.status == CrawlItemStatus::Failed)
                .count(),
        }
    }
}

#[derive(Debug, Serialize)]
struct CrawlArtifacts {
    manifest: PathBuf,
    markdown: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CrawlItemStatus {
    Ok,
    Failed,
}

impl CrawlItemStatus {
    fn as_str(self) -> &'static str {
        match self {
            CrawlItemStatus::Ok => "ok",
            CrawlItemStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Serialize)]
struct CrawlItem {
    index: usize,
    url: String,
    final_url: String,
    depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_url: Option<String>,
    status: CrawlItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifacts: Option<CrawlItemArtifacts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache: Option<CacheMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<UsageMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorBody>,
}

impl CrawlItem {
    fn ok(index: usize, candidate: CrawlCandidate, html: GetSuccess, content: GetSuccess) -> Self {
        Self {
            index,
            url: candidate.url,
            final_url: content.final_url,
            depth: candidate.depth,
            source_url: candidate.source_url,
            status: CrawlItemStatus::Ok,
            artifacts: Some(CrawlItemArtifacts {
                content: PathBuf::from(content.artifacts.content),
                metadata: PathBuf::from(content.artifacts.metadata),
                source_html: PathBuf::from(html.artifacts.content),
            }),
            cache: Some(content.cache),
            usage: Some(content.usage),
            error: None,
        }
    }

    fn failed(index: usize, candidate: CrawlCandidate, error: ErrorBody) -> Self {
        Self {
            index,
            url: candidate.url.clone(),
            final_url: candidate.url,
            depth: candidate.depth,
            source_url: candidate.source_url,
            status: CrawlItemStatus::Failed,
            artifacts: None,
            cache: None,
            usage: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize)]
struct CrawlItemArtifacts {
    content: PathBuf,
    metadata: PathBuf,
    source_html: PathBuf,
}
