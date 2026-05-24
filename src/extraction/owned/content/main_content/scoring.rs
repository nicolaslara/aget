use scraper::ElementRef;

use crate::error::AgetError;
use crate::extraction::html_clean::parse_css_selector;

use super::labels::{aget_like_class_id_noise_penalty, content_label_bonus, content_label_penalty};
use super::text::{normalize_text_pieces, word_count};

pub(in crate::extraction::owned::content::main_content) fn score_main_content_candidate(
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
    let class_id_noise_penalty = aget_like_class_id_noise_penalty(element) as i64;
    let pruning_score = aget_like_pruning_score(element, &link_selector);
    if matches!(element.value().name(), "div" | "section")
        && positive_label_bonus == 0
        && !is_aget_dense_generic_candidate(element, pruning_score)?
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

fn is_aget_dense_generic_candidate(
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

fn aget_like_pruning_score(element: ElementRef<'_>, link_selector: &scraper::Selector) -> i64 {
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
