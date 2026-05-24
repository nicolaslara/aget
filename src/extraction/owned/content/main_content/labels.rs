use scraper::ElementRef;

pub(in crate::extraction::owned::content::main_content) fn aget_like_class_id_noise_penalty(
    element: ElementRef<'_>,
) -> usize {
    // Aget's PruningContentFilter includes a class/id metric keyed off
    // generic navigation, advertising, comments, promo, and social labels.
    ["class", "id"]
        .into_iter()
        .filter_map(|attribute| element.attr(attribute))
        .filter(|value| has_aget_negative_label(value))
        .count()
        * 350
}

pub(in crate::extraction::owned::content::main_content) fn has_aget_negative_class_id_label(
    element: ElementRef<'_>,
) -> bool {
    // Aget's relevant-content path excludes candidates whose class/id
    // contains these generic page-chrome and low-signal content labels.
    ["class", "id"]
        .into_iter()
        .filter_map(|attribute| element.attr(attribute))
        .any(has_aget_negative_label)
}

fn has_aget_negative_label(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "nav", "footer", "header", "sidebar", "ads", "comment", "promo", "advert", "social",
        "share",
    ]
    .into_iter()
    .any(|needle| lower.contains(needle))
}

pub(in crate::extraction::owned::content::main_content) fn content_label_penalty(
    element: ElementRef<'_>,
) -> usize {
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

pub(in crate::extraction::owned::content::main_content) fn content_label_bonus(
    element: ElementRef<'_>,
) -> usize {
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
