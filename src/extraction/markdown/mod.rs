use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

mod block;
mod inline;
mod normalize;
mod table;
mod writer;

use self::block::{
    is_structural_block, render_block, render_blockquote, render_code_block,
    render_definition_list, render_heading, render_horizontal_rule, render_list,
};
use self::inline::{
    inline_markdown_from_children, inline_text_from_node, raw_text_from_node, render_abbreviation,
    render_image, render_link,
};
pub(super) use self::normalize::{
    apply_markdown_body_width, apply_markdown_single_line_break, normalize_markdown,
    resolve_markdown_url,
};
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
    ul_item_mark: &str,
    emphasis_mark: &str,
    strong_mark: &str,
    ignore_images: bool,
    images_as_html: bool,
    images_to_alt: bool,
    images_with_size: bool,
    ignore_emphasis: bool,
    ignore_links: bool,
    inline_links: bool,
    links_each_paragraph: bool,
    ignore_mailto_links: bool,
    ignore_tables: bool,
    bypass_tables: bool,
    hide_strikethrough: bool,
    google_doc: bool,
    pad_tables: bool,
    protect_links: bool,
    use_automatic_links: bool,
    unicode_snob: bool,
    escape_backslash: bool,
    escape_snob: bool,
    escape_dot: bool,
    escape_plus: bool,
    escape_dash: bool,
    include_sup_sub: bool,
    single_line_break: bool,
    body_width: usize,
    wrap_links: bool,
    wrap_list_items: bool,
    wrap_tables: bool,
) -> String {
    let mut writer = MarkdownWriter::new(
        base_url,
        only_text,
        skip_internal_links,
        default_image_alt,
        open_quote,
        close_quote,
        ul_item_mark,
        emphasis_mark,
        strong_mark,
        ignore_images,
        images_as_html,
        images_to_alt,
        images_with_size,
        ignore_emphasis,
        ignore_links,
        inline_links,
        links_each_paragraph,
        ignore_mailto_links,
        ignore_tables,
        bypass_tables,
        hide_strikethrough,
        google_doc,
        protect_links,
        use_automatic_links,
        unicode_snob,
        escape_backslash,
        escape_snob,
        escape_dot,
        escape_plus,
        escape_dash,
        include_sup_sub,
    );
    render_node(*element, &mut writer);
    writer.append_reference_link_definitions();
    writer.append_abbreviation_definitions();
    let markdown = normalize_markdown(&writer.output);
    let markdown = apply_markdown_single_line_break(&markdown, single_line_break);
    let markdown = apply_markdown_body_width(
        &markdown,
        body_width,
        wrap_links,
        wrap_list_items,
        wrap_tables,
    );
    if pad_tables {
        table::pad_markdown_tables(&markdown)
    } else {
        markdown
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

fn has_line_through_style(node: NodeRef<'_, Node>) -> bool {
    ElementRef::wrap(node)
        .and_then(|element| element.value().attr("style"))
        .is_some_and(|style| style.to_ascii_lowercase().contains("line-through"))
}

fn has_google_doc_text_style(node: NodeRef<'_, Node>) -> bool {
    ElementRef::wrap(node)
        .and_then(|element| element.value().attr("style"))
        .is_some_and(|style| {
            let style = style.to_ascii_lowercase();
            google_doc_style_is_bold(&style)
                || style.contains("font-style:italic")
                || google_doc_style_is_fixed_width(&style)
        })
}

fn render_google_doc_styled_inline(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let Some(style) = ElementRef::wrap(node)
        .and_then(|element| element.value().attr("style"))
        .map(|style| style.to_ascii_lowercase())
    else {
        render_children(node, writer);
        return;
    };
    let inner = inline_markdown_from_children(node, writer);
    if inner.is_empty() {
        return;
    }
    let mut rendered = inner;
    if google_doc_style_is_fixed_width(&style) {
        rendered = format!("`{rendered}`");
    }
    if google_doc_style_is_bold(&style) {
        rendered = format!("{}{rendered}{}", writer.strong_mark, writer.strong_mark);
    }
    if style.contains("font-style:italic") {
        rendered = format!("{}{rendered}{}", writer.emphasis_mark, writer.emphasis_mark);
    }
    writer.push_inline(&rendered);
}

fn google_doc_style_is_bold(style: &str) -> bool {
    style.contains("font-weight:bold")
        || style.contains("font-weight: bold")
        || style.contains("font-weight:700")
        || style.contains("font-weight: 700")
}

fn google_doc_style_is_fixed_width(style: &str) -> bool {
    style.contains("font-family:consolas")
        || style.contains("font-family: consolas")
        || style.contains("font-family:courier new")
        || style.contains("font-family: courier new")
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
