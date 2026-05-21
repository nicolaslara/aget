use ego_tree::NodeId;
use html5ever::tree_builder::TreeSink;
use scraper::{ElementRef, Html, HtmlTreeSink, Node, Selector};

use crate::error::AgetError;

use super::extraction_failed;

const CRAWL4AI_IMPORTANT_ATTRS: &[&str] = &[
    "src", "href", "alt", "title", "width", "height", "class", "id",
];
const CRAWL4AI_EMPTY_ELEMENT_BYPASS_TAGS: &[&str] = &[
    "a", "img", "br", "hr", "input", "meta", "link", "source", "track", "wbr", "tr", "td", "th",
];
const CRAWL4AI_OVERLAY_SELECTORS: &[&str] = &[
    r#"button[class*="close" i]"#,
    r#"button[class*="dismiss" i]"#,
    r#"button[aria-label*="close" i]"#,
    r#"button[title*="close" i]"#,
    r#"a[class*="close" i]"#,
    r#"span[class*="close" i]"#,
    r#"[class*="cookie-banner" i]"#,
    r#"[id*="cookie-banner" i]"#,
    r#"[class*="cookie-consent" i]"#,
    r#"[id*="cookie-consent" i]"#,
    r#"[class*="newsletter" i]"#,
    r#"[class*="subscribe" i]"#,
    r#"[class*="popup" i]"#,
    r#"[class*="modal" i]"#,
    r#"[class*="overlay" i]"#,
    r#"[class*="dialog" i]"#,
    r#"[role="dialog"]"#,
    r#"[role="alertdialog"]"#,
];

pub(super) fn prune_owned_unwanted_attributes(mut document: Html) -> Html {
    for node in document.tree.values_mut() {
        if let Node::Element(element) = node {
            element
                .attrs
                .retain(|(name, _)| is_crawl4ai_important_attr(name.local.as_ref()));
        }
    }
    document
}

pub(super) fn clean_owned_base64_image_sources(mut document: Html) -> Html {
    for node in document.tree.values_mut() {
        let Node::Element(element) = node else {
            continue;
        };
        if element.name.local.as_ref() != "img" {
            continue;
        }
        for (name, value) in &mut element.attrs {
            if name.local.as_ref() == "src" && is_base64_image_src(value.as_ref()) {
                value.clear();
            }
        }
    }
    document
}

pub(super) fn remove_owned_comments(document: Html) -> Html {
    let node_ids = document
        .tree
        .nodes()
        .filter_map(|node| matches!(node.value(), Node::Comment(_)).then_some(node.id()))
        .collect::<Vec<_>>();
    let sink = HtmlTreeSink::new(document);
    for id in node_ids {
        sink.remove_from_parent(&id);
    }
    sink.finish()
}

pub(super) fn remove_owned_empty_elements(
    document: Html,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
    word_count_threshold: usize,
) -> Html {
    let node_ids = document
        .tree
        .nodes()
        .filter_map(|node| ElementRef::wrap(node).map(|element| element.id()))
        .collect::<Vec<_>>();
    let sink = HtmlTreeSink::new(document);
    for id in node_ids.into_iter().rev() {
        let should_remove = {
            let document = sink.0.borrow();
            should_remove_owned_empty_element(
                &document,
                id,
                root_ids,
                target_ids,
                word_count_threshold,
            )
        };
        if should_remove {
            sink.remove_from_parent(&id);
        }
    }
    sink.finish()
}

fn should_remove_owned_empty_element(
    document: &Html,
    id: NodeId,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
    word_count_threshold: usize,
) -> bool {
    if root_ids.contains(&id) || target_ids.contains(&id) {
        return false;
    }
    let Some(element) = document.tree.get(id).and_then(ElementRef::wrap) else {
        return false;
    };
    if element.parent().is_none() {
        return false;
    }
    let tag = element.value().name();
    if CRAWL4AI_EMPTY_ELEMENT_BYPASS_TAGS.contains(&tag) || is_descendant_of_code_block(element) {
        return false;
    }
    if element.child_elements().next().is_some() {
        return false;
    }
    element.text().flat_map(str::split_whitespace).count() < word_count_threshold
}

fn is_descendant_of_code_block(element: ElementRef<'_>) -> bool {
    element.ancestors().any(|ancestor| {
        ElementRef::wrap(ancestor)
            .map(|ancestor| matches!(ancestor.value().name(), "pre" | "code"))
            .unwrap_or(false)
    })
}

fn is_base64_image_src(src: &str) -> bool {
    let Some(after_prefix) = src.strip_prefix("data:image/") else {
        return false;
    };
    let Some((mime_type, after_mime_type)) = after_prefix.split_once(';') else {
        return false;
    };
    !mime_type.is_empty() && after_mime_type.starts_with("base64,")
}

fn is_crawl4ai_important_attr(name: &str) -> bool {
    CRAWL4AI_IMPORTANT_ATTRS.contains(&name)
}

pub(super) fn remove_owned_excluded_tags(
    document: Html,
    tags: &[String],
) -> Result<Html, AgetError> {
    if tags.is_empty() {
        return Ok(document);
    }
    remove_selected_elements(document, &tags.join(","))
}

pub(super) fn remove_owned_overlay_elements(document: Html) -> Result<Html, AgetError> {
    remove_selected_elements(document, &CRAWL4AI_OVERLAY_SELECTORS.join(","))
}

pub(super) fn remove_selected_elements(
    document: Html,
    selector_list: &str,
) -> Result<Html, AgetError> {
    let Ok(selector) = parse_css_selector(selector_list) else {
        return Ok(document);
    };
    let node_ids = document
        .select(&selector)
        .map(|element| element.id())
        .collect::<Vec<_>>();
    let tree = HtmlTreeSink::new(document);
    for id in node_ids {
        tree.remove_from_parent(&id);
    }
    Ok(tree.finish())
}

pub(super) fn parse_css_selector(raw: &str) -> Result<Selector, AgetError> {
    let selector = raw.trim().strip_prefix("css:").unwrap_or(raw.trim()).trim();
    if selector.is_empty() {
        return Err(extraction_failed(format!(
            "owned extractor received an empty CSS selector from '{raw}'"
        )));
    }
    Selector::parse(selector).map_err(|error| {
        extraction_failed(format!(
            "owned extractor could not parse CSS selector '{raw}': {error:?}"
        ))
    })
}
