use ego_tree::NodeRef;
use scraper::Node;

use super::MarkdownWriter;

mod bypass;
mod ignored;
mod pad;
mod rows;

use bypass::render_bypassed_table;
use ignored::render_ignored_table;
pub(super) use pad::pad_markdown_tables;
use rows::{collect_table_rows, markdown_table_line, normalize_table_row, table_caption_text};

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
