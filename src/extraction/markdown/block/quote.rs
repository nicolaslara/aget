use ego_tree::NodeRef;
use scraper::Node;

use super::super::normalize::normalize_markdown;
use super::super::render_children;
use super::super::writer::MarkdownWriter;

pub(in crate::extraction::markdown) fn render_blockquote(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
    let mut child_writer = writer.child();
    render_children(node, &mut child_writer);
    let quote = normalize_markdown(&child_writer.output);
    if quote.is_empty() {
        return;
    }
    writer.ensure_blank_line();
    for line in quote.lines() {
        writer.output.push('>');
        let line = line.trim();
        if !line.is_empty() {
            writer.output.push(' ');
            writer.output.push_str(line);
        }
        writer.output.push('\n');
    }
    writer.ensure_blank_line();
}
