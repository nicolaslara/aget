use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use crate::extraction::markdown::normalize::escape_markdown_link_target;
use crate::extraction::markdown::writer::MarkdownWriter;

pub(in crate::extraction::markdown) fn render_image(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
    if let Some(markdown) = image_markdown(node, writer) {
        writer.push_inline(&markdown);
    }
}

pub(super) fn image_markdown(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) -> Option<String> {
    if writer.ignore_images {
        return None;
    }
    let element = ElementRef::wrap(node)?;
    let src = element.attr("src")?;
    if src.trim().is_empty() {
        return None;
    }
    let alt = writer.image_alt(element.attr("alt")).to_string();
    if writer.images_as_html || (writer.images_with_size && image_has_size(element)) {
        return Some(image_html(element, src, &alt));
    }
    if writer.images_to_alt {
        return Some(escape_markdown_link_target(&alt));
    }
    if !writer.inline_links {
        let reference = writer.reference_link(src, "");
        return Some(format!(
            "![{}][{reference}]",
            escape_markdown_link_target(&alt)
        ));
    }
    Some(format!(
        "![{}]({})",
        escape_markdown_link_target(&alt),
        escape_markdown_link_target(&writer.resolve_url(src))
    ))
}

fn image_has_size(element: ElementRef<'_>) -> bool {
    element.attr("width").is_some() || element.attr("height").is_some()
}

fn image_html(element: ElementRef<'_>, src: &str, alt: &str) -> String {
    let mut output = format!("<img src='{src}' ");
    if let Some(width) = element.attr("width").filter(|value| !value.is_empty()) {
        output.push_str(&format!("width='{width}' "));
    }
    if let Some(height) = element.attr("height").filter(|value| !value.is_empty()) {
        output.push_str(&format!("height='{height}' "));
    }
    if !alt.is_empty() {
        output.push_str(&format!("alt='{alt}' "));
    }
    output.push_str("/>");
    output
}
