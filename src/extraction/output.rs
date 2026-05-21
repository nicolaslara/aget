use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{GetOptions, Limits};
use crate::cli::OutputFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputOptions {
    pub content_format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for_selector: Option<String>,
    pub backend_options: BTreeMap<String, String>,
}

pub(super) struct LimitApplication {
    pub(super) content: String,
    pub(super) metadata: Limits,
}

pub(super) fn apply_limits(content: String, max_chars: Option<usize>) -> LimitApplication {
    let before = content.chars().count();
    let (content, truncated) = match max_chars {
        Some(max_chars) if before > max_chars => (content.chars().take(max_chars).collect(), true),
        _ => (content, false),
    };
    let after = content.chars().count();

    LimitApplication {
        content,
        metadata: Limits {
            max_chars,
            truncated,
            truncated_by: truncated.then(|| "max_chars".to_string()),
            content_chars_before_truncation: before,
            content_chars_after_truncation: after,
        },
    }
}

pub(super) fn output_options(options: &GetOptions) -> OutputOptions {
    let backend_options = options
        .backend_options
        .iter()
        .map(|option| (option.key.clone(), option.value.clone()))
        .collect();

    OutputOptions {
        content_format: options.content_format,
        selector: options.selector.clone(),
        exclude_selector: options.exclude_selector.clone(),
        wait_for_selector: options.wait_for_selector.clone(),
        backend_options,
    }
}
