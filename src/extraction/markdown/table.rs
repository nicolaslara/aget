use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::normalize::escape_table_cell;
use super::{inline_markdown_from_children, MarkdownWriter};

struct MarkdownTableRow {
    cells: Vec<String>,
    has_header_cells: bool,
}

pub(super) fn render_table(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    if writer.ignore_tables {
        render_ignored_table(node, writer);
        return;
    }
    if writer.bypass_tables {
        render_bypassed_table(node, writer);
        return;
    }

    let caption = table_caption_text(node, writer);
    let mut rows = Vec::new();
    collect_table_rows(node, writer, &mut rows);
    let Some(max_columns) = rows.iter().map(|row| row.cells.len()).max() else {
        if !caption.is_empty() {
            writer.ensure_blank_line();
            writer.output.push_str(&caption);
            writer.ensure_blank_line();
        }
        return;
    };
    if max_columns == 0 {
        if !caption.is_empty() {
            writer.ensure_blank_line();
            writer.output.push_str(&caption);
            writer.ensure_blank_line();
        }
        return;
    }

    let header_index = rows
        .iter()
        .position(|row| row.has_header_cells)
        .unwrap_or(0);
    let header = normalize_table_row(&rows[header_index].cells, max_columns);
    let body_rows = rows
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != header_index)
        .map(|(_, row)| normalize_table_row(&row.cells, max_columns))
        .collect::<Vec<_>>();

    writer.ensure_blank_line();
    if !caption.is_empty() {
        writer.output.push_str(&caption);
        writer.ensure_blank_line();
    }
    writer.output.push_str(&markdown_table_line(&header));
    writer.output.push('\n');
    writer
        .output
        .push_str(&markdown_table_line(&vec!["---".to_string(); max_columns]));
    writer.output.push('\n');
    for row in body_rows {
        writer.output.push_str(&markdown_table_line(&row));
        writer.output.push('\n');
    }
    writer.ensure_blank_line();
}

fn render_ignored_table(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
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

fn render_bypassed_table(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
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

fn table_caption_text(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> String {
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

fn collect_table_rows(
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

fn normalize_table_row(cells: &[String], columns: usize) -> Vec<String> {
    let mut row = cells.to_vec();
    row.resize(columns, String::new());
    row
}

fn markdown_table_line(cells: &[String]) -> String {
    format!("| {} |", cells.join(" | "))
}
