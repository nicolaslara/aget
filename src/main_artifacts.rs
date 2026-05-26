use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aget::{
    ArtifactsSubcommand, DeleteArtifactCommand, ErrorCode, ErrorResponse, InspectArtifactCommand,
    PruneArtifactsCommand, SessionStore, TimingMs,
};
use serde::Serialize;
use serde_json::Value;

pub(super) fn run_artifacts(
    command: ArtifactsSubcommand,
    json: bool,
    quiet: bool,
) -> Result<(), ErrorResponse> {
    let command_name = artifacts_command_name(&command);
    let started = Instant::now();

    (|| {
        let store = SessionStore::from_env().map_err(super::io_error)?;
        let runs_dir = store.home().join("runs");
        match command {
            ArtifactsSubcommand::List => run_list(&runs_dir, json, quiet, started),
            ArtifactsSubcommand::Inspect(command) => {
                run_inspect(&runs_dir, command, json, quiet, started)
            }
            ArtifactsSubcommand::Delete(command) => {
                run_delete(&runs_dir, command, json, quiet, started)
            }
            ArtifactsSubcommand::Prune(command) => {
                run_prune(&runs_dir, command, json, quiet, started)
            }
        }
    })()
    .map_err(|error: ErrorResponse| error.with_command(command_name))
}

fn artifacts_command_name(command: &ArtifactsSubcommand) -> &'static str {
    match command {
        ArtifactsSubcommand::List => "artifacts.list",
        ArtifactsSubcommand::Inspect(_) => "artifacts.inspect",
        ArtifactsSubcommand::Delete(_) => "artifacts.delete",
        ArtifactsSubcommand::Prune(_) => "artifacts.prune",
    }
}

fn run_list(
    runs_dir: &Path,
    json: bool,
    quiet: bool,
    started: Instant,
) -> Result<(), ErrorResponse> {
    let runs = list_runs(runs_dir)?;
    let summary = artifact_summary(&runs);
    let data = ArtifactList { runs, summary };
    if json {
        super::print_success_envelope(
            "artifacts.list",
            serde_json::to_value(&data).map_err(super::io_error)?,
            Vec::new(),
            elapsed_timing(started),
        )?;
    } else if !quiet {
        if data.runs.is_empty() {
            println!("No run artifacts found");
        } else {
            println!(
                "{:<34} {:>8} {:>10} {:<6} {:<9} {:<24} CONTENT",
                "RUN ID", "AGE", "SIZE", "STATUS", "SENSITIVE", "HOST"
            );
            for run in &data.runs {
                println!(
                    "{:<34} {:>8} {:>10} {:<6} {:<9} {:<24} {}",
                    run.run_id,
                    format_age(run.age_seconds),
                    run.size_bytes,
                    status_label(run.ok, run.metadata.valid),
                    if run.sensitive { "yes" } else { "no" },
                    run.source.host.as_deref().unwrap_or("-"),
                    run.content.ownership
                );
            }
        }
    }
    Ok(())
}

fn run_inspect(
    runs_dir: &Path,
    command: InspectArtifactCommand,
    json: bool,
    quiet: bool,
    started: Instant,
) -> Result<(), ErrorResponse> {
    let run = load_run(runs_dir, &command.run_id)?;
    let data = inspect_data(&run)?;
    if json {
        super::print_success_envelope(
            "artifacts.inspect",
            serde_json::to_value(&data).map_err(super::io_error)?,
            Vec::new(),
            elapsed_timing(started),
        )?;
    } else if !quiet {
        println!("Run: {}", data.run_id);
        println!("Directory: {}", data.run_dir.display());
        println!("Status: {}", status_label(data.ok, data.metadata.valid));
        println!("Sensitive: {}", data.sensitive);
        println!("URL: {}", data.source.url.as_deref().unwrap_or("-"));
        println!("Size: {} bytes", data.size_bytes);
        for file in &data.files {
            println!(
                "- {} {} {} bytes {}",
                file.kind,
                file.ownership,
                file.size_bytes.unwrap_or(0),
                file.path.display()
            );
        }
    }
    Ok(())
}

fn run_delete(
    runs_dir: &Path,
    command: DeleteArtifactCommand,
    json: bool,
    quiet: bool,
    started: Instant,
) -> Result<(), ErrorResponse> {
    let run = load_run(runs_dir, &command.run_id)?;
    if !command.yes {
        return Err(usage_error(format!(
            "deleting artifact {} requires --yes",
            command.run_id
        )));
    }
    let result = delete_run(run)?;
    if json {
        super::print_success_envelope(
            "artifacts.delete",
            serde_json::to_value(&result).map_err(super::io_error)?,
            Vec::new(),
            elapsed_timing(started),
        )?;
    } else if !quiet {
        println!(
            "Deleted artifact {} and freed {} bytes",
            result.run_id, result.freed_bytes
        );
        for path in &result.preserved_external_paths {
            println!("Preserved external content {}", path.display());
        }
    }
    Ok(())
}

fn run_prune(
    runs_dir: &Path,
    command: PruneArtifactsCommand,
    json: bool,
    quiet: bool,
    started: Instant,
) -> Result<(), ErrorResponse> {
    if command.dry_run && command.yes {
        return Err(usage_error("--dry-run and --yes cannot be used together"));
    }
    if command.older_than.is_none() && command.keep_last.is_none() && command.max_bytes.is_none() {
        return Err(usage_error(
            "artifacts prune requires --older-than, --keep-last, or --max-bytes",
        ));
    }

    let runs = list_runs(runs_dir)?;
    let selected = prune_selection(&runs, &command);
    let dry_run = !command.yes;
    let mut deleted = Vec::new();
    let mut preserved_external_paths = Vec::new();
    let would_free_bytes = selected.iter().map(|run| run.size_bytes).sum::<u64>();

    if !dry_run {
        for run in selected.iter().cloned() {
            let result = delete_run(run)?;
            preserved_external_paths.extend(result.preserved_external_paths);
            deleted.push(result.run_id);
        }
    }

    let data = PruneResult {
        dry_run,
        would_delete: selected.iter().map(|run| run.run_id.clone()).collect(),
        deleted,
        would_free_bytes,
        preserved_external_paths,
    };

    if json {
        super::print_success_envelope(
            "artifacts.prune",
            serde_json::to_value(&data).map_err(super::io_error)?,
            Vec::new(),
            elapsed_timing(started),
        )?;
    } else if !quiet {
        if dry_run {
            println!(
                "Would delete {} artifact(s), freeing {} bytes",
                data.would_delete.len(),
                data.would_free_bytes
            );
            println!("Rerun with --yes to delete");
        } else {
            println!(
                "Deleted {} artifact(s), freed {} bytes",
                data.deleted.len(),
                data.would_free_bytes
            );
        }
    }
    Ok(())
}

fn prune_selection(runs: &[RunSummary], command: &PruneArtifactsCommand) -> Vec<RunSummary> {
    let mut newest = runs.to_vec();
    newest.sort_by(|left, right| right.modified_unix.cmp(&left.modified_unix));
    let protected = command
        .keep_last
        .map(|keep_last| {
            newest
                .iter()
                .take(keep_last)
                .map(|run| run.run_id.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let mut selected = BTreeSet::new();
    if let Some(older_than) = command.older_than {
        for run in runs {
            if run.age_seconds > older_than && !protected.contains(&run.run_id) {
                selected.insert(run.run_id.clone());
            }
        }
    }

    if command.older_than.is_none() && command.max_bytes.is_none() && command.keep_last.is_some() {
        for run in runs {
            if !protected.contains(&run.run_id) {
                selected.insert(run.run_id.clone());
            }
        }
    }

    if let Some(max_bytes) = command.max_bytes {
        let mut total = runs.iter().map(|run| run.size_bytes).sum::<u64>();
        for run_id in &selected {
            if let Some(run) = runs.iter().find(|run| &run.run_id == run_id) {
                total = total.saturating_sub(run.size_bytes);
            }
        }
        let mut oldest = runs
            .iter()
            .filter(|run| !protected.contains(&run.run_id))
            .cloned()
            .collect::<Vec<_>>();
        oldest.sort_by(|left, right| left.modified_unix.cmp(&right.modified_unix));
        for run in oldest {
            if total <= max_bytes {
                break;
            }
            if selected.insert(run.run_id.clone()) {
                total = total.saturating_sub(run.size_bytes);
            }
        }
    }

    let mut selected_runs = runs
        .iter()
        .filter(|run| selected.contains(&run.run_id))
        .cloned()
        .collect::<Vec<_>>();
    selected_runs.sort_by(|left, right| left.modified_unix.cmp(&right.modified_unix));
    selected_runs
}

fn list_runs(runs_dir: &Path) -> Result<Vec<RunSummary>, ErrorResponse> {
    let mut runs = Vec::new();
    for entry in fs::read_dir(runs_dir).map_err(super::io_error)? {
        let entry = entry.map_err(super::io_error)?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(run_id) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if validate_run_id(run_id).is_err() {
            continue;
        }
        runs.push(load_run(runs_dir, run_id)?);
    }
    runs.sort_by(|left, right| right.modified_unix.cmp(&left.modified_unix));
    Ok(runs)
}

fn load_run(runs_dir: &Path, run_id: &str) -> Result<RunSummary, ErrorResponse> {
    validate_run_id(run_id)?;
    let run_dir = runs_dir.join(run_id);
    if !run_dir.starts_with(runs_dir) {
        return Err(usage_error("run id resolves outside AGET_HOME/runs"));
    }
    if !run_dir.is_dir() {
        return Err(usage_error(format!("artifact run not found: {run_id}")));
    }
    let metadata_path = run_dir.join("metadata.json");
    let metadata_result = read_metadata(&metadata_path);
    let metadata = metadata_result.as_ref().ok();
    let valid = metadata.is_some();
    let source = source_summary(metadata, valid);
    let content = content_summary(&run_dir, metadata);
    let modified = fs::metadata(&run_dir)
        .and_then(|metadata| metadata.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let modified_unix = system_time_secs(modified);
    let now = system_time_secs(SystemTime::now());
    let size_bytes = dir_size(&run_dir).unwrap_or(0);
    Ok(RunSummary {
        run_id: run_id.to_string(),
        run_dir,
        modified_unix,
        age_seconds: now.saturating_sub(modified_unix),
        size_bytes,
        ok: metadata
            .and_then(|value| value.get("ok"))
            .and_then(Value::as_bool),
        sensitive: metadata
            .and_then(|value| value.get("sensitive"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        content_format: metadata
            .and_then(|value| value.get("content_format"))
            .and_then(Value::as_str)
            .map(str::to_string),
        extractor: metadata
            .and_then(|value| value.get("extractor"))
            .and_then(Value::as_str)
            .map(str::to_string),
        source,
        content,
        metadata: MetadataSummary {
            path: metadata_path,
            valid,
            error: metadata_result.err(),
        },
    })
}

fn inspect_data(run: &RunSummary) -> Result<InspectResult, ErrorResponse> {
    let mut files = Vec::new();
    files.push(FileEntry {
        path: run.metadata.path.clone(),
        kind: "metadata",
        ownership: "internal",
        exists: run.metadata.path.exists(),
        size_bytes: file_size(&run.metadata.path),
    });
    files.push(FileEntry {
        path: run.content.path.clone(),
        kind: "content",
        ownership: run.content.ownership,
        exists: run.content.exists,
        size_bytes: file_size(&run.content.path),
    });
    let metadata_value = if run.metadata.valid {
        let value = read_metadata(&run.metadata.path).map_err(super::io_error)?;
        files.extend(debug_file_entries(&run.run_dir, &value));
        files.extend(action_file_entries(&run.run_dir, &value));
        Some(redacted_metadata(value, run.sensitive))
    } else {
        None
    };
    Ok(InspectResult {
        run_id: run.run_id.clone(),
        run_dir: run.run_dir.clone(),
        size_bytes: run.size_bytes,
        ok: run.ok,
        sensitive: run.sensitive,
        source: run.source.clone(),
        metadata: InspectMetadata {
            path: run.metadata.path.clone(),
            valid: run.metadata.valid,
            error: run.metadata.error.clone(),
            value: metadata_value,
        },
        files,
    })
}

fn delete_run(run: RunSummary) -> Result<DeleteResult, ErrorResponse> {
    let freed_bytes = run.size_bytes;
    let preserved_external_paths = if run.content.ownership == "external" {
        vec![run.content.path.clone()]
    } else {
        Vec::new()
    };
    let deleted_paths = collect_files(&run.run_dir).map_err(super::io_error)?;
    fs::remove_dir_all(&run.run_dir).map_err(super::io_error)?;
    Ok(DeleteResult {
        run_id: run.run_id,
        deleted: true,
        deleted_paths,
        preserved_external_paths,
        freed_bytes,
    })
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

fn read_metadata(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn source_summary(metadata: Option<&Value>, valid: bool) -> SourceSummary {
    let sensitive = metadata
        .and_then(|value| value.get("sensitive"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let url = metadata
        .and_then(|value| value.get("url"))
        .and_then(Value::as_str)
        .map(|url| redact_url(url, sensitive));
    let final_url = metadata
        .and_then(|value| value.get("final_url"))
        .and_then(Value::as_str)
        .map(|url| redact_url(url, sensitive));
    let host = url
        .as_deref()
        .and_then(|url| url::Url::parse(url).ok())
        .and_then(|url| url.host_str().map(str::to_string));
    SourceSummary {
        url: valid.then_some(url).flatten(),
        final_url: valid.then_some(final_url).flatten(),
        host,
    }
}

fn content_summary(run_dir: &Path, metadata: Option<&Value>) -> ContentSummary {
    let path = metadata
        .and_then(|value| value.pointer("/artifacts/content"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| run_dir.join("content.md"));
    let ownership = if path.starts_with(run_dir) {
        "internal"
    } else {
        "external"
    };
    ContentSummary {
        exists: path.exists(),
        path,
        ownership,
    }
}

fn debug_file_entries(run_dir: &Path, metadata: &Value) -> Vec<FileEntry> {
    let mut files = Vec::new();
    for (pointer, kind) in [
        ("/artifacts/debug/screenshot/path", "debug-screenshot"),
        ("/artifacts/debug/trace/path", "debug-trace"),
    ] {
        let Some(path) = metadata.pointer(pointer).and_then(Value::as_str) else {
            continue;
        };
        let path = PathBuf::from(path);
        let ownership = if path.starts_with(run_dir) {
            "internal"
        } else {
            "external"
        };
        files.push(FileEntry {
            size_bytes: file_size(&path),
            exists: path.exists(),
            path,
            kind,
            ownership,
        });
    }
    files
}

fn action_file_entries(run_dir: &Path, metadata: &Value) -> Vec<FileEntry> {
    let mut files = Vec::new();
    for (pointer, kind) in [
        ("/artifacts/actions_request", "actions-request"),
        ("/artifacts/actions_result", "actions-result"),
    ] {
        let Some(path) = metadata.pointer(pointer).and_then(Value::as_str) else {
            continue;
        };
        files.push(file_entry(run_dir, PathBuf::from(path), kind));
    }

    if let Some(captures) = metadata
        .pointer("/artifacts/captures")
        .and_then(Value::as_array)
    {
        for capture in captures {
            let Some(path) = capture.get("path").and_then(Value::as_str) else {
                continue;
            };
            files.push(file_entry(run_dir, PathBuf::from(path), "capture"));
        }
    }
    if let Some(extracts) = metadata
        .pointer("/artifacts/extracts")
        .and_then(Value::as_array)
    {
        for extract in extracts {
            let Some(path) = extract.get("path").and_then(Value::as_str) else {
                continue;
            };
            files.push(file_entry(run_dir, PathBuf::from(path), "extract"));
        }
    }
    files
}

fn file_entry(run_dir: &Path, path: PathBuf, kind: &'static str) -> FileEntry {
    let ownership = if path.starts_with(run_dir) {
        "internal"
    } else {
        "external"
    };
    FileEntry {
        size_bytes: file_size(&path),
        exists: path.exists(),
        path,
        kind,
        ownership,
    }
}

fn redacted_metadata(mut value: Value, sensitive: bool) -> Value {
    if sensitive {
        if let Some(url) = value.get("url").and_then(Value::as_str) {
            value["url"] = Value::String(redact_url(url, true));
        }
        if let Some(url) = value.get("final_url").and_then(Value::as_str) {
            value["final_url"] = Value::String(redact_url(url, true));
        }
    }
    value
}

fn redact_url(value: &str, sensitive: bool) -> String {
    if !sensitive {
        return value.to_string();
    }
    let Ok(mut url) = url::Url::parse(value) else {
        return "<redacted>".to_string();
    };
    if url.query().is_some() {
        url.set_query(Some("<redacted>"));
    }
    if url.fragment().is_some() {
        url.set_fragment(Some("<redacted>"));
    }
    url.to_string()
}

fn artifact_summary(runs: &[RunSummary]) -> ArtifactSummary {
    ArtifactSummary {
        count: runs.len(),
        total_size_bytes: runs.iter().map(|run| run.size_bytes).sum(),
        sensitive_count: runs.iter().filter(|run| run.sensitive).count(),
        malformed_count: runs.iter().filter(|run| !run.metadata.valid).count(),
    }
}

fn status_label(ok: Option<bool>, valid_metadata: bool) -> &'static str {
    if !valid_metadata {
        "invalid"
    } else if ok == Some(true) {
        "ok"
    } else {
        "error"
    }
}

fn format_age(age_seconds: u64) -> String {
    if age_seconds >= 24 * 60 * 60 {
        format!("{}d", age_seconds / (24 * 60 * 60))
    } else if age_seconds >= 60 * 60 {
        format!("{}h", age_seconds / (60 * 60))
    } else if age_seconds >= 60 {
        format!("{}m", age_seconds / 60)
    } else {
        format!("{age_seconds}s")
    }
}

fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut size = 0u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            size = size.saturating_add(dir_size(&entry.path())?);
        } else {
            size = size.saturating_add(metadata.len());
        }
    }
    Ok(size)
}

fn file_size(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|metadata| metadata.len())
}

fn collect_files(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files(&path)?);
        } else {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn system_time_secs(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn usage_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::UsageError, message)
}

fn elapsed_timing(started: Instant) -> TimingMs {
    TimingMs {
        total: started.elapsed().as_millis(),
    }
}

#[derive(Debug, Clone, Serialize)]
struct ArtifactList {
    runs: Vec<RunSummary>,
    summary: ArtifactSummary,
}

#[derive(Debug, Clone, Serialize)]
struct ArtifactSummary {
    count: usize,
    total_size_bytes: u64,
    sensitive_count: usize,
    malformed_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct RunSummary {
    run_id: String,
    run_dir: PathBuf,
    modified_unix: u64,
    age_seconds: u64,
    size_bytes: u64,
    ok: Option<bool>,
    sensitive: bool,
    content_format: Option<String>,
    extractor: Option<String>,
    source: SourceSummary,
    content: ContentSummary,
    metadata: MetadataSummary,
}

#[derive(Debug, Clone, Serialize)]
struct SourceSummary {
    url: Option<String>,
    final_url: Option<String>,
    host: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ContentSummary {
    path: PathBuf,
    ownership: &'static str,
    exists: bool,
}

#[derive(Debug, Clone, Serialize)]
struct MetadataSummary {
    path: PathBuf,
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct InspectResult {
    run_id: String,
    run_dir: PathBuf,
    size_bytes: u64,
    ok: Option<bool>,
    sensitive: bool,
    source: SourceSummary,
    metadata: InspectMetadata,
    files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct InspectMetadata {
    path: PathBuf,
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct FileEntry {
    path: PathBuf,
    kind: &'static str,
    ownership: &'static str,
    exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteResult {
    run_id: String,
    deleted: bool,
    deleted_paths: Vec<PathBuf>,
    preserved_external_paths: Vec<PathBuf>,
    freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
struct PruneResult {
    dry_run: bool,
    would_delete: Vec<String>,
    deleted: Vec<String>,
    would_free_bytes: u64,
    preserved_external_paths: Vec<PathBuf>,
}
