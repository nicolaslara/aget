use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aget::{ErrorCode, ErrorResponse, ExtractCommand, ExtractOutput, SessionStore, TimingMs};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

pub(super) fn run_extract(
    command: ExtractCommand,
    json: bool,
    quiet: bool,
) -> Result<ExitCode, ErrorResponse> {
    let started = Instant::now();
    let input = resolve_input(&command)?;
    let specs = extraction_fields(&command)?;
    let store = SessionStore::from_env().map_err(super::io_error)?;
    let sources = read_sources(&store, &input, command.allow_private_content)?;
    let output_dir = store.home().join("runs").join(extract_run_id());
    fs::create_dir_all(&output_dir).map_err(super::io_error)?;

    let records = sources
        .iter()
        .map(|source| extract_record(source, &specs))
        .collect::<Result<Vec<_>, _>>()?;
    let sensitive_sources = records.iter().filter(|record| record.sensitive).count();
    let result = ExtractResult {
        summary: ExtractSummary {
            sources: records.len(),
            fields: specs.len(),
            sensitive_sources,
        },
        source: ExtractSource {
            kind: input.kind(),
            artifact: match &input {
                ExtractInput::Artifact(run_id) => Some(run_id.clone()),
                ExtractInput::Manifest(_) => None,
            },
            manifest: match &input {
                ExtractInput::Artifact(_) => None,
                ExtractInput::Manifest(path) => Some(path.clone()),
            },
        },
        schema: ExtractSchemaSummary {
            path: command.schema,
            fields: specs,
        },
        artifacts: ExtractArtifacts {
            json: output_dir.join("extract-results.json"),
            markdown: output_dir.join("extract-results.md"),
            metadata: output_dir.join("metadata.json"),
        },
        records,
    };
    write_outputs(&result)?;

    if json {
        super::print_success_envelope(
            "extract",
            serde_json::to_value(&result).map_err(super::io_error)?,
            Vec::new(),
            elapsed_timing(started),
        )?;
    } else if !quiet {
        match command.output {
            ExtractOutput::Json => println!(
                "{}",
                serde_json::to_string_pretty(&result).map_err(super::io_error)?
            ),
            ExtractOutput::Markdown => print!("{}", extract_markdown(&result)),
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn resolve_input(command: &ExtractCommand) -> Result<ExtractInput, ErrorResponse> {
    match (&command.artifact, &command.manifest) {
        (Some(run_id), None) => {
            validate_run_id(run_id)?;
            Ok(ExtractInput::Artifact(run_id.clone()))
        }
        (None, Some(path)) => Ok(ExtractInput::Manifest(path.clone())),
        (None, None) => Err(usage_error(
            "extract requires exactly one --artifact <run-id> or --manifest <path>",
        )),
        (Some(_), Some(_)) => Err(usage_error(
            "extract requires exactly one input; do not combine --artifact and --manifest",
        )),
    }
}

fn extraction_fields(command: &ExtractCommand) -> Result<Vec<ExtractFieldSpec>, ErrorResponse> {
    let mut fields = Vec::new();
    for field in &command.fields {
        fields.push(shortcut_field(field)?);
    }
    if let Some(path) = &command.schema {
        let schema = read_schema(path)?;
        fields.extend(schema.fields);
    }
    if fields.is_empty() {
        return Err(usage_error(
            "extract requires --schema <path> or at least one --field <name>",
        ));
    }
    for field in &fields {
        if field.name.trim().is_empty() {
            return Err(usage_error("extract schema field names must not be empty"));
        }
        match field.kind {
            ExtractFieldKind::Selector if field.selector.as_deref().unwrap_or("").is_empty() => {
                return Err(usage_error(format!(
                    "extract schema field '{}' requires selector",
                    field.name
                )));
            }
            ExtractFieldKind::Json if field.path.as_deref().unwrap_or("").is_empty() => {
                return Err(usage_error(format!(
                    "extract schema field '{}' requires path",
                    field.name
                )));
            }
            _ => {}
        }
    }
    Ok(fields)
}

fn shortcut_field(field: &str) -> Result<ExtractFieldSpec, ErrorResponse> {
    let kind = match field {
        "headings" => ExtractFieldKind::Headings,
        "links" => ExtractFieldKind::Links,
        "tables" => ExtractFieldKind::Tables,
        "definitions" => ExtractFieldKind::Definitions,
        "metadata" => ExtractFieldKind::Metadata,
        _ => {
            return Err(usage_error(format!(
                "unsupported extract --field '{field}'; expected headings, links, tables, definitions, or metadata"
            )));
        }
    };
    Ok(ExtractFieldSpec {
        name: field.to_string(),
        kind,
        selector: None,
        attribute: None,
        multiple: None,
        path: None,
    })
}

fn read_schema(path: &Path) -> Result<ExtractSchema, ErrorResponse> {
    let text = fs::read_to_string(path).map_err(super::io_error)?;
    let schema: ExtractSchema = serde_json::from_str(&text).map_err(|error| {
        usage_error(format!(
            "invalid extract schema '{}': {error}",
            path.display()
        ))
    })?;
    if schema.fields.is_empty() {
        return Err(usage_error(
            "extract schema must contain at least one field",
        ));
    }
    Ok(schema)
}

fn read_sources(
    store: &SessionStore,
    input: &ExtractInput,
    allow_private_content: bool,
) -> Result<Vec<SourcePage>, ErrorResponse> {
    match input {
        ExtractInput::Artifact(run_id) => {
            let source = read_artifact_source(store, run_id, allow_private_content)?;
            Ok(vec![source])
        }
        ExtractInput::Manifest(path) => read_manifest_sources(store, path, allow_private_content),
    }
}

fn read_artifact_source(
    store: &SessionStore,
    run_id: &str,
    allow_private_content: bool,
) -> Result<SourcePage, ErrorResponse> {
    let run_dir = store.home().join("runs").join(run_id);
    let metadata_path = run_dir.join("metadata.json");
    if !metadata_path.is_file() {
        return Err(usage_error(format!("artifact run not found: {run_id}")));
    }
    let metadata = read_json(&metadata_path)?;
    if metadata.get("ok").and_then(Value::as_bool) != Some(true)
        || metadata
            .get("command")
            .and_then(Value::as_str)
            .is_some_and(|command| command != "get")
    {
        return Err(usage_error(
            "extract --artifact requires a successful get run",
        ));
    }
    source_from_metadata(
        Some(run_id.to_string()),
        metadata,
        allow_private_content,
        Some(ContentPathPolicy {
            root: &run_dir,
            expected: None,
            outside_message: "extract --artifact refuses external caller-owned content paths",
            mismatch_message: "",
        }),
    )
}

fn read_manifest_sources(
    store: &SessionStore,
    manifest_path: &Path,
    allow_private_content: bool,
) -> Result<Vec<SourcePage>, ErrorResponse> {
    let manifest = read_json(manifest_path)?;
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let runs_dir = store.home().join("runs");
    let items = manifest
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            usage_error("extract --manifest requires an aget batch or crawl manifest")
        })?;

    let mut sources = Vec::new();
    for item in items {
        if item.get("status").and_then(Value::as_str) != Some("ok") {
            continue;
        }
        let Some(item_content_path) = item.pointer("/artifacts/content").and_then(Value::as_str)
        else {
            continue;
        };
        let metadata_path = item
            .pointer("/artifacts/metadata")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                usage_error(
                    "extract --manifest requires successful items to include artifacts.metadata",
                )
            })?;
        if !Path::new(metadata_path).is_file() {
            return Err(usage_error(format!(
                "extract --manifest item metadata not found: {metadata_path}"
            )));
        }
        ensure_path_inside(
            Path::new(metadata_path),
            &runs_dir,
            "extract --manifest requires item metadata under AGET_HOME/runs",
        )?;
        let metadata = read_json(Path::new(metadata_path))?;
        let artifact = Path::new(metadata_path)
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .filter(|name| name.starts_with("run-"))
            .map(str::to_string);
        let source = source_from_metadata(
            artifact,
            metadata,
            allow_private_content,
            Some(ContentPathPolicy {
                root: manifest_dir,
                expected: Some(Path::new(item_content_path)),
                outside_message:
                    "extract --manifest refuses content paths outside the manifest directory",
                mismatch_message:
                    "extract --manifest item content path does not match item metadata",
            }),
        )?;
        sources.push(source);
    }

    if sources.is_empty() {
        return Err(usage_error(
            "extract --manifest found no successful items with content artifacts",
        ));
    }
    Ok(sources)
}

fn source_from_metadata(
    artifact: Option<String>,
    metadata: Value,
    allow_private_content: bool,
    content_policy: Option<ContentPathPolicy<'_>>,
) -> Result<SourcePage, ErrorResponse> {
    let sensitive = metadata
        .get("sensitive")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if sensitive && !allow_private_content {
        return Err(ErrorResponse::new(
            ErrorCode::RequiresUserAction,
            "extract source artifact is sensitive; rerun with --allow-private-content to emit structured values",
        ));
    }
    let content_path = metadata
        .pointer("/artifacts/content")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| usage_error("extract source metadata does not include artifacts.content"))?;
    if let Some(policy) = content_policy {
        ensure_path_inside(&content_path, policy.root, policy.outside_message)?;
        if let Some(expected) = policy.expected {
            ensure_same_path(&content_path, expected, policy.mismatch_message)?;
        }
    }
    let content = fs::read_to_string(&content_path).map_err(super::io_error)?;
    let url = metadata
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let final_url = metadata
        .get("final_url")
        .and_then(Value::as_str)
        .unwrap_or(url.as_str())
        .to_string();
    let content_format = metadata
        .get("content_format")
        .and_then(Value::as_str)
        .unwrap_or("markdown")
        .to_string();
    Ok(SourcePage {
        artifact,
        url,
        final_url,
        sensitive,
        content_format,
        metadata,
        content_path,
        content,
    })
}

fn ensure_path_inside(
    path: &Path,
    root: &Path,
    message: &'static str,
) -> Result<(), ErrorResponse> {
    let path = path.canonicalize().map_err(super::io_error)?;
    let root = root.canonicalize().map_err(super::io_error)?;
    if !path.starts_with(root) {
        return Err(usage_error(message));
    }
    Ok(())
}

fn ensure_same_path(left: &Path, right: &Path, message: &'static str) -> Result<(), ErrorResponse> {
    let left = left.canonicalize().map_err(super::io_error)?;
    let right = right.canonicalize().map_err(super::io_error)?;
    if left != right {
        return Err(usage_error(message));
    }
    Ok(())
}

fn extract_record(
    source: &SourcePage,
    fields: &[ExtractFieldSpec],
) -> Result<ExtractRecord, ErrorResponse> {
    let mut values = BTreeMap::new();
    let mut provenance = Vec::new();
    for field in fields {
        values.insert(field.name.clone(), extract_field(source, field)?);
        provenance.push(ExtractProvenance {
            field: field.name.clone(),
            kind: field.kind,
            artifact: source.artifact.clone(),
            source_url: source.final_url.clone(),
            content_path: source.content_path.clone(),
            content_format: source.content_format.clone(),
            selector: field.selector.clone(),
            path: field.path.clone(),
        });
    }
    Ok(ExtractRecord {
        artifact: source.artifact.clone(),
        url: source.url.clone(),
        final_url: source.final_url.clone(),
        sensitive: source.sensitive,
        values,
        provenance,
    })
}

fn extract_field(source: &SourcePage, field: &ExtractFieldSpec) -> Result<Value, ErrorResponse> {
    match field.kind {
        ExtractFieldKind::Headings => Ok(headings(source)),
        ExtractFieldKind::Links => Ok(links(source)),
        ExtractFieldKind::Tables => Ok(tables(source)),
        ExtractFieldKind::Definitions => Ok(definitions(source)),
        ExtractFieldKind::Metadata => metadata_value(source, field),
        ExtractFieldKind::Selector => selector_value(source, field),
        ExtractFieldKind::Json => json_value(source, field),
    }
}

fn headings(source: &SourcePage) -> Value {
    if source.is_html() {
        let document = Html::parse_document(&source.content);
        let selector = Selector::parse("h1,h2,h3,h4,h5,h6").expect("valid heading selector");
        let values = document
            .select(&selector)
            .map(|element| {
                let level = element
                    .value()
                    .name()
                    .trim_start_matches('h')
                    .parse::<usize>()
                    .unwrap_or(0);
                json!({"level": level, "text": element_text(&element)})
            })
            .collect::<Vec<_>>();
        return json!(values);
    }

    let values = source
        .content
        .lines()
        .filter_map(markdown_heading)
        .map(|(level, text)| json!({"level": level, "text": text}))
        .collect::<Vec<_>>();
    json!(values)
}

fn links(source: &SourcePage) -> Value {
    if source.is_html() {
        return html_links(source);
    }
    markdown_links(source)
}

fn html_links(source: &SourcePage) -> Value {
    let document = Html::parse_document(&source.content);
    let selector = Selector::parse("a[href]").expect("valid link selector");
    let base = base_url(source, &document);
    let values = document
        .select(&selector)
        .filter_map(|element| {
            let href = element.value().attr("href")?;
            Some(json!({
                "text": element_text(&element),
                "url": resolve_href(base.as_ref(), href),
            }))
        })
        .collect::<Vec<_>>();
    json!(values)
}

fn markdown_links(source: &SourcePage) -> Value {
    let base = Url::parse(&source.final_url)
        .ok()
        .or_else(|| Url::parse(&source.url).ok());
    let mut values = Vec::new();
    let mut start = 0usize;
    while let Some(open) = source.content[start..].find('[') {
        let open = start + open;
        if open > 0 && source.content.as_bytes().get(open - 1) == Some(&b'!') {
            start = open + 1;
            continue;
        }
        let Some(close) = source.content[open..].find("](").map(|index| open + index) else {
            break;
        };
        let href_start = close + 2;
        let Some(href_end) = source.content[href_start..]
            .find(')')
            .map(|index| href_start + index)
        else {
            break;
        };
        let text = source.content[open + 1..close].trim();
        let href = source.content[href_start..href_end].trim();
        if !text.is_empty() && !href.is_empty() {
            values.push(json!({
                "text": text,
                "url": resolve_href(base.as_ref(), href),
            }));
        }
        start = href_end + 1;
    }
    json!(values)
}

fn tables(source: &SourcePage) -> Value {
    if source.is_html() {
        return html_tables(source);
    }
    markdown_tables(&source.content)
}

fn html_tables(source: &SourcePage) -> Value {
    let document = Html::parse_document(&source.content);
    let table_selector = Selector::parse("table").expect("valid table selector");
    let row_selector = Selector::parse("tr").expect("valid row selector");
    let cell_selector = Selector::parse("th,td").expect("valid cell selector");
    let th_selector = Selector::parse("th").expect("valid th selector");

    let mut tables = Vec::new();
    for table in document.select(&table_selector) {
        let mut rows = table
            .select(&row_selector)
            .map(|row| {
                row.select(&cell_selector)
                    .map(|cell| element_text(&cell))
                    .collect::<Vec<_>>()
            })
            .filter(|cells| !cells.is_empty())
            .collect::<Vec<_>>();
        if rows.is_empty() {
            continue;
        }
        let header_first = table
            .select(&row_selector)
            .next()
            .is_some_and(|row| row.select(&th_selector).next().is_some());
        let headers = if header_first {
            rows.remove(0)
        } else {
            (1..=rows.iter().map(Vec::len).max().unwrap_or(0))
                .map(|index| format!("column_{index}"))
                .collect::<Vec<_>>()
        };
        tables.push(table_value(headers, rows));
    }
    json!(tables)
}

fn markdown_tables(content: &str) -> Value {
    let lines = content.lines().collect::<Vec<_>>();
    let mut tables = Vec::new();
    let mut index = 0usize;
    while index + 1 < lines.len() {
        if !looks_like_table_line(lines[index]) || !is_markdown_table_separator(lines[index + 1]) {
            index += 1;
            continue;
        }
        let headers = table_cells(lines[index]);
        let mut rows = Vec::new();
        index += 2;
        while index < lines.len() && looks_like_table_line(lines[index]) {
            rows.push(table_cells(lines[index]));
            index += 1;
        }
        tables.push(table_value(headers, rows));
    }
    json!(tables)
}

fn table_value(headers: Vec<String>, rows: Vec<Vec<String>>) -> Value {
    let values = rows
        .into_iter()
        .map(|row| {
            let mut object = serde_json::Map::new();
            for (index, header) in headers.iter().enumerate() {
                object.insert(
                    header.clone(),
                    Value::String(row.get(index).cloned().unwrap_or_default()),
                );
            }
            Value::Object(object)
        })
        .collect::<Vec<_>>();
    json!({"headers": headers, "rows": values})
}

fn definitions(source: &SourcePage) -> Value {
    if source.is_html() {
        return html_definitions(source);
    }
    markdown_definitions(&source.content)
}

fn html_definitions(source: &SourcePage) -> Value {
    let document = Html::parse_document(&source.content);
    let dt_selector = Selector::parse("dt").expect("valid dt selector");
    let dd_selector = Selector::parse("dd").expect("valid dd selector");
    let terms = document
        .select(&dt_selector)
        .map(|element| element_text(&element))
        .collect::<Vec<_>>();
    let definitions = document
        .select(&dd_selector)
        .map(|element| element_text(&element))
        .collect::<Vec<_>>();
    let values = terms
        .into_iter()
        .zip(definitions)
        .map(|(term, definition)| json!({"term": term, "definition": definition}))
        .collect::<Vec<_>>();
    json!(values)
}

fn markdown_definitions(content: &str) -> Value {
    let values = content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let (term, definition) = trimmed.split_once(':')?;
            let term = term.trim();
            let definition = definition.trim();
            if term.is_empty()
                || definition.is_empty()
                || term.len() > 80
                || term.contains("://")
                || term.starts_with('#')
            {
                return None;
            }
            Some(json!({"term": term, "definition": definition}))
        })
        .collect::<Vec<_>>();
    json!(values)
}

fn metadata_value(source: &SourcePage, field: &ExtractFieldSpec) -> Result<Value, ErrorResponse> {
    Ok(match &field.path {
        Some(path) => source
            .metadata
            .pointer(&pointer_from_path(path))
            .cloned()
            .unwrap_or(Value::Null),
        None => source.metadata.clone(),
    })
}

fn selector_value(source: &SourcePage, field: &ExtractFieldSpec) -> Result<Value, ErrorResponse> {
    if !source.is_html() {
        return Err(usage_error(format!(
            "extract selector field '{}' requires an HTML source artifact; fetch with --content-format html",
            field.name
        )));
    }
    let selector_text = field.selector.as_deref().unwrap_or_default();
    let selector = Selector::parse(selector_text).map_err(|error| {
        usage_error(format!(
            "invalid selector for extract field '{}': {error}",
            field.name
        ))
    })?;
    let document = Html::parse_document(&source.content);
    let values = document
        .select(&selector)
        .map(|element| selected_attribute(&element, field.attribute.as_deref()))
        .collect::<Vec<_>>();
    if field.multiple.unwrap_or(false) {
        Ok(json!(values))
    } else {
        Ok(values
            .into_iter()
            .next()
            .map(Value::String)
            .unwrap_or(Value::Null))
    }
}

fn json_value(source: &SourcePage, field: &ExtractFieldSpec) -> Result<Value, ErrorResponse> {
    let content: Value = serde_json::from_str(&source.content).map_err(|error| {
        usage_error(format!(
            "extract json field '{}' requires JSON content: {error}",
            field.name
        ))
    })?;
    Ok(content
        .pointer(&pointer_from_path(
            field.path.as_deref().unwrap_or_default(),
        ))
        .cloned()
        .unwrap_or(Value::Null))
}

fn markdown_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let hashes = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&hashes) || trimmed.chars().nth(hashes) != Some(' ') {
        return None;
    }
    Some((hashes, trimmed[hashes..].trim().to_string()))
}

fn looks_like_table_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains('|') && table_cells(trimmed).len() >= 2
}

fn is_markdown_table_separator(line: &str) -> bool {
    let cells = table_cells(line);
    cells.len() >= 2
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            trimmed.contains('-')
                && trimmed
                    .chars()
                    .all(|character| matches!(character, '-' | ':' | ' '))
        })
}

fn table_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .map(str::to_string)
        .collect()
}

fn selected_attribute(element: &scraper::ElementRef<'_>, attribute: Option<&str>) -> String {
    match attribute.unwrap_or("text") {
        "text" => element_text(element),
        "html" => element.inner_html(),
        attribute => {
            let attribute = attribute.strip_prefix("attr:").unwrap_or(attribute);
            element
                .value()
                .attr(attribute)
                .unwrap_or_default()
                .to_string()
        }
    }
}

fn element_text(element: &scraper::ElementRef<'_>) -> String {
    element
        .text()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn base_url(source: &SourcePage, document: &Html) -> Option<Url> {
    document_base_url(document)
        .or_else(|| Url::parse(&source.final_url).ok())
        .or_else(|| Url::parse(&source.url).ok())
}

fn document_base_url(document: &Html) -> Option<Url> {
    let selector = Selector::parse("base[href]").ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("href"))
        .and_then(|href| Url::parse(href).ok())
}

fn resolve_href(base: Option<&Url>, href: &str) -> String {
    let href = href.trim();
    if href.is_empty() || href.starts_with("javascript:") {
        return String::new();
    }
    base.and_then(|base| base.join(href).ok())
        .map(|url| url.to_string())
        .unwrap_or_else(|| href.to_string())
}

fn pointer_from_path(path: &str) -> String {
    if path.starts_with('/') {
        return path.to_string();
    }
    let path = path.strip_prefix("$.").unwrap_or(path);
    format!(
        "/{}",
        path.split('.')
            .filter(|part| !part.is_empty())
            .map(|part| part.replace('~', "~0").replace('/', "~1"))
            .collect::<Vec<_>>()
            .join("/")
    )
}

fn write_outputs(result: &ExtractResult) -> Result<(), ErrorResponse> {
    let metadata = json!({
        "ok": true,
        "command": "extract",
        "url": result.records.first().map(|record| record.url.as_str()),
        "final_url": result.records.first().map(|record| record.final_url.as_str()),
        "content_format": "extract-results",
        "source": result.source,
        "artifacts": result.artifacts,
        "summary": result.summary,
        "sensitive": result.summary.sensitive_sources > 0,
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
    fs::write(&result.artifacts.markdown, extract_markdown(result)).map_err(super::io_error)
}

fn extract_markdown(result: &ExtractResult) -> String {
    let mut markdown = format!(
        "# aget extract\n\nSources: {}\nFields: {}\nSensitive sources: {}\n\n",
        result.summary.sources, result.summary.fields, result.summary.sensitive_sources
    );
    for (index, record) in result.records.iter().enumerate() {
        markdown.push_str(&format!(
            "## Source {}\n\nURL: {}\n\n```json\n{}\n```\n\n",
            index + 1,
            record.final_url,
            serde_json::to_string_pretty(&record.values).unwrap_or_else(|_| "{}".to_string())
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

fn extract_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("run-{}-{nanos}-extract", std::process::id())
}

fn usage_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::UsageError, message)
}

fn elapsed_timing(started: Instant) -> TimingMs {
    TimingMs {
        total: started.elapsed().as_millis(),
    }
}

enum ExtractInput {
    Artifact(String),
    Manifest(PathBuf),
}

impl ExtractInput {
    fn kind(&self) -> ExtractSourceKind {
        match self {
            ExtractInput::Artifact(_) => ExtractSourceKind::Artifact,
            ExtractInput::Manifest(_) => ExtractSourceKind::Manifest,
        }
    }
}

struct SourcePage {
    artifact: Option<String>,
    url: String,
    final_url: String,
    sensitive: bool,
    content_format: String,
    metadata: Value,
    content_path: PathBuf,
    content: String,
}

struct ContentPathPolicy<'a> {
    root: &'a Path,
    expected: Option<&'a Path>,
    outside_message: &'static str,
    mismatch_message: &'static str,
}

impl SourcePage {
    fn is_html(&self) -> bool {
        self.content_format == "html"
            || self.content.contains("<html")
            || self.content.contains("<table")
            || self.content.contains("<a ")
            || self.content.contains("<dl")
    }
}

#[derive(Debug, Deserialize)]
struct ExtractSchema {
    fields: Vec<ExtractFieldSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractFieldSpec {
    name: String,
    #[serde(rename = "type")]
    kind: ExtractFieldKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multiple: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExtractFieldKind {
    Headings,
    Links,
    Tables,
    Definitions,
    Metadata,
    Selector,
    Json,
}

#[derive(Debug, Serialize)]
struct ExtractResult {
    summary: ExtractSummary,
    source: ExtractSource,
    schema: ExtractSchemaSummary,
    artifacts: ExtractArtifacts,
    records: Vec<ExtractRecord>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct ExtractSummary {
    sources: usize,
    fields: usize,
    sensitive_sources: usize,
}

#[derive(Debug, Serialize)]
struct ExtractSource {
    kind: ExtractSourceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manifest: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExtractSourceKind {
    Artifact,
    Manifest,
}

#[derive(Debug, Serialize)]
struct ExtractSchemaSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,
    fields: Vec<ExtractFieldSpec>,
}

#[derive(Debug, Serialize)]
struct ExtractArtifacts {
    json: PathBuf,
    markdown: PathBuf,
    metadata: PathBuf,
}

#[derive(Debug, Serialize)]
struct ExtractRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact: Option<String>,
    url: String,
    final_url: String,
    sensitive: bool,
    values: BTreeMap<String, Value>,
    provenance: Vec<ExtractProvenance>,
}

#[derive(Debug, Serialize)]
struct ExtractProvenance {
    field: String,
    kind: ExtractFieldKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact: Option<String>,
    source_url: String,
    content_path: PathBuf,
    content_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}
