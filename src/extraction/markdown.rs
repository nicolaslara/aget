use std::cell::RefCell;
use std::rc::Rc;

use ego_tree::NodeRef;
use scraper::{ElementRef, Node};
use url::Url;

pub(super) fn element_to_markdown(
    element: ElementRef<'_>,
    base_url: &str,
    only_text: bool,
) -> String {
    let mut writer = MarkdownWriter::new(base_url, only_text);
    render_node(*element, &mut writer);
    writer.append_abbreviation_definitions();
    normalize_markdown(&writer.output)
}

struct MarkdownWriter {
    output: String,
    base_url: Option<Url>,
    only_text: bool,
    list_depth: usize,
    abbreviations: Rc<RefCell<Vec<(String, String)>>>,
}

impl MarkdownWriter {
    fn new(base_url: &str, only_text: bool) -> Self {
        Self {
            output: String::new(),
            base_url: Url::parse(base_url).ok(),
            only_text,
            list_depth: 0,
            abbreviations: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn child(&self) -> Self {
        Self {
            output: String::new(),
            base_url: self.base_url.clone(),
            only_text: self.only_text,
            list_depth: self.list_depth,
            abbreviations: Rc::clone(&self.abbreviations),
        }
    }

    fn resolve_url(&self, raw: &str) -> String {
        self.base_url
            .as_ref()
            .map(|base| resolve_markdown_url(base.as_str(), raw))
            .unwrap_or_else(|| raw.to_string())
    }

    fn push_text(&mut self, text: &str) {
        let mut text = normalize_inline_markdown(text);
        text = escape_markdown_text_backslashes(&text);
        if is_markdown_line_start(&self.output) {
            text = escape_markdown_line_start(&text);
        }
        self.push_inline(&text);
    }

    fn push_inline(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if needs_space_before_inline(&self.output) && !starts_with_closing_punctuation(text) {
            self.output.push(' ');
        }
        self.output.push_str(text);
    }

    fn ensure_blank_line(&mut self) {
        trim_trailing_horizontal_space(&mut self.output);
        if self.output.is_empty() {
            return;
        }
        match trailing_newline_count(&self.output) {
            0 => self.output.push_str("\n\n"),
            1 => self.output.push('\n'),
            _ => {}
        }
    }

    fn record_abbreviation(&mut self, text: String, title: String) {
        if text.is_empty() || title.is_empty() {
            return;
        }
        let mut abbreviations = self.abbreviations.borrow_mut();
        if let Some((_, existing_title)) = abbreviations
            .iter_mut()
            .find(|(existing_text, _)| existing_text == &text)
        {
            *existing_title = title;
        } else {
            abbreviations.push((text, title));
        }
    }

    fn append_abbreviation_definitions(&mut self) {
        let abbreviations = self.abbreviations.borrow().clone();
        if abbreviations.is_empty() {
            return;
        }
        self.ensure_blank_line();
        for (text, title) in abbreviations {
            self.output.push_str("  *[");
            self.output.push_str(&text);
            self.output.push_str("]: ");
            self.output.push_str(&title);
            self.output.push('\n');
        }
    }
}

fn render_node(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    match node.value() {
        Node::Text(text) => writer.push_text(text),
        Node::Element(element) => render_element(node, element.name(), writer),
        _ => render_children(node, writer),
    }
}

fn render_element(node: NodeRef<'_, Node>, tag: &str, writer: &mut MarkdownWriter) {
    if writer.only_text && is_only_text_eligible_tag(tag) {
        writer.push_text(&raw_text_from_node(node));
        return;
    }

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => render_heading(node, tag, writer),
        "p" => render_block(node, writer),
        "br" => writer.output.push_str("  \n"),
        "hr" => render_horizontal_rule(writer),
        "ul" => render_list(node, false, writer),
        "ol" => render_list(node, true, writer),
        "li" => render_block(node, writer),
        "dl" => render_definition_list(node, writer),
        "table" => render_table(node, writer),
        "pre" => render_code_block(node, writer),
        "code" | "kbd" | "tt" => writer.push_inline(&format!("`{}`", inline_text_from_node(node))),
        "address" | "details" | "figcaption" | "figure" | "summary" => render_block(node, writer),
        "strong" | "b" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("**{inner}**"));
        }
        "em" | "i" | "u" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("_{inner}_"));
        }
        "del" | "strike" | "s" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("~~{inner}~~"));
        }
        "q" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("\"{inner}\""));
        }
        "a" => render_link(node, writer),
        "abbr" => render_abbreviation(node, writer),
        "img" => render_image(node, writer),
        "blockquote" => render_blockquote(node, writer),
        "article" | "aside" | "body" | "div" | "footer" | "header" | "html" | "main" | "nav"
        | "section" => {
            render_children(node, writer);
            if is_structural_block(tag) {
                writer.ensure_blank_line();
            }
        }
        _ => render_children(node, writer),
    }
}

fn is_only_text_eligible_tag(tag: &str) -> bool {
    matches!(
        tag,
        "abbr"
            | "b"
            | "cite"
            | "code"
            | "del"
            | "dfn"
            | "em"
            | "i"
            | "ins"
            | "kbd"
            | "mark"
            | "q"
            | "s"
            | "small"
            | "span"
            | "strike"
            | "strong"
            | "sub"
            | "sup"
            | "tt"
            | "time"
            | "u"
            | "var"
    )
}

fn render_children(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        render_node(current, writer);
        child = next;
    }
}

fn render_heading(node: NodeRef<'_, Node>, tag: &str, writer: &mut MarkdownWriter) {
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

fn render_block(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    render_children(node, writer);
    writer.ensure_blank_line();
}

fn render_horizontal_rule(writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    writer.output.push_str("* * *");
    writer.ensure_blank_line();
}

fn render_list(node: NodeRef<'_, Node>, ordered: bool, writer: &mut MarkdownWriter) {
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
                writer.output.push_str(&list_item_indent(item_depth));
                if ordered {
                    writer.output.push_str(&format!("{index}. "));
                } else {
                    writer.output.push_str("* ");
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

fn list_item_indent(depth: usize) -> String {
    "  ".repeat(depth.saturating_sub(1))
}

fn render_definition_list(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
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

fn render_code_block(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
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

struct MarkdownTableRow {
    cells: Vec<String>,
    has_header_cells: bool,
}

fn render_table(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
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

fn render_link(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let Some(element) = ElementRef::wrap(node) else {
        render_children(node, writer);
        return;
    };
    let Some(href) = element.attr("href") else {
        render_children(node, writer);
        return;
    };
    if href.starts_with("mailto:") || href.starts_with('#') {
        render_children(node, writer);
        return;
    }
    let title = element.attr("title").unwrap_or("").trim();
    let title = if title.is_empty() {
        String::new()
    } else {
        format!(" \"{}\"", escape_link_title(title))
    };

    if let Some(image_node) = single_image_child(node) {
        if let Some(image) = image_markdown(image_node, writer) {
            writer.push_inline(&format!(
                "[{}]({}{})",
                image,
                escape_markdown_link_target(&writer.resolve_url(href)),
                title
            ));
            return;
        }
    }

    let label = inline_markdown_from_children(node, writer);
    if title.is_empty() && !label.is_empty() && label == href && is_absolute_http_url(href) {
        writer.push_inline(&format!("<{}>", href));
        return;
    }
    writer.push_inline(&format!(
        "[{}]({}{})",
        escape_link_text(&label),
        escape_markdown_link_target(&writer.resolve_url(href)),
        title
    ));
}

fn single_image_child(node: NodeRef<'_, Node>) -> Option<NodeRef<'_, Node>> {
    let mut image = None;
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        match current.value() {
            Node::Text(text) if text.trim().is_empty() => {}
            Node::Element(element) if element.name() == "img" && image.is_none() => {
                image = Some(current);
            }
            _ => return None,
        }
        child = next;
    }
    image
}

fn render_abbreviation(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
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

fn render_image(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    if let Some(markdown) = image_markdown(node, writer) {
        writer.push_inline(&markdown);
    }
}

fn image_markdown(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> Option<String> {
    let Some(element) = ElementRef::wrap(node) else {
        return None;
    };
    let Some(src) = element.attr("src") else {
        return None;
    };
    if src.trim().is_empty() {
        return None;
    }
    let alt = element.attr("alt").unwrap_or("");
    Some(format!(
        "![{}]({})",
        escape_markdown_link_target(alt),
        escape_markdown_link_target(&writer.resolve_url(src))
    ))
}

fn render_blockquote(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
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

fn inline_markdown_from_children(node: NodeRef<'_, Node>, parent: &MarkdownWriter) -> String {
    let mut writer = parent.child();
    render_children(node, &mut writer);
    normalize_inline_markdown(&writer.output)
}

fn inline_text_from_node(node: NodeRef<'_, Node>) -> String {
    normalize_inline_markdown(&raw_text_from_node(node))
}

fn raw_text_from_node(node: NodeRef<'_, Node>) -> String {
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

pub(super) fn normalize_markdown(markdown: &str) -> String {
    let mut output = String::new();
    let mut blank_lines = 0usize;
    for line in markdown.lines().map(normalize_markdown_line_end) {
        if line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 && !output.is_empty() {
                output.push('\n');
            }
            continue;
        }
        blank_lines = 0;
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&line);
        output.push('\n');
    }
    output.trim().to_string()
}

fn normalize_markdown_line_end(line: &str) -> String {
    let without_tabs = line.trim_end_matches('\t');
    if without_tabs.ends_with("  ") {
        let without_spaces = without_tabs.trim_end_matches(' ');
        if !without_spaces.is_empty() {
            return format!("{without_spaces}  ");
        }
    }
    line.trim_end().to_string()
}

fn normalize_inline_markdown(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn needs_space_before_inline(output: &str) -> bool {
    output
        .chars()
        .last()
        .is_some_and(|character| !character.is_whitespace())
}

fn starts_with_closing_punctuation(text: &str) -> bool {
    if text.starts_with("![") {
        return false;
    }
    text.chars()
        .next()
        .is_some_and(|character| matches!(character, '.' | ',' | ':' | ';' | '!' | '?' | ')' | ']'))
}

fn is_markdown_line_start(output: &str) -> bool {
    output.is_empty() || output.ends_with('\n')
}

fn escape_markdown_text_backslashes(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(text.len());
    for (index, character) in chars.iter().copied().enumerate() {
        if character == '\\'
            && chars
                .get(index + 1)
                .is_some_and(|next| is_markdown_backslash_sensitive(*next))
        {
            output.push('\\');
        }
        output.push(character);
    }
    output
}

fn is_markdown_backslash_sensitive(character: char) -> bool {
    matches!(
        character,
        '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!'
    )
}

fn escape_markdown_line_start(text: &str) -> String {
    if starts_with_ordered_list_marker(text) {
        return text.replacen('.', "\\.", 1);
    }
    if text
        .strip_prefix(['-', '+'])
        .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    {
        return format!("\\{text}");
    }
    text.to_string()
}

fn starts_with_ordered_list_marker(text: &str) -> bool {
    let Some((prefix, rest)) = text.split_once('.') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|character| character.is_ascii_digit())
        && rest.starts_with(char::is_whitespace)
}

fn trim_trailing_horizontal_space(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

fn trailing_newline_count(output: &str) -> usize {
    output.chars().rev().take_while(|&c| c == '\n').count()
}

fn escape_link_text(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}

fn escape_markdown_link_target(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn escape_link_title(text: &str) -> String {
    escape_markdown_link_target(text).replace('"', "\\\"")
}

fn is_absolute_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn escape_table_cell(text: &str) -> String {
    text.replace('\n', " ").replace('|', "\\|")
}

pub(super) fn resolve_markdown_url(base: &str, raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("mailto:") {
        return raw.to_string();
    }
    Url::parse(base)
        .and_then(|base| base.join(raw))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| raw.to_string())
}

fn is_structural_block(tag: &str) -> bool {
    matches!(
        tag,
        "article" | "aside" | "footer" | "header" | "main" | "nav" | "section"
    )
}
