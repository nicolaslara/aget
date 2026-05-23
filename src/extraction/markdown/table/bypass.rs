use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::super::{inline_markdown_from_children, MarkdownWriter};

pub(super) fn render_bypassed_table(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    render_bypassed_table_node(node, writer);
    writer.ensure_blank_line();
}

fn render_bypassed_table_node(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    if let Some(element) = ElementRef::wrap(node) {
        match element.value().name() {
            "table" | "tr" => {
                let tag = element.value().name();
                writer.output.push('<');
                writer.output.push_str(tag);
                writer.output.push('>');
                render_bypassed_table_children(node, writer);
                writer.output.push_str("</");
                writer.output.push_str(tag);
                writer.output.push('>');
                return;
            }
            "td" | "th" => {
                let tag = element.value().name();
                writer.output.push('<');
                writer.output.push_str(tag);
                writer.output.push('>');
                writer
                    .output
                    .push_str(&inline_markdown_from_children(node, writer));
                writer.output.push_str("</");
                writer.output.push_str(tag);
                writer.output.push('>');
                return;
            }
            "caption" => {
                let caption = inline_markdown_from_children(node, writer);
                if !caption.is_empty() {
                    writer.output.push_str(&caption);
                    writer.output.push('\n');
                }
                return;
            }
            _ => {}
        }
    }
    render_bypassed_table_children(node, writer);
}

fn render_bypassed_table_children(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        render_bypassed_table_node(current, writer);
        child = next;
    }
}
