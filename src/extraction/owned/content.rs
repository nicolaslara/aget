use ego_tree::NodeId;
use scraper::{ElementRef, Html};

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
        vec![default_main_content_element(&document, owned_options.word_count_threshold)?.id()]
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
    document = remove_owned_empty_elements(
        document,
        &root_ids,
        &target_ids,
        owned_options.word_count_threshold,
    );
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
        markdown: element_to_markdown(element, base_url, owned_options.only_text),
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
            .map(|element| element_to_markdown(*element, base_url, owned_options.only_text))
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

fn default_main_content_element(
    document: &Html,
    word_count_threshold: usize,
) -> Result<ElementRef<'_>, AgetError> {
    if let Some(element) = best_main_content_candidate(document, word_count_threshold)? {
        return Ok(element);
    }
    if let Some(body) = first_selected_element(document, "body")? {
        return Ok(body);
    }
    Ok(document.root_element())
}

fn best_main_content_candidate<'a>(
    document: &'a Html,
    word_count_threshold: usize,
) -> Result<Option<ElementRef<'a>>, AgetError> {
    let mut best = None;
    for selector in ["main", r#"[role="main"]"#, "article", "section", "div"] {
        let selector = parse_css_selector(selector)?;
        for element in document.select(&selector) {
            if is_inside_crawl4ai_pruning_excluded_tag(element) {
                continue;
            }
            let score = score_main_content_candidate(element, word_count_threshold)?;
            if score <= 0 {
                continue;
            }
            let replace = best
                .as_ref()
                .map(|(best_score, _)| score > *best_score)
                .unwrap_or(true);
            if replace {
                best = Some((score, element.id()));
            }
        }
    }
    best.map(|(_, id)| element_by_id(document, id)).transpose()
}

fn is_inside_crawl4ai_pruning_excluded_tag(element: ElementRef<'_>) -> bool {
    element.ancestors().any(|ancestor| {
        ElementRef::wrap(ancestor)
            .map(|ancestor| {
                matches!(
                    ancestor.value().name(),
                    "nav"
                        | "footer"
                        | "header"
                        | "aside"
                        | "script"
                        | "style"
                        | "form"
                        | "iframe"
                        | "noscript"
                )
            })
            .unwrap_or(false)
    })
}

fn score_main_content_candidate(
    element: ElementRef<'_>,
    word_count_threshold: usize,
) -> Result<i64, AgetError> {
    let text_words = word_count(element.text());
    if text_words == 0 {
        return Ok(0);
    }
    if word_count_threshold > 0 && text_words < word_count_threshold {
        return Ok(0);
    }
    let link_selector = parse_css_selector("a")?;
    let link_words = element
        .select(&link_selector)
        .map(|link| word_count(link.text()))
        .sum::<usize>();
    let tag_bonus = match element.value().name() {
        "main" => 300,
        "article" => 250,
        "section" => 125,
        "div" => 75,
        _ => 150,
    };
    let positive_label_bonus = content_label_bonus(element) as i64;
    if matches!(element.value().name(), "div" | "section") && positive_label_bonus == 0 {
        return Ok(0);
    }
    let label_penalty = content_label_penalty(element) as i64;
    let class_id_noise_penalty = crawl4ai_like_class_id_noise_penalty(element) as i64;
    let pruning_score = crawl4ai_like_pruning_score(element, &link_selector);
    Ok(
        (text_words as i64 * 8) - (link_words as i64 * 12) + tag_bonus + positive_label_bonus
            - label_penalty
            - class_id_noise_penalty
            + pruning_score,
    )
}

fn crawl4ai_like_pruning_score(element: ElementRef<'_>, link_selector: &scraper::Selector) -> i64 {
    let text_len = normalize_text_pieces(element.text()).chars().count();
    if text_len == 0 {
        return 0;
    }
    let tag_len = element.html().chars().count().max(1);
    let link_text_len = element
        .select(link_selector)
        .map(|link| normalize_text_pieces(link.text()).chars().count())
        .sum::<usize>();
    let text_density = text_len as f64 / tag_len as f64;
    let non_link_density = 1.0 - (link_text_len as f64 / text_len as f64).clamp(0.0, 1.0);
    let tag_weight = match element.value().name() {
        "article" => 1.5,
        "main" => 1.4,
        "section" | "p" => 1.0,
        "div" | "li" | "ul" | "ol" => 0.5,
        "h1" => 1.2,
        "h2" => 1.1,
        "h3" => 1.0,
        "h4" => 0.9,
        "h5" => 0.8,
        "h6" => 0.7,
        _ => 0.5,
    };
    let length_bonus = (text_len as f64 + 1.0).ln();

    ((text_density * 300.0)
        + (non_link_density * 220.0)
        + (tag_weight * 80.0)
        + (length_bonus * 20.0))
        .round() as i64
}

fn crawl4ai_like_class_id_noise_penalty(element: ElementRef<'_>) -> usize {
    // Crawl4AI's PruningContentFilter includes a class/id metric keyed off
    // generic navigation, advertising, comments, promo, and social labels.
    ["class", "id"]
        .into_iter()
        .filter_map(|attribute| element.attr(attribute))
        .filter(|value| has_crawl4ai_negative_label(value))
        .count()
        * 350
}

fn has_crawl4ai_negative_label(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "nav", "footer", "header", "sidebar", "ads", "comment", "promo", "advert", "social",
        "share",
    ]
    .into_iter()
    .any(|needle| lower.starts_with(needle))
}

fn word_count<'a>(pieces: impl IntoIterator<Item = &'a str>) -> usize {
    pieces
        .into_iter()
        .flat_map(str::split_whitespace)
        .filter(|word| !word.is_empty())
        .count()
}

fn content_label_penalty(element: ElementRef<'_>) -> usize {
    let label = ["id", "class", "role", "aria-label"]
        .into_iter()
        .filter_map(|attribute| element.attr(attribute))
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    [
        "nav", "footer", "header", "sidebar", "aside", "ad", "advert", "promo", "comment",
        "related", "share", "social",
    ]
    .into_iter()
    .filter(|needle| label.contains(needle))
    .count()
        * 200
}

fn content_label_bonus(element: ElementRef<'_>) -> usize {
    let label = ["id", "class", "role", "aria-label"]
        .into_iter()
        .filter_map(|attribute| element.attr(attribute))
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    [
        "article",
        "body",
        "content",
        "doc",
        "documentation",
        "entry",
        "main",
        "post",
        "story",
    ]
    .into_iter()
    .filter(|needle| label.contains(needle))
    .count()
        * 150
}

fn first_selected_element<'a>(
    document: &'a Html,
    raw_selector: &str,
) -> Result<Option<ElementRef<'a>>, AgetError> {
    let selector = parse_css_selector(raw_selector)?;
    Ok(document.select(&selector).next())
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

fn normalize_text_pieces<'a>(pieces: impl IntoIterator<Item = &'a str>) -> String {
    pieces
        .into_iter()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}
