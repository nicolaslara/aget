use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::render_children;
use crate::extraction::markdown::inline::inline_markdown_from_children;
use crate::extraction::markdown::writer::MarkdownWriter;

pub(super) fn has_google_doc_text_style(node: NodeRef<'_, Node>) -> bool {
    ElementRef::wrap(node)
        .and_then(|element| element.value().attr("style"))
        .is_some_and(|style| {
            let style = style.to_ascii_lowercase();
            google_doc_style_is_bold(&style)
                || style.contains("font-style:italic")
                || google_doc_style_is_fixed_width(&style)
        })
}

pub(super) fn render_google_doc_styled_inline(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
    let Some(style) = ElementRef::wrap(node)
        .and_then(|element| element.value().attr("style"))
        .map(|style| style.to_ascii_lowercase())
    else {
        render_children(node, writer);
        return;
    };
    let inner = inline_markdown_from_children(node, writer);
    if inner.is_empty() {
        return;
    }
    let mut rendered = inner;
    if google_doc_style_is_fixed_width(&style) {
        rendered = format!("`{rendered}`");
    }
    if google_doc_style_is_bold(&style) {
        rendered = format!("{}{rendered}{}", writer.strong_mark, writer.strong_mark);
    }
    if style.contains("font-style:italic") {
        rendered = format!("{}{rendered}{}", writer.emphasis_mark, writer.emphasis_mark);
    }
    writer.push_inline(&rendered);
}

fn google_doc_style_is_bold(style: &str) -> bool {
    style.contains("font-weight:bold")
        || style.contains("font-weight: bold")
        || style.contains("font-weight:700")
        || style.contains("font-weight: 700")
}

fn google_doc_style_is_fixed_width(style: &str) -> bool {
    style.contains("font-family:consolas")
        || style.contains("font-family: consolas")
        || style.contains("font-family:courier new")
        || style.contains("font-family: courier new")
}

pub(super) fn google_doc_list_is_ordered(node: NodeRef<'_, Node>, fallback: bool) -> bool {
    let Some(list_style_type) = ElementRef::wrap(node)
        .and_then(|element| element.value().attr("style"))
        .and_then(google_doc_list_style_type)
    else {
        return fallback;
    };
    !matches!(
        list_style_type.as_str(),
        "disc" | "circle" | "square" | "none"
    )
}

fn google_doc_list_style_type(style: &str) -> Option<String> {
    style
        .split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .find_map(|(name, value)| {
            name.trim()
                .eq_ignore_ascii_case("list-style-type")
                .then(|| value.trim().to_ascii_lowercase())
        })
        .filter(|value| !value.is_empty())
}
