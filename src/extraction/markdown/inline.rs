use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::normalize::{
    escape_link_text, escape_link_title, escape_markdown_link_target, is_absolute_http_url,
    normalize_inline_markdown,
};
use super::render_children;
use super::writer::MarkdownWriter;

pub(super) fn render_link(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    if writer.ignore_links {
        render_children(node, writer);
        return;
    }
    let Some(element) = ElementRef::wrap(node) else {
        render_children(node, writer);
        return;
    };
    let Some(href) = element.attr("href") else {
        render_children(node, writer);
        return;
    };
    if writer.skip_internal_links && href.starts_with('#') {
        render_children(node, writer);
        return;
    }
    if href.starts_with("mailto:") {
        render_children(node, writer);
        return;
    }
    let title = element.attr("title").unwrap_or("").trim();
    let title = if title.is_empty() {
        String::new()
    } else {
        format!(" \"{}\"", escape_link_title(title))
    };

    if let Some((level, heading_node)) = single_heading_child(node) {
        let label = link_label_markdown_from_children(heading_node, writer);
        if !label.is_empty() {
            writer.ensure_blank_line();
            writer.output.push_str(&"#".repeat(level));
            writer.output.push(' ');
            writer.output.push_str(&format!(
                "[{}]({}{})",
                escape_link_text(&label),
                markdown_link_target(writer, href),
                title
            ));
            writer.ensure_blank_line();
            return;
        }
    }

    if let Some(image_node) = single_image_child(node) {
        if let Some(image) = image_markdown(image_node, writer) {
            writer.push_inline(&format!(
                "[{}]({}{})",
                image,
                markdown_link_target(writer, href),
                title
            ));
            return;
        }
    }

    let label = link_label_markdown_from_children(node, writer);
    if !label.is_empty() && label == href && is_absolute_http_url(href) {
        writer.push_inline(&format!("<{}>", href));
        return;
    }
    writer.push_inline(&format!(
        "[{}]({}{})",
        escape_link_text(&label),
        markdown_link_target(writer, href),
        title
    ));
}

fn markdown_link_target(writer: &MarkdownWriter, href: &str) -> String {
    let resolved = writer.resolve_url(href);
    if writer.protect_links {
        format!("<{}>", resolved.replace('[', "\\[").replace(']', "\\]"))
    } else {
        escape_markdown_link_target(&resolved)
    }
}

fn single_image_child(node: NodeRef<'_, Node>) -> Option<NodeRef<'_, Node>> {
    let mut image = None;
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        match current.value() {
            Node::Text(text) if text.trim().is_empty() => {}
            Node::Element(element) if element.name() == "img" && image.is_none() => {
                image = Some(current);
            }
            _ => return None,
        }
        child = next;
    }
    image
}

fn single_heading_child(node: NodeRef<'_, Node>) -> Option<(usize, NodeRef<'_, Node>)> {
    let mut heading = None;
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        match current.value() {
            Node::Text(text) if text.trim().is_empty() => {}
            Node::Element(element) if heading.is_none() => {
                let Some(level) = heading_level(element.name()) else {
                    return None;
                };
                heading = Some((level, current));
            }
            _ => return None,
        }
        child = next;
    }
    heading
}

fn heading_level(tag: &str) -> Option<usize> {
    tag.strip_prefix('h')
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|level| (1..=6).contains(level))
}

pub(super) fn render_abbreviation(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let text = inline_markdown_from_children(node, writer);
    if text.is_empty() {
        return;
    }
    if let Some(title) = ElementRef::wrap(node)
        .and_then(|element| element.attr("title"))
        .map(normalize_inline_markdown)
    {
        writer.record_abbreviation(text.clone(), title);
    }
    writer.push_inline(&text);
}

pub(super) fn render_image(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    if let Some(markdown) = image_markdown(node, writer) {
        writer.push_inline(&markdown);
    }
}

fn image_markdown(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> Option<String> {
    if writer.ignore_images {
        return None;
    }
    let element = ElementRef::wrap(node)?;
    let src = element.attr("src")?;
    if src.trim().is_empty() {
        return None;
    }
    let alt = element.attr("alt").unwrap_or("");
    Some(format!(
        "![{}]({})",
        escape_markdown_link_target(alt),
        escape_markdown_link_target(&writer.resolve_url(src))
    ))
}

pub(super) fn inline_markdown_from_children(
    node: NodeRef<'_, Node>,
    parent: &MarkdownWriter,
) -> String {
    let mut writer = parent.child();
    render_children(node, &mut writer);
    normalize_inline_markdown(&writer.output)
}

fn link_label_markdown_from_children(node: NodeRef<'_, Node>, parent: &MarkdownWriter) -> String {
    let mut writer = parent.link_child();
    render_children(node, &mut writer);
    normalize_inline_markdown(&writer.output)
}

pub(super) fn inline_text_from_node(node: NodeRef<'_, Node>) -> String {
    normalize_inline_markdown(&raw_text_from_node(node))
}

pub(super) fn raw_text_from_node(node: NodeRef<'_, Node>) -> String {
    let mut output = String::new();
    collect_raw_text(node, &mut output);
    output
}

fn collect_raw_text(node: NodeRef<'_, Node>, output: &mut String) {
    if let Node::Text(text) = node.value() {
        output.push_str(text);
    }
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        collect_raw_text(current, output);
        child = next;
    }
}
