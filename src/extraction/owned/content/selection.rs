use ego_tree::NodeId;
use scraper::{ElementRef, Html};

use crate::error::AgetError;
use crate::extraction::extraction_failed;
use crate::extraction::html_clean::parse_css_selector;

pub(super) fn is_body_or_document_root(
    document: &Html,
    root_ids: &[NodeId],
) -> Result<bool, AgetError> {
    let [root_id] = root_ids else {
        return Ok(false);
    };
    let root = element_by_id(document, *root_id)?;
    Ok(matches!(root.value().name(), "body" | "html"))
}

pub(super) fn collect_target_owned_element_ids(
    document: &Html,
    source_ids: &[NodeId],
    raw_selectors: &[String],
) -> Result<Vec<NodeId>, AgetError> {
    let mut ids = Vec::new();
    for source_id in source_ids {
        let source = element_by_id(document, *source_id)?;
        for raw_selector in raw_selectors {
            let selector = parse_css_selector(raw_selector)?;
            ids.extend(source.select(&selector).map(|element| element.id()));
        }
    }
    Ok(ids)
}

pub(super) fn element_by_id(document: &Html, id: NodeId) -> Result<ElementRef<'_>, AgetError> {
    document
        .tree
        .get(id)
        .and_then(ElementRef::wrap)
        .ok_or_else(|| extraction_failed("owned extractor lost a selected HTML element"))
}
