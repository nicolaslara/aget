use ego_tree::NodeRef;
use scraper::Node;

use super::super::inline::raw_text_from_node;
use super::super::writer::MarkdownWriter;

pub(in crate::extraction::markdown) fn render_code_block(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
    let text = if writer.handle_code_in_pre {
        pre_text_with_code_markers(node)
    } else {
        raw_text_from_node(node)
    };
    if text.trim().is_empty() {
        return;
    }
    writer.ensure_blank_line();
    writer.output.push_str("```\n");
    writer.output.push_str(text.trim_matches('\n'));
    writer.output.push_str("\n```");
    writer.ensure_blank_line();
}

fn pre_text_with_code_markers(node: NodeRef<'_, Node>) -> String {
    let mut output = String::new();
    collect_pre_text_with_code_markers(node, &mut output);
    output
}

fn collect_pre_text_with_code_markers(node: NodeRef<'_, Node>, output: &mut String) {
    match node.value() {
        Node::Text(text) => output.push_str(text),
        Node::Element(element) if element.name() == "code" => {
            output.push('`');
            output.push_str(&raw_text_from_node(node));
            output.push('`');
        }
        _ => {
            let mut child = node.first_child();
            while let Some(current) = child {
                collect_pre_text_with_code_markers(current, output);
                child = current.next_sibling();
            }
        }
    }
}
