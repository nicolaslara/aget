use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::super::normalize::escape_table_cell;
use super::super::{inline_markdown_from_children, MarkdownWriter};

pub(super) struct MarkdownTableRow {
    pub(super) cells: Vec<String>,
    pub(super) has_header_cells: bool,
}

pub(super) fn table_caption_text(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> String {
    if let Some(element) = ElementRef::wrap(node) {
        if element.value().name() == "caption" {
            return inline_markdown_from_children(node, writer);
        }
    }

    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        let caption = table_caption_text(current, writer);
        if !caption.is_empty() {
            return caption;
        }
        child = next;
    }
    String::new()
}

pub(super) fn collect_table_rows(
    node: NodeRef<'_, Node>,
    writer: &MarkdownWriter,
    rows: &mut Vec<MarkdownTableRow>,
) {
    if let Some(element) = ElementRef::wrap(node) {
        if element.value().name() == "tr" {
            rows.push(markdown_table_row(node, writer));
            return;
        }
    }

    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        collect_table_rows(current, writer, rows);
        child = next;
    }
}

fn markdown_table_row(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> MarkdownTableRow {
    let mut cells = Vec::new();
    let mut has_header_cells = false;
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        if let Some(element) = ElementRef::wrap(current) {
            match element.value().name() {
                "th" => {
                    has_header_cells = true;
                    cells.push(table_cell_text(current, writer));
                }
                "td" => cells.push(table_cell_text(current, writer)),
                _ => {}
            }
        }
        child = next;
    }
    MarkdownTableRow {
        cells,
        has_header_cells,
    }
}

fn table_cell_text(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> String {
    escape_table_cell(&inline_markdown_from_children(node, writer))
}

pub(super) fn normalize_table_row(cells: &[String], columns: usize) -> Vec<String> {
    let mut row = cells.to_vec();
    row.resize(columns, String::new());
    row
}

pub(super) fn markdown_table_line(cells: &[String]) -> String {
    format!("| {} |", cells.join(" | "))
}
