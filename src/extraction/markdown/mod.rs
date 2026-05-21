use std::cell::RefCell;
use std::rc::Rc;

use ego_tree::NodeRef;
use scraper::{ElementRef, Node};
use url::Url;

mod normalize;
mod table;

use self::normalize::{
    escape_link_text, escape_link_title, escape_markdown_line_start, escape_markdown_link_target,
    escape_markdown_text_backslashes, is_absolute_http_url, is_markdown_line_start,
    needs_space_before_inline, normalize_inline_markdown, starts_with_closing_punctuation,
    trailing_newline_count, trim_trailing_horizontal_space,
};
pub(super) use self::normalize::{normalize_markdown, resolve_markdown_url};
use self::table::render_table;

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
    if !label.is_empty() && label == href && is_absolute_http_url(href) {
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

fn is_structural_block(tag: &str) -> bool {
    matches!(
        tag,
        "article" | "aside" | "footer" | "header" | "main" | "nav" | "section"
    )
}
