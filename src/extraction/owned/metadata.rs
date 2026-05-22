use std::collections::BTreeMap;

use scraper::{ElementRef, Html};
use serde_json::Value;

use crate::error::AgetError;
use crate::extraction::html_clean::parse_css_selector;

pub(super) fn extract_page_metadata(document: &Html) -> Result<BTreeMap<String, Value>, AgetError> {
    let mut metadata = BTreeMap::new();
    let title = match title_from_head(document)? {
        Some(title) => Some(title),
        None => prefixed_meta_value(document, "property", "og:title")?.or(prefixed_meta_value(
            document,
            "name",
            "twitter:title",
        )?),
    };
    metadata.insert("title".to_string(), optional_string_value(title));
    for (key, selector) in [
        ("description", r#"meta[name="description"]"#),
        ("keywords", r#"meta[name="keywords"]"#),
        ("author", r#"meta[name="author"]"#),
    ] {
        metadata.insert(
            key.to_string(),
            optional_string_value(first_meta_content(document, selector)?),
        );
    }
    collect_prefixed_meta(document, "property", "og:", &mut metadata)?;
    collect_prefixed_meta(document, "name", "twitter:", &mut metadata)?;
    collect_prefixed_meta(document, "property", "article:", &mut metadata)?;
    Ok(metadata)
}

fn title_from_head(document: &Html) -> Result<Option<String>, AgetError> {
    let selector = parse_css_selector("head title")?;
    Ok(document
        .select(&selector)
        .next()
        .map(element_text)
        .filter(|title| !title.is_empty()))
}

fn first_meta_content(document: &Html, raw_selector: &str) -> Result<Option<String>, AgetError> {
    let selector = parse_css_selector(raw_selector)?;
    Ok(document
        .select(&selector)
        .next()
        .and_then(meta_content)
        .filter(|content| !content.is_empty()))
}

fn prefixed_meta_value(
    document: &Html,
    attribute: &str,
    key: &str,
) -> Result<Option<String>, AgetError> {
    let raw_selector = format!(r#"meta[{attribute}="{key}"]"#);
    first_meta_content(document, &raw_selector)
}

fn collect_prefixed_meta(
    document: &Html,
    attribute: &str,
    prefix: &str,
    metadata: &mut BTreeMap<String, Value>,
) -> Result<(), AgetError> {
    let selector = parse_css_selector(&format!("meta[{attribute}]"))?;
    for element in document.select(&selector) {
        let Some(key) = element.attr(attribute).map(str::trim) else {
            continue;
        };
        if !key.starts_with(prefix) {
            continue;
        }
        let Some(content) = meta_content(element).filter(|content| !content.is_empty()) else {
            continue;
        };
        metadata.insert(key.to_string(), Value::String(content));
    }
    Ok(())
}

fn element_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}

fn meta_content(element: ElementRef<'_>) -> Option<String> {
    element
        .attr("content")
        .map(|content| content.trim().to_string())
}

fn optional_string_value(value: Option<String>) -> Value {
    value.map(Value::String).unwrap_or(Value::Null)
}
