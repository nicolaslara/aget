use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::inline::{inline_markdown_from_children, raw_text_from_node};
use super::normalize::{normalize_markdown, trim_trailing_horizontal_space};
use super::writer::MarkdownWriter;
use super::{render_children, render_node};

pub(super) fn render_heading(node: NodeRef<'_, Node>, tag: &str, writer: &mut MarkdownWriter) {
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

pub(super) fn render_block(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    render_children(node, writer);
    writer.ensure_blank_line();
    writer.append_paragraph_reference_link_definitions();
}

pub(super) fn render_horizontal_rule(writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    writer.output.push_str("* * *");
    writer.ensure_blank_line();
}

pub(super) fn render_list(node: NodeRef<'_, Node>, ordered: bool, writer: &mut MarkdownWriter) {
    let is_nested = writer.list_depth > 0;
    if is_nested {
        trim_trailing_horizontal_space(&mut writer.output);
        if !writer.output.is_empty() && !writer.output.ends_with('\n') {
            writer.output.push('\n');
        }
    } else {
        writer.ensure_blank_line();
    }
    writer.list_depth += 1;
    let item_depth = writer.list_depth;
    let mut child = node.first_child();
    let mut index = 1usize;
    while let Some(current) = child {
        let next = current.next_sibling();
        if let Some(element) = ElementRef::wrap(current) {
            if element.value().name() == "li" {
                trim_trailing_horizontal_space(&mut writer.output);
                if !writer.output.is_empty() && !writer.output.ends_with('\n') {
                    writer.output.push('\n');
                }
                writer.output.push_str(&list_item_indent(
                    current,
                    item_depth,
                    writer.google_doc,
                    writer.google_list_indent,
                ));
                if ordered {
                    writer.output.push_str(&format!("{index}. "));
                } else {
                    writer.output.push_str(&writer.ul_item_mark);
                    writer.output.push(' ');
                }
                render_children(current, writer);
                trim_trailing_horizontal_space(&mut writer.output);
                if !writer.output.ends_with('\n') {
                    writer.output.push('\n');
                }
                index += 1;
            } else {
                render_node(current, writer);
            }
        } else {
            render_node(current, writer);
        }
        child = next;
    }
    writer.list_depth = writer.list_depth.saturating_sub(1);
    if is_nested {
        trim_trailing_horizontal_space(&mut writer.output);
        if !writer.output.ends_with('\n') {
            writer.output.push('\n');
        }
    } else {
        writer.ensure_blank_line();
    }
}

fn list_item_indent(
    node: NodeRef<'_, Node>,
    depth: usize,
    google_doc: bool,
    google_list_indent: usize,
) -> String {
    if google_doc {
        return "  ".repeat(google_doc_list_nest_count(node, google_list_indent));
    }
    "  ".repeat(depth.saturating_sub(1))
}

fn google_doc_list_nest_count(node: NodeRef<'_, Node>, google_list_indent: usize) -> usize {
    let Some(style) = ElementRef::wrap(node).and_then(|element| element.value().attr("style"))
    else {
        return 0;
    };
    style
        .split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .find_map(|(name, value)| {
            (name.trim().eq_ignore_ascii_case("margin-left"))
                .then(|| parse_google_doc_margin_left(value.trim(), google_list_indent))
                .flatten()
        })
        .unwrap_or(0)
}

fn parse_google_doc_margin_left(value: &str, google_list_indent: usize) -> Option<usize> {
    let raw_pixels = value.trim().strip_suffix("px")?.trim();
    let pixels = raw_pixels.parse::<usize>().ok()?;
    Some(pixels / google_list_indent.max(1))
}

pub(super) fn render_definition_list(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        if let Some(element) = ElementRef::wrap(current) {
            match element.value().name() {
                "dt" => {
                    let term = inline_markdown_from_children(current, writer);
                    if !term.is_empty() {
                        if !writer.output.ends_with('\n') {
                            writer.output.push('\n');
                        }
                        writer.output.push_str(&term);
                        writer.output.push('\n');
                    }
                }
                "dd" => {
                    let definition = inline_markdown_from_children(current, writer);
                    if !definition.is_empty() {
                        writer.output.push_str("    ");
                        writer.output.push_str(&definition);
                        writer.output.push('\n');
                    }
                }
                _ => render_node(current, writer),
            }
        } else {
            render_node(current, writer);
        }
        child = next;
    }
    writer.ensure_blank_line();
}

pub(super) fn render_code_block(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let text = raw_text_from_node(node);
    if text.trim().is_empty() {
        return;
    }
    writer.ensure_blank_line();
    writer.output.push_str("```\n");
    writer.output.push_str(text.trim_matches('\n'));
    writer.output.push_str("\n```");
    writer.ensure_blank_line();
}

pub(super) fn render_blockquote(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
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

pub(super) fn is_structural_block(tag: &str) -> bool {
    matches!(
        tag,
        "article" | "aside" | "footer" | "header" | "main" | "nav" | "section"
    )
}
