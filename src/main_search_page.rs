use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aget::{ErrorCode, ErrorResponse, SearchPageCommand, SearchPageOutput, SessionStore, TimingMs};
use serde::Serialize;
use serde_json::Value;

const MAX_RESULTS_LIMIT: usize = 50;
const MAX_CONTEXT_CHARS: usize = 2000;

pub(super) fn run_search_page(
    command: SearchPageCommand,
    json: bool,
    quiet: bool,
) -> Result<ExitCode, ErrorResponse> {
    let started = Instant::now();
    validate_command(&command)?;

    let store = SessionStore::from_env().map_err(super::io_error)?;
    validate_run_id(&command.artifact)?;
    let run_dir = store.home().join("runs").join(&command.artifact);
    let source = read_source(&run_dir, &command)?;
    let output_dir = store.home().join("runs").join(search_run_id());
    fs::create_dir_all(&output_dir).map_err(super::io_error)?;

    let mut matches = search_content(&source.content, &command.query, command.context_chars);
    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then(left.char_start.cmp(&right.char_start))
    });
    matches.truncate(command.max_results);

    let result = SearchPageResult {
        summary: SearchPageSummary {
            query: command.query,
            matches: matches.len(),
            max_results: command.max_results,
        },
        source: SearchPageSource {
            artifact: command.artifact,
            url: source.url,
            final_url: source.final_url,
            sensitive: source.sensitive,
        },
        artifacts: SearchPageArtifacts {
            json: output_dir.join("search-results.json"),
            markdown: output_dir.join("search-results.md"),
            metadata: output_dir.join("metadata.json"),
        },
        matches,
    };
    write_outputs(&result)?;

    if json {
        super::print_success_envelope(
            "search-page",
            serde_json::to_value(&result).map_err(super::io_error)?,
            Vec::new(),
            elapsed_timing(started),
        )?;
    } else if !quiet {
        match command.output {
            SearchPageOutput::Markdown => print!("{}", search_markdown(&result)),
            SearchPageOutput::Json => println!(
                "{}",
                serde_json::to_string_pretty(&result).map_err(super::io_error)?
            ),
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn validate_command(command: &SearchPageCommand) -> Result<(), ErrorResponse> {
    if command.query.trim().is_empty() {
        return Err(usage_error("search-page --query must not be empty"));
    }
    if command.max_results == 0 || command.max_results > MAX_RESULTS_LIMIT {
        return Err(usage_error(format!(
            "search-page --max-results must be between 1 and {MAX_RESULTS_LIMIT}"
        )));
    }
    if command.context_chars == 0 || command.context_chars > MAX_CONTEXT_CHARS {
        return Err(usage_error(format!(
            "search-page --context-chars must be between 1 and {MAX_CONTEXT_CHARS}"
        )));
    }
    Ok(())
}

fn read_source(
    run_dir: &Path,
    command: &SearchPageCommand,
) -> Result<SourceArtifact, ErrorResponse> {
    let metadata_path = run_dir.join("metadata.json");
    if !metadata_path.is_file() {
        return Err(usage_error(format!(
            "artifact run not found: {}",
            command.artifact
        )));
    }
    let metadata = read_json(&metadata_path)?;
    if metadata.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(usage_error(
            "search-page --artifact requires a successful get run",
        ));
    }
    let sensitive = metadata
        .get("sensitive")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if sensitive && !command.allow_private_content {
        return Err(ErrorResponse::new(
            ErrorCode::RequiresUserAction,
            "search-page artifact is sensitive; rerun with --allow-private-content to emit snippets",
        ));
    }
    let content_path = metadata
        .pointer("/artifacts/content")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| run_dir.join("content.md"));
    let content = fs::read_to_string(&content_path).map_err(super::io_error)?;
    Ok(SourceArtifact {
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
        sensitive,
        content,
    })
}

fn search_content(content: &str, query: &str, context_chars: usize) -> Vec<SearchPageMatch> {
    let query_terms = query_terms(query);
    let query_lower = query.to_lowercase();
    sections(content)
        .into_iter()
        .filter_map(|section| {
            let score = score_section(&section, &query_terms, &query_lower);
            if score == 0 {
                return None;
            }
            let match_char = first_match_char(section.text, &query_terms, &query_lower);
            Some(SearchPageMatch {
                section_id: section.id,
                heading: section.heading,
                score,
                snippet: snippet(section.text, match_char, context_chars),
                char_start: section.char_start,
                char_end: section.char_end,
            })
        })
        .collect()
}

fn score_section(section: &Section<'_>, query_terms: &[String], query_lower: &str) -> usize {
    let heading_lower = section.heading.to_lowercase();
    let text_lower = section.text.to_lowercase();
    let mut score = 0usize;
    if !query_lower.is_empty() && heading_lower.contains(query_lower) {
        score += 120;
    }
    if !query_lower.is_empty() && text_lower.contains(query_lower) {
        score += 60;
    }
    for term in query_terms {
        if heading_lower.contains(term) {
            score += 30;
        }
        score += text_lower.matches(term).take(8).count() * 6;
        score += section
            .text
            .lines()
            .filter(|line| line_contains_structured_match(line, term))
            .take(4)
            .count()
            * 4;
    }
    score
}

fn line_contains_structured_match(line: &str, term: &str) -> bool {
    let trimmed = line.trim_start();
    let structured = trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.contains('|')
        || trimmed.contains('[');
    structured && trimmed.to_lowercase().contains(term)
}

fn first_match_char(text: &str, terms: &[String], query_lower: &str) -> usize {
    let text_lower = text.to_lowercase();
    let mut best = if query_lower.is_empty() {
        None
    } else {
        text_lower.find(query_lower)
    };
    for term in terms {
        if let Some(index) = text_lower.find(term) {
            best = Some(best.map_or(index, |current| current.min(index)));
        }
    }
    best.map(|byte| text_lower[..byte].chars().count())
        .unwrap_or(0)
}

fn query_terms(query: &str) -> Vec<String> {
    let mut terms = query
        .split(|character: char| !character.is_alphanumeric())
        .map(str::trim)
        .filter(|term| term.chars().count() > 1)
        .map(|term| term.to_lowercase())
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}

fn sections(content: &str) -> Vec<Section<'_>> {
    let mut starts = content
        .match_indices('\n')
        .map(|(index, _)| index + 1)
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    starts.sort_unstable();
    starts.dedup();

    let mut heading_starts = starts
        .into_iter()
        .filter(|start| {
            content[*start..]
                .lines()
                .next()
                .is_some_and(is_markdown_heading)
        })
        .collect::<Vec<_>>();
    if heading_starts.first().copied() != Some(0) {
        heading_starts.insert(0, 0);
    }
    heading_starts.push(content.len());

    let mut sections = Vec::new();
    for index in 0..heading_starts.len().saturating_sub(1) {
        let start = heading_starts[index];
        let end = heading_starts[index + 1];
        if start >= end {
            continue;
        }
        let text = &content[start..end];
        let heading = text
            .lines()
            .next()
            .filter(|line| is_markdown_heading(line))
            .map(clean_heading)
            .unwrap_or_else(|| "Document".to_string());
        sections.push(Section {
            id: format!("section-{}", index + 1),
            heading,
            text,
            char_start: content[..start].chars().count(),
            char_end: content[..end].chars().count(),
        });
    }
    sections
}

fn is_markdown_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    let hashes = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    (1..=6).contains(&hashes) && trimmed.chars().nth(hashes) == Some(' ')
}

fn clean_heading(line: &str) -> String {
    line.trim_start_matches('#').trim().to_string()
}

fn snippet(text: &str, match_char: usize, context_chars: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return String::new();
    }
    let half = context_chars / 2;
    let start = match_char.saturating_sub(half);
    let end = (match_char + half).min(chars.len());
    let mut output = String::new();
    if start > 0 {
        output.push_str("...");
    }
    output.extend(chars[start..end].iter());
    if end < chars.len() {
        output.push_str("...");
    }
    collapse_whitespace(&output)
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn write_outputs(result: &SearchPageResult) -> Result<(), ErrorResponse> {
    let metadata = serde_json::json!({
        "ok": true,
        "command": "search-page",
        "url": result.source.url,
        "final_url": result.source.final_url,
        "sensitive": result.source.sensitive,
        "content_format": "search-results",
        "extractor": "aget-search-page",
        "source": result.source,
        "artifacts": result.artifacts,
        "summary": result.summary,
    });
    fs::write(
        &result.artifacts.metadata,
        serde_json::to_vec_pretty(&metadata).map_err(super::io_error)?,
    )
    .map_err(super::io_error)?;
    fs::write(
        &result.artifacts.json,
        serde_json::to_vec_pretty(result).map_err(super::io_error)?,
    )
    .map_err(super::io_error)?;
    fs::write(&result.artifacts.markdown, search_markdown(result)).map_err(super::io_error)
}

fn search_markdown(result: &SearchPageResult) -> String {
    let mut markdown = format!(
        "# aget search-page\n\nSource: {}\nQuery: {}\nMatches: {}\n\n",
        result.source.final_url, result.summary.query, result.summary.matches
    );
    for matched in &result.matches {
        markdown.push_str(&format!(
            "## {} ({})\n\n{}\n\n",
            matched.heading, matched.section_id, matched.snippet
        ));
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

fn search_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("run-{}-{nanos}-search", std::process::id())
}

fn usage_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::UsageError, message)
}

fn elapsed_timing(started: Instant) -> TimingMs {
    TimingMs {
        total: started.elapsed().as_millis(),
    }
}

struct SourceArtifact {
    url: String,
    final_url: String,
    sensitive: bool,
    content: String,
}

struct Section<'a> {
    id: String,
    heading: String,
    text: &'a str,
    char_start: usize,
    char_end: usize,
}

#[derive(Debug, Serialize)]
struct SearchPageResult {
    summary: SearchPageSummary,
    source: SearchPageSource,
    artifacts: SearchPageArtifacts,
    matches: Vec<SearchPageMatch>,
}

#[derive(Debug, Serialize)]
struct SearchPageSummary {
    query: String,
    matches: usize,
    max_results: usize,
}

#[derive(Debug, Serialize)]
struct SearchPageSource {
    artifact: String,
    url: String,
    final_url: String,
    sensitive: bool,
}

#[derive(Debug, Serialize)]
struct SearchPageArtifacts {
    json: PathBuf,
    markdown: PathBuf,
    metadata: PathBuf,
}

#[derive(Debug, Serialize)]
struct SearchPageMatch {
    section_id: String,
    heading: String,
    score: usize,
    snippet: String,
    char_start: usize,
    char_end: usize,
}
