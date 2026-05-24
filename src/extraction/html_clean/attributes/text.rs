use ego_tree::NodeId;
use scraper::node::Text;
use scraper::{ElementRef, Html, Node, StrTendril};

const CRAWL4AI_ONLY_TEXT_ELIGIBLE_TAGS: &[&str] = &[
    "b", "i", "u", "span", "del", "ins", "sub", "sup", "strong", "em", "code", "kbd", "var", "s",
    "q", "abbr", "cite", "dfn", "time", "small", "mark",
];

pub(in crate::extraction) fn replace_owned_only_text_elements(
    mut document: Html,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
) -> Html {
    let node_ids = document
        .tree
        .nodes()
        .filter_map(ElementRef::wrap)
        .filter(|element| should_replace_with_only_text(*element, root_ids, target_ids))
        .map(|element| element.id())
        .collect::<Vec<_>>();

    for id in node_ids {
        let text = document
            .tree
            .get(id)
            .and_then(ElementRef::wrap)
            .map(|element| element.text().collect::<String>())
            .unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        if let Some(mut node) = document.tree.get_mut(id) {
            node.insert_after(Node::Text(Text {
                text: StrTendril::from_slice(&text),
            }));
            node.detach();
        }
    }
    document
}

fn should_replace_with_only_text(
    element: ElementRef<'_>,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
) -> bool {
    let id = element.id();
    CRAWL4AI_ONLY_TEXT_ELIGIBLE_TAGS.contains(&element.value().name())
        && !root_ids.contains(&id)
        && !target_ids.contains(&id)
        && !has_only_text_eligible_ancestor(element, root_ids, target_ids)
        && !has_protected_descendant(element, root_ids, target_ids)
}

fn has_only_text_eligible_ancestor(
    element: ElementRef<'_>,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
) -> bool {
    element.ancestors().skip(1).any(|ancestor| {
        ElementRef::wrap(ancestor)
            .map(|ancestor| {
                CRAWL4AI_ONLY_TEXT_ELIGIBLE_TAGS.contains(&ancestor.value().name())
                    && !root_ids.contains(&ancestor.id())
                    && !target_ids.contains(&ancestor.id())
            })
            .unwrap_or(false)
    })
}

fn has_protected_descendant(
    element: ElementRef<'_>,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
) -> bool {
    element.descendants().skip(1).any(|descendant| {
        let id = descendant.id();
        root_ids.contains(&id) || target_ids.contains(&id)
    })
}
