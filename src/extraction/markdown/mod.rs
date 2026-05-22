use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

mod inline;
mod normalize;
mod table;
mod writer;

use self::inline::{
    inline_markdown_from_children, inline_text_from_node, raw_text_from_node, render_abbreviation,
    render_image, render_link,
};
use self::normalize::trim_trailing_horizontal_space;
pub(super) use self::normalize::{normalize_markdown, resolve_markdown_url};
use self::table::render_table;
use self::writer::MarkdownWriter;

pub(super) fn element_to_markdown(
    element: ElementRef<'_>,
    base_url: &str,
    only_text: bool,
    skip_internal_links: bool,
    default_image_alt: &str,
    open_quote: &str,
    close_quote: &str,
    ignore_images: bool,
    images_as_html: bool,
    images_to_alt: bool,
    images_with_size: bool,
    ignore_emphasis: bool,
    ignore_links: bool,
    ignore_mailto_links: bool,
    ignore_tables: bool,
    bypass_tables: bool,
    protect_links: bool,
    use_automatic_links: bool,
    escape_snob: bool,
    include_sup_sub: bool,
) -> String {
    let mut writer = MarkdownWriter::new(
        base_url,
        only_text,
        skip_internal_links,
        default_image_alt,
        open_quote,
        close_quote,
        ignore_images,
        images_as_html,
        images_to_alt,
        images_with_size,
        ignore_emphasis,
        ignore_links,
        ignore_mailto_links,
        ignore_tables,
        bypass_tables,
        protect_links,
        use_automatic_links,
        escape_snob,
        include_sup_sub,
    );
    render_node(*element, &mut writer);
    writer.append_abbreviation_definitions();
    normalize_markdown(&writer.output)
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
        "code" | "kbd" | "tt" if writer.inside_link => {
            writer.push_inline(&inline_text_from_node(node));
        }
        "code" | "kbd" | "tt" => writer.push_inline(&format!("`{}`", inline_text_from_node(node))),
        "address" | "details" | "figcaption" | "figure" | "summary" => render_block(node, writer),
        "strong" | "b" if writer.ignore_emphasis => render_children(node, writer),
        "strong" | "b" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("**{inner}**"));
        }
        "em" | "i" | "u" if writer.ignore_emphasis => render_children(node, writer),
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
            writer.push_inline(&format!(
                "{}{inner}{}",
                writer.open_quote, writer.close_quote
            ));
        }
        "sub" | "sup" if writer.include_sup_sub => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("<{tag}>{inner}</{tag}>"));
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

fn is_structural_block(tag: &str) -> bool {
    matches!(
        tag,
        "article" | "aside" | "footer" | "header" | "main" | "nav" | "section"
    )
}
