use scraper::ElementRef;

mod block;
mod dispatch;
mod inline;
mod normalize;
mod table;
mod writer;

pub(in crate::extraction::markdown) use self::dispatch::{render_children, render_node};
pub(in crate::extraction::markdown) use self::inline::inline_markdown_from_children;
pub(super) use self::normalize::{
    apply_markdown_body_width, apply_markdown_single_line_break, normalize_markdown,
    resolve_markdown_url,
};
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
    preserve_tags: &[String],
    handle_code_in_pre: bool,
    ignore_emphasis: bool,
    ignore_links: bool,
    inline_links: bool,
    links_each_paragraph: bool,
    ignore_mailto_links: bool,
    ignore_tables: bool,
    bypass_tables: bool,
    hide_strikethrough: bool,
    google_doc: bool,
    google_list_indent: usize,
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
        preserve_tags,
        handle_code_in_pre,
        ignore_emphasis,
        ignore_links,
        inline_links,
        links_each_paragraph,
        ignore_mailto_links,
        ignore_tables,
        bypass_tables,
        hide_strikethrough,
        google_doc,
        google_list_indent,
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
