use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::image::image_markdown;
use crate::extraction::markdown::normalize::{
    escape_link_text, escape_link_title, escape_markdown_link_target, is_absolute_http_url,
};
use crate::extraction::markdown::render_children;
use crate::extraction::markdown::writer::MarkdownWriter;

pub(in crate::extraction::markdown) fn render_link(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
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
    if writer.ignore_mailto_links && href.starts_with("mailto:") {
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
            if writer.inline_links {
                writer.output.push_str(&format!(
                    "[{}]({}{})",
                    escape_link_text(&label),
                    markdown_link_target(writer, href),
                    title
                ));
            } else {
                let reference = writer.reference_link(href, &reference_title(title.trim()));
                writer
                    .output
                    .push_str(&format!("[{}][{reference}]", escape_link_text(&label)));
            }
            writer.ensure_blank_line();
            return;
        }
    }

    if let Some(image_node) = single_image_child(node) {
        if let Some(image) = image_markdown(image_node, writer) {
            if writer.images_to_alt
                && writer.use_automatic_links
                && image == href
                && is_absolute_http_url(href)
            {
                writer.push_inline(&format!("<{}>", href));
                return;
            }
            if writer.inline_links {
                writer.push_inline(&format!(
                    "[{}]({}{})",
                    image,
                    markdown_link_target(writer, href),
                    title
                ));
            } else {
                let reference = writer.reference_link(href, &reference_title(title.trim()));
                writer.push_inline(&format!("[{}][{reference}]", image));
            }
            return;
        }
    }

    let label = link_label_markdown_from_children(node, writer);
    if writer.use_automatic_links
        && !label.is_empty()
        && label == href
        && is_absolute_http_url(href)
    {
        writer.push_inline(&format!("<{}>", href));
        return;
    }
    if writer.inline_links {
        writer.push_inline(&format!(
            "[{}]({}{})",
            escape_link_text(&label),
            markdown_link_target(writer, href),
            title
        ));
    } else {
        let reference = writer.reference_link(href, &reference_title(title.trim()));
        writer.push_inline(&format!("[{}][{reference}]", escape_link_text(&label)));
    }
}

fn reference_title(title: &str) -> String {
    title
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(title)
        .to_string()
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

fn link_label_markdown_from_children(node: NodeRef<'_, Node>, parent: &MarkdownWriter) -> String {
    let mut writer = parent.link_child();
    render_children(node, &mut writer);
    crate::extraction::markdown::normalize::normalize_inline_markdown(&writer.output)
}
