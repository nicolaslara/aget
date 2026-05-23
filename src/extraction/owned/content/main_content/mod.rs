use ego_tree::NodeId;
use scraper::{ElementRef, Html};

use crate::error::AgetError;
use crate::extraction::html_clean::parse_css_selector;

mod labels;
mod scoring;
mod text;

use labels::has_crawl4ai_negative_class_id_label;
use scoring::score_main_content_candidate;

pub(super) fn default_main_content_element_id(
    document: &Html,
    word_count_threshold: usize,
) -> Result<NodeId, AgetError> {
    if let Some(id) = best_main_content_candidate_id(document, word_count_threshold)? {
        return Ok(id);
    }
    if let Some(body) = first_selected_element(document, "body")? {
        return Ok(body.id());
    }
    Ok(document.root_element().id())
}

fn best_main_content_candidate_id(
    document: &Html,
    word_count_threshold: usize,
) -> Result<Option<NodeId>, AgetError> {
    let mut best = None;
    for selector in ["main", r#"[role="main"]"#, "article", "section", "div"] {
        let selector = parse_css_selector(selector)?;
        for element in document.select(&selector) {
            if is_inside_crawl4ai_pruning_excluded_tag(element) {
                continue;
            }
            if has_crawl4ai_negative_class_id_label(element) {
                continue;
            }
            let score = score_main_content_candidate(element, word_count_threshold)?;
            if score <= 0 {
                continue;
            }
            let replace = best
                .as_ref()
                .map(|(best_score, _)| score > *best_score)
                .unwrap_or(true);
            if replace {
                best = Some((score, element.id()));
            }
        }
    }
    Ok(best.map(|(_, id)| id))
}

fn is_inside_crawl4ai_pruning_excluded_tag(element: ElementRef<'_>) -> bool {
    element.ancestors().any(|ancestor| {
        ElementRef::wrap(ancestor)
            .map(|ancestor| {
                matches!(
                    ancestor.value().name(),
                    "nav"
                        | "footer"
                        | "header"
                        | "aside"
                        | "script"
                        | "style"
                        | "form"
                        | "iframe"
                        | "noscript"
                )
            })
            .unwrap_or(false)
    })
}

fn first_selected_element<'a>(
    document: &'a Html,
    raw_selector: &str,
) -> Result<Option<ElementRef<'a>>, AgetError> {
    let selector = parse_css_selector(raw_selector)?;
    Ok(document.select(&selector).next())
}
