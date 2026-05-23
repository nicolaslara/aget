mod google_doc;

use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use self::google_doc::{
    google_doc_list_is_ordered, has_google_doc_text_style, render_google_doc_styled_inline,
};
use super::block::{
    is_structural_block, render_block, render_blockquote, render_code_block,
    render_definition_list, render_heading, render_horizontal_rule, render_list,
};
use super::inline::{
    inline_markdown_from_children, inline_text_from_node, raw_text_from_node, render_abbreviation,
    render_image, render_link,
};
use super::table::render_table;
use super::writer::MarkdownWriter;

pub(in crate::extraction::markdown) fn render_node(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
    match node.value() {
        Node::Text(text) => writer.push_text(text),
        Node::Element(element) => render_element(node, element.name(), writer),
        _ => render_children(node, writer),
    }
}

fn render_element(node: NodeRef<'_, Node>, tag: &str, writer: &mut MarkdownWriter) {
    if writer.should_preserve_tag(tag) {
        render_preserved_html(node, writer);
        return;
    }

    if writer.hide_strikethrough && has_line_through_style(node) {
        return;
    }

    if writer.only_text && is_only_text_eligible_tag(tag) {
        writer.push_text(&raw_text_from_node(node));
        return;
    }

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => render_heading(node, tag, writer),
        "p" => render_block(node, writer),
        "br" => writer.output.push_str("  \n"),
        "hr" => render_horizontal_rule(writer),
        "ul" | "ol" if writer.google_doc => {
            render_list(node, google_doc_list_is_ordered(node, tag == "ol"), writer);
        }
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
            writer.push_inline(&format!(
                "{}{inner}{}",
                writer.strong_mark, writer.strong_mark
            ));
        }
        "em" | "i" | "u" if writer.ignore_emphasis => render_children(node, writer),
        "em" | "i" | "u" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!(
                "{}{inner}{}",
                writer.emphasis_mark, writer.emphasis_mark
            ));
        }
        "del" | "strike" | "s" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("~~{inner}~~"));
        }
        _ if writer.google_doc && has_google_doc_text_style(node) => {
            render_google_doc_styled_inline(node, writer);
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

fn render_preserved_html(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let Some(element) = ElementRef::wrap(node) else {
        render_children(node, writer);
        return;
    };
    writer.ensure_blank_line();
    writer.output.push_str(&element.html());
    writer.ensure_blank_line();
}

fn has_line_through_style(node: NodeRef<'_, Node>) -> bool {
    ElementRef::wrap(node)
        .and_then(|element| element.value().attr("style"))
        .is_some_and(|style| style.to_ascii_lowercase().contains("line-through"))
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

pub(in crate::extraction::markdown) fn render_children(
    node: NodeRef<'_, Node>,
    writer: &mut MarkdownWriter,
) {
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        render_node(current, writer);
        child = next;
    }
}
