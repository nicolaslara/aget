use std::collections::BTreeMap;

use serde_json::Value;

mod backend;
mod direct;
mod finalization;
mod sessions;

pub(super) use self::backend::{run_primary_extractor, try_session_fallback};
pub(crate) use self::direct::finish_direct_extraction;
pub(super) use self::finalization::{finalize_error, finalize_success};
pub(super) use self::sessions::load_selected_sessions;

#[derive(Debug)]
pub(super) struct SuccessfulExtraction {
    pub(super) final_url: String,
    pub(super) content: String,
    pub(super) page_metadata: BTreeMap<String, Value>,
    pub(super) warnings: Vec<String>,
    pub(super) extractor: String,
    pub(super) source_bytes: Option<usize>,
    pub(super) screenshot_png: Option<Vec<u8>>,
}
