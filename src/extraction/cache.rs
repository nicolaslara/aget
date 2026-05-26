use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::cli::{CachePolicy, ExtractorOption, OutputFormat};
use crate::error::{AgetError, ErrorCode};

use super::artifacts::{create_private_dir, write_private_bytes, write_private_file};
use super::pipeline::SuccessfulExtraction;
use super::{CacheMetadata, CacheStatus, GetOptions};

const CACHE_SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone)]
pub(in crate::extraction) struct CacheContext {
    metadata: CacheMetadata,
    entry_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub(in crate::extraction) enum CacheLookup {
    Hit {
        extraction: SuccessfulExtraction,
        metadata: CacheMetadata,
    },
    Fetch(CacheContext),
}

impl CacheContext {
    pub(in crate::extraction) fn lookup(
        home: &Path,
        options: &GetOptions,
        sensitive: bool,
    ) -> Result<CacheLookup, AgetError> {
        let ttl_seconds = options.cache_ttl.as_secs();
        if options.cache_policy == CachePolicy::Off {
            return Ok(CacheLookup::Fetch(Self {
                metadata: metadata(
                    CacheStatus::Disabled,
                    options.cache_policy,
                    false,
                    None,
                    ttl_seconds,
                    None,
                    Some("cache policy is off".to_string()),
                ),
                entry_dir: None,
            }));
        }

        if sensitive {
            return Ok(CacheLookup::Fetch(Self {
                metadata: metadata(
                    CacheStatus::Ineligible,
                    options.cache_policy,
                    false,
                    None,
                    ttl_seconds,
                    None,
                    Some(
                        "session-backed requests are not reusable across cache scopes".to_string(),
                    ),
                ),
                entry_dir: None,
            }));
        }

        if options.debug.screenshot {
            return Ok(CacheLookup::Fetch(Self {
                metadata: metadata(
                    CacheStatus::Ineligible,
                    options.cache_policy,
                    false,
                    None,
                    ttl_seconds,
                    None,
                    Some("screenshot capture requires a live extraction".to_string()),
                ),
                entry_dir: None,
            }));
        }

        if !cacheable_url(&options.url) {
            return Ok(CacheLookup::Fetch(Self {
                metadata: metadata(
                    CacheStatus::Ineligible,
                    options.cache_policy,
                    false,
                    None,
                    ttl_seconds,
                    None,
                    Some("only public http(s) URL inputs are cacheable".to_string()),
                ),
                entry_dir: None,
            }));
        }

        let key = cache_key(options);
        let entry_dir = home.join("cache").join(&key);
        if options.cache_policy == CachePolicy::Refresh {
            return Ok(CacheLookup::Fetch(Self {
                metadata: metadata(
                    CacheStatus::Refresh,
                    options.cache_policy,
                    true,
                    Some(key),
                    ttl_seconds,
                    None,
                    Some("fresh fetch requested".to_string()),
                ),
                entry_dir: Some(entry_dir),
            }));
        }

        let entry = read_entry(&entry_dir)?;
        let Some((entry, content)) = entry else {
            return Ok(CacheLookup::Fetch(Self {
                metadata: metadata(
                    CacheStatus::Miss,
                    options.cache_policy,
                    true,
                    Some(key),
                    ttl_seconds,
                    None,
                    Some("cache entry not found".to_string()),
                ),
                entry_dir: Some(entry_dir),
            }));
        };

        let age_seconds = now_secs().saturating_sub(entry.created_at_secs);
        if age_seconds >= ttl_seconds {
            return Ok(CacheLookup::Fetch(Self {
                metadata: metadata(
                    CacheStatus::Stale,
                    options.cache_policy,
                    true,
                    Some(key),
                    ttl_seconds,
                    Some(age_seconds),
                    Some("cache entry is stale".to_string()),
                ),
                entry_dir: Some(entry_dir),
            }));
        }

        Ok(CacheLookup::Hit {
            extraction: SuccessfulExtraction {
                final_url: entry.final_url,
                content,
                page_metadata: entry.page_metadata,
                warnings: entry.warnings,
                extractor: entry.extractor,
                source_bytes: entry.source_bytes,
                screenshot_png: None,
            },
            metadata: metadata(
                CacheStatus::Hit,
                options.cache_policy,
                true,
                Some(key),
                ttl_seconds,
                Some(age_seconds),
                None,
            ),
        })
    }

    pub(in crate::extraction) fn metadata(&self) -> CacheMetadata {
        self.metadata.clone()
    }

    pub(in crate::extraction) fn store(
        &self,
        options: &GetOptions,
        extraction: &SuccessfulExtraction,
    ) -> Result<(), AgetError> {
        let Some(entry_dir) = &self.entry_dir else {
            return Ok(());
        };
        create_private_dir(entry_dir).map_err(io_aget_error)?;
        let content_path = entry_dir.join("content");
        write_private_bytes(&content_path, extraction.content.as_bytes()).map_err(io_aget_error)?;
        let entry = CacheEntry {
            schema_version: CACHE_SCHEMA_VERSION,
            created_at_secs: now_secs(),
            url: options.url.clone(),
            final_url: extraction.final_url.clone(),
            content_format: options.content_format,
            extractor: extraction.extractor.clone(),
            page_metadata: extraction.page_metadata.clone(),
            warnings: extraction.warnings.clone(),
            source_bytes: extraction.source_bytes,
        };
        let bytes = serde_json::to_vec_pretty(&entry).map_err(|error| AgetError::Stable {
            code: ErrorCode::IoError,
            message: error.to_string(),
        })?;
        write_private_file(&entry_dir.join("metadata.json"), &bytes).map_err(io_aget_error)
    }
}

fn metadata(
    status: CacheStatus,
    policy: CachePolicy,
    eligible: bool,
    key: Option<String>,
    ttl_seconds: u64,
    age_seconds: Option<u64>,
    reason: Option<String>,
) -> CacheMetadata {
    CacheMetadata {
        status,
        policy,
        eligible,
        key,
        ttl_seconds,
        age_seconds,
        reason,
    }
}

fn read_entry(entry_dir: &Path) -> Result<Option<(CacheEntry, String)>, AgetError> {
    let metadata_path = entry_dir.join("metadata.json");
    let content_path = entry_dir.join("content");
    if !metadata_path.exists() || !content_path.exists() {
        return Ok(None);
    }
    let text = match fs::read_to_string(&metadata_path) {
        Ok(text) => text,
        Err(_) => return Ok(None),
    };
    let entry: CacheEntry = match serde_json::from_str(&text) {
        Ok(entry) => entry,
        Err(_) => return Ok(None),
    };
    if entry.schema_version != CACHE_SCHEMA_VERSION {
        return Ok(None);
    }
    let content = match fs::read_to_string(&content_path) {
        Ok(content) => content,
        Err(_) => return Ok(None),
    };
    Ok(Some((entry, content)))
}

fn cacheable_url(url: &str) -> bool {
    Url::parse(url)
        .map(|url| matches!(url.scheme(), "http" | "https"))
        .unwrap_or(false)
}

fn cache_key(options: &GetOptions) -> String {
    let mut text = String::new();
    push_field(&mut text, "v", "1");
    push_field(&mut text, "url", &options.url);
    push_field(&mut text, "format", &options.content_format.to_string());
    push_optional(&mut text, "selector", options.selector.as_deref());
    push_optional(
        &mut text,
        "exclude_selector",
        options.exclude_selector.as_deref(),
    );
    push_optional(
        &mut text,
        "wait_for_selector",
        options.wait_for_selector.as_deref(),
    );
    let mut backend_options = options.backend_options.iter().collect::<Vec<_>>();
    backend_options
        .sort_by(|left, right| left.key.cmp(&right.key).then(left.value.cmp(&right.value)));
    for ExtractorOption { key, value } in backend_options {
        push_field(&mut text, "backend", key);
        push_field(&mut text, "value", value);
    }
    format!("{:016x}", fnv1a64(text.as_bytes()))
}

fn push_field(text: &mut String, key: &str, value: &str) {
    text.push_str(key);
    text.push('\0');
    text.push_str(value);
    text.push('\0');
}

fn push_optional(text: &mut String, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_field(text, key, value);
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    schema_version: u8,
    created_at_secs: u64,
    url: String,
    final_url: String,
    content_format: OutputFormat,
    extractor: String,
    page_metadata: BTreeMap<String, Value>,
    warnings: Vec<String>,
    source_bytes: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn cache_key_ignores_output_and_limits() {
        let mut left = options("https://example.com/a");
        let mut right = left.clone();
        left.max_chars = Some(10);
        right.output = Some(PathBuf::from("out.md"));

        assert_eq!(cache_key(&left), cache_key(&right));
    }

    #[test]
    fn cache_key_changes_with_output_shape() {
        let left = options("https://example.com/a");
        let mut right = left.clone();
        right.selector = Some("main".to_string());

        assert_ne!(cache_key(&left), cache_key(&right));
    }

    #[test]
    fn cache_key_normalizes_backend_option_order() {
        let mut left = options("https://example.com/a");
        let mut right = left.clone();
        left.backend_options = vec![
            ExtractorOption {
                key: "aget.body_width".to_string(),
                value: "80".to_string(),
            },
            ExtractorOption {
                key: "aget.ignore_links".to_string(),
                value: "true".to_string(),
            },
        ];
        right.backend_options = left.backend_options.iter().cloned().rev().collect();

        assert_eq!(cache_key(&left), cache_key(&right));
    }

    fn options(url: &str) -> GetOptions {
        GetOptions {
            url: url.to_string(),
            sessions: Vec::new(),
            output: None,
            home: None,
            timeout: None,
            content_format: OutputFormat::Markdown,
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            cache_policy: CachePolicy::Auto,
            cache_ttl: Duration::from_secs(60),
            debug: Default::default(),
            backend_options: Vec::new(),
        }
    }
}
