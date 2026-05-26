use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::output::OutputOptions;
use crate::cli::{CachePolicy, ExtractorOption, OutputFormat};
use crate::error::AgetError;
use crate::session::{PlaywrightState, Session, SessionStore};

#[derive(Clone)]
pub struct GetOptions {
    pub url: String,
    pub sessions: Vec<String>,
    pub output: Option<PathBuf>,
    pub home: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub content_format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for_selector: Option<String>,
    pub max_chars: Option<usize>,
    pub cache_policy: CachePolicy,
    pub cache_ttl: Duration,
    pub debug: DebugCaptureOptions,
    pub backend_options: Vec<ExtractorOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSuccess {
    pub ok: bool,
    pub url: String,
    pub final_url: String,
    pub content_format: String,
    pub extractor: String,
    pub content: String,
    pub page_metadata: BTreeMap<String, Value>,
    pub artifacts: Artifacts,
    pub sessions: Vec<String>,
    pub sensitive: bool,
    pub warnings: Vec<String>,
    pub timing_ms: TimingMs,
    pub limits: Limits,
    pub cache: CacheMetadata,
    pub usage: UsageMetrics,
    pub output_options: OutputOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifacts {
    pub content: String,
    pub metadata: String,
    #[serde(default, skip_serializing_if = "DebugArtifacts::is_empty")]
    pub debug: DebugArtifacts,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebugCaptureOptions {
    pub screenshot: bool,
    pub trace: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebugArtifacts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<DebugArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<DebugArtifact>,
}

impl DebugArtifacts {
    pub fn is_empty(&self) -> bool {
        self.screenshot.is_none() && self.trace.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebugArtifact {
    pub path: String,
    pub media_type: String,
    pub sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMs {
    pub total: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub status: CacheStatus,
    pub policy: CachePolicy,
    pub eligible: bool,
    pub key: Option<String>,
    pub ttl_seconds: u64,
    pub age_seconds: Option<u64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheStatus {
    Hit,
    Miss,
    Stale,
    Refresh,
    Disabled,
    Ineligible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub fetched_bytes: Option<usize>,
    pub content_bytes: usize,
    pub estimated_tokens: usize,
    pub estimated_tokens_saved: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub max_chars: Option<usize>,
    pub truncated: bool,
    pub truncated_by: Option<String>,
    pub content_chars_before_truncation: usize,
    pub content_chars_after_truncation: usize,
}

pub struct ExtractorRequest<'a> {
    pub url: &'a str,
    // Structured session state lets in-process extractors avoid re-reading the
    // temp Playwright file that exists for command-backed compatibility.
    pub state: &'a PlaywrightState,
    pub state_path: &'a Path,
    pub content_path: &'a Path,
    pub metadata_path: &'a Path,
    pub options: &'a GetOptions,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorBackendResult {
    pub ok: bool,
    #[serde(default)]
    pub final_url: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub page_metadata: BTreeMap<String, Value>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub source_bytes: Option<usize>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub screenshot_png: Option<Vec<u8>>,
}

pub trait ExtractorBackend {
    // Extraction is the "URL plus session state to content" capability. The
    // default implementation is AgetExtractor, but the rest of the pipeline
    // should not know whether content came from an owned or compatibility backend.
    fn name(&self) -> &'static str;
    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError>;
}

pub struct BrowserFallbackRequest<'a> {
    pub tmp_dir: &'a Path,
    pub url: &'a str,
    // Keep the structured state alongside the temp file path so future browser
    // backends can load cookies/storage without coupling to command adapter I/O.
    pub state: &'a PlaywrightState,
    pub state_path: &'a Path,
    pub options: &'a GetOptions,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct BrowserFallbackResult {
    pub final_url: String,
    pub content: String,
    pub page_metadata: BTreeMap<String, Value>,
    pub warnings: Vec<String>,
    pub extractor: String,
    pub source_bytes: Option<usize>,
    pub screenshot_png: Option<Vec<u8>>,
}

pub trait BrowserFallbackBackend {
    // Fallback browser extraction handles authenticated pages when the primary
    // extractor cannot consume the composed session state directly.
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError>;
}

pub trait ExtractionSessionStore {
    // `aget get` only needs scoped session lookup plus a local home for private
    // run artifacts. Keeping this separate lets `AgetWith` use non-filesystem
    // stores without changing the extraction pipeline.
    fn home(&self) -> &Path;
    fn load(&self, name: &str) -> io::Result<Session>;
}

impl ExtractionSessionStore for SessionStore {
    fn home(&self) -> &Path {
        self.home()
    }

    fn load(&self, name: &str) -> io::Result<Session> {
        self.load(name)
    }
}
