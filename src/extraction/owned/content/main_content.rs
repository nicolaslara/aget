use ego_tree::NodeId;
use scraper::{ElementRef, Html};

use crate::error::AgetError;
use crate::extraction::html_clean::parse_css_selector;

pub(super) fn default_main_content_element_id(
    document: &Html,
    word_count_threshold: usize,
) -> Result<NodeId, AgetError> {
    if let Some(id) = best_main_content_candidate_id(document, word_count_threshold)? {
        return Ok(id);
    }
    if let Some(body) = first_selected_element(document, "body")? {
        return Ok(body.id());
    }
    Ok(document.root_element().id())
}

fn best_main_content_candidate_id(
    document: &Html,
    word_count_threshold: usize,
) -> Result<Option<NodeId>, AgetError> {
    let mut best = None;
    for selector in ["main", r#"[role="main"]"#, "article", "section", "div"] {
        let selector = parse_css_selector(selector)?;
        for element in document.select(&selector) {
            if is_inside_crawl4ai_pruning_excluded_tag(element) {
                continue;
            }
            if has_crawl4ai_negative_class_id_label(element) {
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
    Ok(best.map(|(_, id)| id))
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
    let label_penalty = content_label_penalty(element) as i64;
    let class_id_noise_penalty = crawl4ai_like_class_id_noise_penalty(element) as i64;
    let pruning_score = crawl4ai_like_pruning_score(element, &link_selector);
    if matches!(element.value().name(), "div" | "section")
        && positive_label_bonus == 0
        && !is_crawl4ai_dense_generic_candidate(element, pruning_score)?
    {
        return Ok(0);
    }
    Ok(
        (text_words as i64 * 8) - (link_words as i64 * 12) + tag_bonus + positive_label_bonus
            - label_penalty
            - class_id_noise_penalty
            + pruning_score,
    )
}

fn is_crawl4ai_dense_generic_candidate(
    element: ElementRef<'_>,
    pruning_score: i64,
) -> Result<bool, AgetError> {
    if pruning_score < 450 {
        return Ok(false);
    }
    if contains_descendant_positive_content_container(element)? {
        return Ok(false);
    }
    Ok(true)
}

fn contains_descendant_positive_content_container(
    element: ElementRef<'_>,
) -> Result<bool, AgetError> {
    let selector = parse_css_selector(r#"main, [role="main"], article, section, div"#)?;
    Ok(element
        .select(&selector)
        .any(|descendant| descendant.id() != element.id() && content_label_bonus(descendant) > 0))
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

fn has_crawl4ai_negative_class_id_label(element: ElementRef<'_>) -> bool {
    // Crawl4AI's relevant-content path excludes candidates whose class/id
    // contains these generic page-chrome and low-signal content labels.
    ["class", "id"]
        .into_iter()
        .filter_map(|attribute| element.attr(attribute))
        .any(has_crawl4ai_negative_label)
}

fn has_crawl4ai_negative_label(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "nav", "footer", "header", "sidebar", "ads", "comment", "promo", "advert", "social",
        "share",
    ]
    .into_iter()
    .any(|needle| lower.contains(needle))
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

fn normalize_text_pieces<'a>(pieces: impl IntoIterator<Item = &'a str>) -> String {
    pieces
        .into_iter()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}
