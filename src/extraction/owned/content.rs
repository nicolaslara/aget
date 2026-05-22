mod main_content;

use ego_tree::NodeId;
use scraper::{ElementRef, Html};

use main_content::default_main_content_element_id;

use super::options::OwnedExtractorOptions;
use super::text::element_to_text;
use crate::error::AgetError;

use crate::extraction::extraction_failed;
use crate::extraction::html_clean::{
    clean_owned_base64_image_sources, parse_css_selector, prune_owned_unwanted_attributes,
    remove_owned_empty_elements, replace_owned_only_text_elements,
};
use crate::extraction::markdown::{element_to_markdown, normalize_markdown, resolve_markdown_url};

pub(super) struct ExtractedOwnedContent {
    pub(super) html: String,
    pub(super) markdown: String,
    pub(super) text: String,
}

pub(super) fn extract_owned_content(
    mut document: Html,
    selector: Option<&str>,
    base_url: &str,
    prefer_main_content: bool,
    owned_options: &OwnedExtractorOptions,
) -> Result<ExtractedOwnedContent, AgetError> {
    let root_ids = if let Some(raw_selector) = selector {
        match parse_css_selector(raw_selector) {
            Ok(selector) => {
                let selected = document
                    .select(&selector)
                    .map(|element| element.id())
                    .collect::<Vec<_>>();
                if selected.is_empty() {
                    vec![document.root_element().id()]
                } else {
                    selected
                }
            }
            Err(_) => vec![document.root_element().id()],
        }
    } else if !owned_options.target_elements.is_empty() {
        vec![document.root_element().id()]
    } else if prefer_main_content {
        vec![default_main_content_element_id(
            &document,
            owned_options.word_count_threshold,
        )?]
    } else {
        vec![document.root_element().id()]
    };
    let target_ids = if owned_options.target_elements.is_empty() {
        Vec::new()
    } else {
        collect_target_owned_element_ids(&document, &root_ids, &owned_options.target_elements)?
    };

    // Match Crawl4AI's cleanup order: selectors see original attributes, but
    // serialized cleaned HTML keeps only its small important-attribute allowlist.
    if owned_options.only_text {
        document = replace_owned_only_text_elements(document, &root_ids, &target_ids);
    }
    document = clean_owned_base64_image_sources(document);
    document = remove_owned_empty_elements(document, &root_ids, &target_ids);
    document = prune_owned_unwanted_attributes(document, owned_options.keep_data_attributes);

    if owned_options.target_elements.is_empty() {
        if let [root_id] = root_ids.as_slice() {
            let root = element_by_id(&document, *root_id)?;
            return Ok(extract_single_owned_element(root, base_url, owned_options));
        }
        return extract_target_owned_elements(&document, &root_ids, base_url, owned_options);
    }
    extract_target_owned_elements(&document, &target_ids, base_url, owned_options)
}

fn extract_single_owned_element(
    element: ElementRef<'_>,
    base_url: &str,
    owned_options: &OwnedExtractorOptions,
) -> ExtractedOwnedContent {
    ExtractedOwnedContent {
        html: element.inner_html(),
        markdown: element_to_markdown(
            element,
            base_url,
            owned_options.only_text,
            owned_options.skip_internal_links,
            owned_options.ignore_images,
            owned_options.ignore_emphasis,
            owned_options.ignore_links,
            owned_options.ignore_mailto_links,
            owned_options.protect_links,
            owned_options.escape_snob,
            owned_options.include_sup_sub,
        ),
        text: element_to_text(element),
    }
}

fn collect_target_owned_element_ids(
    document: &Html,
    source_ids: &[NodeId],
    raw_selectors: &[String],
) -> Result<Vec<NodeId>, AgetError> {
    let mut ids = Vec::new();
    for source_id in source_ids {
        let source = element_by_id(document, *source_id)?;
        for raw_selector in raw_selectors {
            let selector = parse_css_selector(raw_selector)?;
            ids.extend(source.select(&selector).map(|element| element.id()));
        }
    }
    Ok(ids)
}

fn extract_target_owned_elements(
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
            .map(|element| {
                element_to_markdown(
                    *element,
                    base_url,
                    owned_options.only_text,
                    owned_options.skip_internal_links,
                    owned_options.ignore_images,
                    owned_options.ignore_emphasis,
                    owned_options.ignore_links,
                    owned_options.ignore_mailto_links,
                    owned_options.protect_links,
                    owned_options.escape_snob,
                    owned_options.include_sup_sub,
                )
            })
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

fn element_by_id(document: &Html, id: NodeId) -> Result<ElementRef<'_>, AgetError> {
    document
        .tree
        .get(id)
        .and_then(ElementRef::wrap)
        .ok_or_else(|| extraction_failed("owned extractor lost a selected HTML element"))
}

pub(super) fn markdown_base_url(document: &Html, final_url: &str) -> Result<String, AgetError> {
    let selector = parse_css_selector("base[href]")?;
    let Some(base_href) = document
        .select(&selector)
        .next()
        .and_then(|element| element.attr("href"))
    else {
        return Ok(final_url.to_string());
    };
    Ok(resolve_markdown_url(final_url, base_href))
}
