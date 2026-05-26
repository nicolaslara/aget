use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aget::error::ErrorBody;
use aget::{
    Aget, BatchCommand, CacheMetadata, ErrorCode, ErrorResponse, GetSuccess, UsageMetrics,
    ENVELOPE_SCHEMA_VERSION,
};
use serde::Serialize;

const MAX_BATCH_CONCURRENCY: usize = 8;

pub(super) fn run_batch(
    command: BatchCommand,
    json: bool,
    quiet: bool,
) -> Result<ExitCode, ErrorResponse> {
    let started = Instant::now();
    let urls = collect_urls(&command)?;
    if command.concurrency == 0 || command.concurrency > MAX_BATCH_CONCURRENCY {
        return Err(usage_error(format!(
            "batch concurrency must be between 1 and {MAX_BATCH_CONCURRENCY}"
        )));
    }

    let output_dir = resolve_output_dir(command.output_dir.clone())?;
    fs::create_dir_all(output_dir.join("items")).map_err(super::io_error)?;

    let mut items = urls
        .into_iter()
        .enumerate()
        .map(|(index, input)| match input {
            BatchInput::Fetch(url) => BatchItem::pending(index, url),
            BatchInput::Skipped { url, reason } => BatchItem::skipped(index, url, reason),
        })
        .collect::<Vec<_>>();

    let config = BatchFetchConfig::from_command(&command, output_dir.clone());
    let fetch_indices = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| (item.status == BatchItemStatus::Pending).then_some(index))
        .collect::<Vec<_>>();

    let mut stop = false;
    for chunk in fetch_indices.chunks(command.concurrency.max(1)) {
        if stop {
            for index in chunk {
                items[*index].mark_skipped("fail_fast");
            }
            continue;
        }

        let mut results = Vec::new();
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for index in chunk {
                let config = config.clone();
                let item = items[*index].clone();
                handles.push(scope.spawn(move || (*index, fetch_item(item, &config))));
            }
            for handle in handles {
                results.push(handle.join().expect("batch worker panicked"));
            }
        });

        for (index, result) in results {
            items[index] = result;
        }

        if command.fail_fast
            && chunk
                .iter()
                .any(|index| items[*index].status == BatchItemStatus::Failed)
        {
            stop = true;
        }
    }

    if stop {
        for item in &mut items {
            if item.status == BatchItemStatus::Pending {
                item.mark_skipped("fail_fast");
            }
        }
    }

    let summary = BatchSummary::from_items(&items);
    let manifest = BatchManifest {
        summary,
        artifacts: BatchArtifacts {
            manifest: output_dir.join("manifest.json"),
            markdown: output_dir.join("manifest.md"),
        },
        items,
    };
    write_manifest(&manifest)?;

    if json {
        print_batch_envelope(&manifest, started)?;
    } else if !quiet {
        println!(
            "Batch: {} requested, {} succeeded, {} failed, {} skipped",
            manifest.summary.requested,
            manifest.summary.succeeded,
            manifest.summary.failed,
            manifest.summary.skipped
        );
        println!("Manifest: {}", manifest.artifacts.manifest.display());
    }

    Ok(if manifest.summary.failed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn collect_urls(command: &BatchCommand) -> Result<Vec<BatchInput>, ErrorResponse> {
    if command.stdin && (!command.urls.is_empty() || command.file.is_some()) {
        return Err(usage_error(
            "batch --stdin cannot be combined with positional URLs or --file",
        ));
    }

    let mut raw = Vec::new();
    raw.extend(command.urls.iter().cloned());
    if let Some(file) = &command.file {
        raw.extend(read_url_lines(
            &fs::read_to_string(file).map_err(super::io_error)?,
        ));
    }
    if command.stdin {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .map_err(super::io_error)?;
        raw.extend(read_url_lines(&input));
    }

    if raw.is_empty() {
        return Err(usage_error(
            "batch requires positional URLs, --file, or --stdin",
        ));
    }

    let mut seen = BTreeSet::new();
    let mut inputs = Vec::new();
    for url in raw {
        if !supported_input(&url) {
            return Err(usage_error(format!("unsupported batch URL input '{url}'")));
        }
        if seen.insert(url.clone()) {
            inputs.push(BatchInput::Fetch(url));
        } else {
            inputs.push(BatchInput::Skipped {
                url,
                reason: "duplicate".to_string(),
            });
        }
    }
    Ok(inputs)
}

fn read_url_lines(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn supported_input(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("raw:")
        || url.starts_with("raw://")
        || url.starts_with("file://")
}

fn resolve_output_dir(output_dir: Option<PathBuf>) -> Result<PathBuf, ErrorResponse> {
    if let Some(output_dir) = output_dir {
        return Ok(output_dir);
    }
    let store = aget::SessionStore::from_env().map_err(super::io_error)?;
    Ok(store.home().join("runs").join(batch_run_id()))
}

fn batch_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("batch-{}-{nanos}", std::process::id())
}

fn fetch_item(mut item: BatchItem, config: &BatchFetchConfig) -> BatchItem {
    let item_dir = config.item_dir(item.index);
    if let Err(error) = fs::create_dir_all(&item_dir) {
        item.fail(ErrorBody {
            code: ErrorCode::IoError,
            message: error.to_string(),
            retry: None,
        });
        return item;
    }
    let content_path = item_dir.join("content.md");
    let request = Aget::from_env()
        .map_err(|error| ErrorBody {
            code: ErrorCode::IoError,
            message: error.to_string(),
            retry: None,
        })
        .map(|aget| {
            let mut request = aget
                .get(item.url.clone())
                .content_format(config.content_format)
                .output(content_path);
            for session in &config.sessions {
                request = request.session(session.clone());
            }
            if let Some(selector) = &config.selector {
                request = request.selector(selector.clone());
            }
            if let Some(exclude_selector) = &config.exclude_selector {
                request = request.exclude_selector(exclude_selector.clone());
            }
            if let Some(wait_for_selector) = &config.wait_for_selector {
                request = request.wait_for_selector(wait_for_selector.clone());
            }
            if let Some(max_chars) = config.max_chars {
                request = request.max_chars(max_chars);
            }
            request = request
                .cache_policy(config.cache_policy)
                .cache_ttl(config.cache_ttl);
            for option in &config.backend_options {
                request = request.backend_option(option.key.clone(), option.value.clone());
            }
            request
        });

    match request.and_then(|request| {
        request.run().map_err(|error| match error {
            aget::AgetError::Stable { code, message } => ErrorBody {
                code,
                message,
                retry: None,
            },
        })
    }) {
        Ok(success) => item.succeed(success),
        Err(error) => item.fail(error),
    }
    item
}

fn write_manifest(manifest: &BatchManifest) -> Result<(), ErrorResponse> {
    let json = serde_json::to_vec_pretty(manifest).map_err(super::io_error)?;
    fs::write(&manifest.artifacts.manifest, json).map_err(super::io_error)?;
    fs::write(&manifest.artifacts.markdown, manifest_markdown(manifest))
        .map_err(super::io_error)?;
    Ok(())
}

fn manifest_markdown(manifest: &BatchManifest) -> String {
    let mut markdown = format!(
        "# aget batch\n\nRequested: {}\nSucceeded: {}\nFailed: {}\nSkipped: {}\n\n",
        manifest.summary.requested,
        manifest.summary.succeeded,
        manifest.summary.failed,
        manifest.summary.skipped
    );
    for item in &manifest.items {
        markdown.push_str(&format!(
            "- {} `{}` {}\n",
            item.status.as_str(),
            item.url,
            item.error
                .as_ref()
                .map(|error| error.message.as_str())
                .unwrap_or("")
        ));
    }
    markdown
}

fn print_batch_envelope(manifest: &BatchManifest, started: Instant) -> Result<(), ErrorResponse> {
    let envelope = serde_json::json!({
        "ok": manifest.summary.failed == 0,
        "schema_version": ENVELOPE_SCHEMA_VERSION,
        "command": "batch",
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

fn usage_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::UsageError, message)
}

#[derive(Clone)]
struct BatchFetchConfig {
    sessions: Vec<String>,
    content_format: aget::OutputFormat,
    selector: Option<String>,
    exclude_selector: Option<String>,
    wait_for_selector: Option<String>,
    max_chars: Option<usize>,
    cache_policy: aget::CachePolicy,
    cache_ttl: std::time::Duration,
    backend_options: Vec<aget::ExtractorOption>,
    output_dir: PathBuf,
}

impl BatchFetchConfig {
    fn from_command(command: &BatchCommand, output_dir: PathBuf) -> Self {
        Self {
            sessions: command.session.clone(),
            content_format: command.content_format,
            selector: command.selector.clone(),
            exclude_selector: command.exclude_selector.clone(),
            wait_for_selector: command.wait_for_selector.clone(),
            max_chars: command.max_chars,
            cache_policy: command.cache.policy(),
            cache_ttl: command.cache.cache_ttl,
            backend_options: command.backend_options.clone(),
            output_dir,
        }
    }

    fn item_dir(&self, index: usize) -> PathBuf {
        self.output_dir.join("items").join(format!("{index:06}"))
    }
}

enum BatchInput {
    Fetch(String),
    Skipped { url: String, reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum BatchItemStatus {
    Pending,
    Ok,
    Failed,
    Skipped,
}

impl BatchItemStatus {
    fn as_str(self) -> &'static str {
        match self {
            BatchItemStatus::Pending => "pending",
            BatchItemStatus::Ok => "ok",
            BatchItemStatus::Failed => "failed",
            BatchItemStatus::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct BatchItem {
    index: usize,
    url: String,
    status: BatchItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifacts: Option<BatchItemArtifacts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache: Option<CacheMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<UsageMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorBody>,
}

impl BatchItem {
    fn pending(index: usize, url: String) -> Self {
        Self {
            index,
            url,
            status: BatchItemStatus::Pending,
            reason: None,
            artifacts: None,
            cache: None,
            usage: None,
            error: None,
        }
    }

    fn skipped(index: usize, url: String, reason: String) -> Self {
        Self {
            index,
            url,
            status: BatchItemStatus::Skipped,
            reason: Some(reason),
            artifacts: None,
            cache: None,
            usage: None,
            error: None,
        }
    }

    fn succeed(&mut self, success: GetSuccess) {
        self.status = BatchItemStatus::Ok;
        self.artifacts = Some(BatchItemArtifacts {
            content: PathBuf::from(success.artifacts.content),
            metadata: PathBuf::from(success.artifacts.metadata),
        });
        self.cache = Some(success.cache);
        self.usage = Some(success.usage);
    }

    fn fail(&mut self, error: ErrorBody) {
        self.status = BatchItemStatus::Failed;
        self.error = Some(error);
    }

    fn mark_skipped(&mut self, reason: impl Into<String>) {
        self.status = BatchItemStatus::Skipped;
        self.reason = Some(reason.into());
    }
}

#[derive(Debug, Clone, Serialize)]
struct BatchItemArtifacts {
    content: PathBuf,
    metadata: PathBuf,
}

#[derive(Debug, Serialize)]
struct BatchManifest {
    summary: BatchSummary,
    artifacts: BatchArtifacts,
    items: Vec<BatchItem>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct BatchSummary {
    requested: usize,
    succeeded: usize,
    failed: usize,
    skipped: usize,
}

impl BatchSummary {
    fn from_items(items: &[BatchItem]) -> Self {
        Self {
            requested: items.len(),
            succeeded: items
                .iter()
                .filter(|item| item.status == BatchItemStatus::Ok)
                .count(),
            failed: items
                .iter()
                .filter(|item| item.status == BatchItemStatus::Failed)
                .count(),
            skipped: items
                .iter()
                .filter(|item| item.status == BatchItemStatus::Skipped)
                .count(),
        }
    }
}

#[derive(Debug, Serialize)]
struct BatchArtifacts {
    manifest: PathBuf,
    markdown: PathBuf,
}
