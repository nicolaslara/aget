use ego_tree::NodeId;
use scraper::{ElementRef, Html};

use super::selection::element_by_id;
use super::ExtractedOwnedContent;
use crate::error::AgetError;
use crate::extraction::markdown::{element_to_markdown, normalize_markdown};

use super::super::options::OwnedExtractorOptions;
use super::super::text::element_to_text;

pub(super) fn extract_single_owned_element(
    element: ElementRef<'_>,
    base_url: &str,
    owned_options: &OwnedExtractorOptions,
) -> ExtractedOwnedContent {
    ExtractedOwnedContent {
        html: element.inner_html(),
        markdown: element_to_owned_markdown(element, base_url, owned_options),
        text: element_to_text(element),
    }
}

pub(super) fn extract_target_owned_elements(
    document: &Html,
    element_ids: &[NodeId],
    base_url: &str,
    owned_options: &OwnedExtractorOptions,
) -> Result<ExtractedOwnedContent, AgetError> {
    let elements = element_ids
        .iter()
        .map(|id| element_by_id(document, *id))
        .collect::<Result<Vec<_>, _>>()?;

    let html = elements
        .iter()
        .map(|element| element.html())
        .collect::<Vec<_>>()
        .join("\n");
    let markdown = normalize_markdown(
        &elements
            .iter()
            .map(|element| element_to_owned_markdown(*element, base_url, owned_options))
            .filter(|markdown| !markdown.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
    );
    let text = elements
        .iter()
        .map(|element| element_to_text(*element))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(ExtractedOwnedContent {
        html,
        markdown,
        text,
    })
}

fn element_to_owned_markdown(
    element: ElementRef<'_>,
    base_url: &str,
    owned_options: &OwnedExtractorOptions,
) -> String {
    element_to_markdown(
        element,
        base_url,
        owned_options.only_text,
        owned_options.skip_internal_links,
        &owned_options.default_image_alt,
        &owned_options.open_quote,
        &owned_options.close_quote,
        &owned_options.ul_item_mark,
        &owned_options.emphasis_mark,
        &owned_options.strong_mark,
        owned_options.ignore_images,
        owned_options.images_as_html,
        owned_options.images_to_alt,
        owned_options.images_with_size,
        &owned_options.preserve_tags,
        owned_options.handle_code_in_pre,
        owned_options.ignore_emphasis,
        owned_options.ignore_links,
        owned_options.inline_links,
        owned_options.links_each_paragraph,
        owned_options.ignore_mailto_links,
        owned_options.ignore_tables,
        owned_options.bypass_tables,
        owned_options.hide_strikethrough,
        owned_options.google_doc,
        owned_options.google_list_indent,
        owned_options.pad_tables,
        owned_options.protect_links,
        owned_options.use_automatic_links,
        owned_options.unicode_snob,
        owned_options.escape_backslash,
        owned_options.escape_snob,
        owned_options.escape_dot,
        owned_options.escape_plus,
        owned_options.escape_dash,
        owned_options.include_sup_sub,
        owned_options.single_line_break,
        owned_options.body_width,
        owned_options.wrap_links,
        owned_options.wrap_list_items,
        owned_options.wrap_tables,
    )
}
