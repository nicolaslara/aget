use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use crate::extraction::markdown::normalize::normalize_inline_markdown;
use crate::extraction::markdown::render_children;
use crate::extraction::markdown::writer::MarkdownWriter;

pub(in crate::extraction::markdown) fn render_abbreviation(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
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

pub(in crate::extraction::markdown) fn inline_markdown_from_children(
    node: NodeRef<'_, Node>,
    parent: &MarkdownWriter,
) -> String {
    let mut writer = parent.child();
    render_children(node, &mut writer);
    normalize_inline_markdown(&writer.output)
}

pub(in crate::extraction::markdown) fn inline_text_from_node(node: NodeRef<'_, Node>) -> String {
    normalize_inline_markdown(&raw_text_from_node(node))
}

pub(in crate::extraction::markdown) fn raw_text_from_node(node: NodeRef<'_, Node>) -> String {
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
