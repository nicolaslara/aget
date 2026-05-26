use std::collections::BTreeMap;
use std::fs;
use std::time::Instant;

use crate::error::{AgetError, ErrorCode};
use crate::session::SessionStore;

use super::finalization::{disabled_cache_metadata, finalize_success};
use super::SuccessfulExtraction;
use crate::extraction::output::output_options;
use crate::extraction::{create_private_dir, io_aget_error, run_id, GetOptions, GetSuccess};

pub(crate) fn finish_direct_extraction(
    mut options: GetOptions,
    final_url: String,
    content: String,
    warnings: Vec<String>,
    extractor: impl Into<String>,
    sensitive: bool,
    source_bytes: Option<usize>,
    started: Instant,
) -> Result<GetSuccess, AgetError> {
    if options.home.is_none() {
        let store = SessionStore::from_env().map_err(io_aget_error)?;
        options.home = Some(store.home().to_path_buf());
    }
    let home = options.home.clone().ok_or_else(|| AgetError::Stable {
        code: ErrorCode::IoError,
        message: "direct extraction requires an aget home directory".to_string(),
    })?;
    let run_dir = home.join("runs").join(run_id());
    create_private_dir(&run_dir).map_err(io_aget_error)?;
    let content_path = options
        .output
        .clone()
        .unwrap_or_else(|| run_dir.join("content.md"));
    if let Some(parent) = content_path.parent() {
        fs::create_dir_all(parent).map_err(io_aget_error)?;
    }
    let metadata_path = run_dir.join("metadata.json");
    let output_options = output_options(&options);
    finalize_success(
        &options,
        &content_path,
        &metadata_path,
        Vec::new(),
        sensitive,
        output_options,
        disabled_cache_metadata(&options),
        SuccessfulExtraction {
            final_url,
            content,
            page_metadata: BTreeMap::new(),
            warnings,
            extractor: extractor.into(),
            source_bytes,
        },
        started,
    )
}
