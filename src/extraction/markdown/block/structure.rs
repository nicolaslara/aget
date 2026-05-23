use ego_tree::NodeRef;
use scraper::Node;

use super::super::inline::inline_markdown_from_children;
use super::super::render_children;
use super::super::writer::MarkdownWriter;

pub(in crate::extraction::markdown) fn render_heading(
    node: NodeRef<'_, Node>,
    tag: &str,
    writer: &mut MarkdownWriter,
) {
    let level = tag
        .strip_prefix('h')
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
        .clamp(1, 6);
    let text = inline_markdown_from_children(node, writer);
    if text.is_empty() {
        return;
    }
    writer.ensure_blank_line();
    writer.output.push_str(&"#".repeat(level));
    writer.output.push(' ');
    writer.output.push_str(&text);
    writer.ensure_blank_line();
}

pub(in crate::extraction::markdown) fn render_block(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
    writer.ensure_blank_line();
    render_children(node, writer);
    writer.ensure_blank_line();
    writer.append_paragraph_reference_link_definitions();
}

pub(in crate::extraction::markdown) fn render_horizontal_rule(writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    writer.output.push_str("* * *");
    writer.ensure_blank_line();
}

pub(in crate::extraction::markdown) fn is_structural_block(tag: &str) -> bool {
    matches!(
        tag,
        "article" | "aside" | "footer" | "header" | "main" | "nav" | "section"
    )
}
