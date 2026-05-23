use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::super::{inline_markdown_from_children, MarkdownWriter};
use super::rows::table_caption_text;

pub(super) fn render_ignored_table(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let caption = table_caption_text(node, writer);
    let rows = ignored_table_rows(node, writer);
    if caption.is_empty() && rows.is_empty() {
        return;
    }

    writer.ensure_blank_line();
    if !caption.is_empty() {
        writer.output.push_str(&caption);
        writer.output.push('\n');
    }
    for row in rows {
        writer.output.push_str(&row);
        writer.output.push('\n');
    }
    writer.ensure_blank_line();
}

fn ignored_table_rows(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> Vec<String> {
    let mut rows = Vec::new();
    collect_ignored_table_rows(node, writer, &mut rows);
    rows
}

fn collect_ignored_table_rows(
    node: NodeRef<'_, Node>,
    writer: &MarkdownWriter,
    rows: &mut Vec<String>,
) {
    if let Some(element) = ElementRef::wrap(node) {
        if element.value().name() == "tr" {
            let row = ignored_table_row_text(node, writer);
            if !row.is_empty() {
                rows.push(row);
            }
            return;
        }
    }

    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        collect_ignored_table_rows(current, writer, rows);
        child = next;
    }
}

fn ignored_table_row_text(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> String {
    let mut cells = Vec::new();
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        if let Some(element) = ElementRef::wrap(current) {
            if matches!(element.value().name(), "th" | "td") {
                let cell = inline_markdown_from_children(current, writer);
                if !cell.is_empty() {
                    cells.push(cell);
                }
            }
        }
        child = next;
    }
    cells.join(" ")
}
