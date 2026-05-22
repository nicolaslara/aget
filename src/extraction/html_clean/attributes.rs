use ego_tree::NodeId;
use html5ever::tree_builder::TreeSink;
use scraper::node::Text;
use scraper::{ElementRef, Html, HtmlTreeSink, Node, StrTendril};

const CRAWL4AI_IMPORTANT_ATTRS: &[&str] = &[
    "src", "href", "alt", "title", "width", "height", "class", "id",
];
const CRAWL4AI_EMPTY_ELEMENT_BYPASS_TAGS: &[&str] = &[
    "a", "img", "br", "hr", "input", "meta", "link", "source", "track", "wbr", "tr", "td", "th",
];
const CRAWL4AI_EMPTY_ELEMENT_WORD_THRESHOLD: usize = 1;
const CRAWL4AI_ONLY_TEXT_ELIGIBLE_TAGS: &[&str] = &[
    "b", "i", "u", "span", "del", "ins", "sub", "sup", "strong", "em", "code", "kbd", "var", "s",
    "q", "abbr", "cite", "dfn", "time", "small", "mark",
];

pub(in crate::extraction) fn prune_owned_unwanted_attributes(
    mut document: Html,
    keep_data_attributes: bool,
    keep_attrs: &[String],
    keep_style_attributes: bool,
) -> Html {
    for node in document.tree.values_mut() {
        if let Node::Element(element) = node {
            element.attrs.retain(|(name, _)| {
                let name = name.local.as_ref();
                is_crawl4ai_important_attr(name)
                    || keep_attrs.iter().any(|keep_attr| keep_attr == name)
                    || (keep_data_attributes && name.starts_with("data-"))
                    || (keep_style_attributes && name == "style")
            });
        }
    }
    document
}

pub(in crate::extraction) fn clean_owned_base64_image_sources(mut document: Html) -> Html {
    for node in document.tree.values_mut() {
        let Node::Element(element) = node else {
            continue;
        };
        if element.name.local.as_ref() != "img" {
            continue;
        }
        for (name, value) in &mut element.attrs {
            if name.local.as_ref() == "src" && is_base64_image_src(value.as_ref()) {
                value.clear();
            }
        }
    }
    document
}

pub(in crate::extraction) fn remove_owned_comments(document: Html) -> Html {
    let node_ids = document
        .tree
        .nodes()
        .filter_map(|node| matches!(node.value(), Node::Comment(_)).then_some(node.id()))
        .collect::<Vec<_>>();
    let sink = HtmlTreeSink::new(document);
    for id in node_ids {
        sink.remove_from_parent(&id);
    }
    sink.finish()
}

pub(in crate::extraction) fn remove_owned_empty_elements(
    document: Html,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
) -> Html {
    let node_ids = document
        .tree
        .nodes()
        .filter_map(|node| ElementRef::wrap(node).map(|element| element.id()))
        .collect::<Vec<_>>();
    let sink = HtmlTreeSink::new(document);
    for id in node_ids.into_iter().rev() {
        let should_remove = {
            let document = sink.0.borrow();
            should_remove_owned_empty_element(
                &document,
                id,
                root_ids,
                target_ids,
                CRAWL4AI_EMPTY_ELEMENT_WORD_THRESHOLD,
            )
        };
        if should_remove {
            sink.remove_from_parent(&id);
        }
    }
    sink.finish()
}

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

fn should_remove_owned_empty_element(
    document: &Html,
    id: NodeId,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
    word_count_threshold: usize,
) -> bool {
    if root_ids.contains(&id) || target_ids.contains(&id) {
        return false;
    }
    let Some(element) = document.tree.get(id).and_then(ElementRef::wrap) else {
        return false;
    };
    if element.parent().is_none() {
        return false;
    }
    let tag = element.value().name();
    if CRAWL4AI_EMPTY_ELEMENT_BYPASS_TAGS.contains(&tag) || is_descendant_of_code_block(element) {
        return false;
    }
    if element.child_elements().next().is_some() {
        return false;
    }
    element.text().flat_map(str::split_whitespace).count() < word_count_threshold
}

fn is_descendant_of_code_block(element: ElementRef<'_>) -> bool {
    element.ancestors().any(|ancestor| {
        ElementRef::wrap(ancestor)
            .map(|ancestor| matches!(ancestor.value().name(), "pre" | "code"))
            .unwrap_or(false)
    })
}

fn is_base64_image_src(src: &str) -> bool {
    let Some(after_prefix) = src.strip_prefix("data:image/") else {
        return false;
    };
    let Some((mime_type, after_mime_type)) = after_prefix.split_once(';') else {
        return false;
    };
    !mime_type.is_empty() && after_mime_type.starts_with("base64,")
}

fn is_crawl4ai_important_attr(name: &str) -> bool {
    CRAWL4AI_IMPORTANT_ATTRS.contains(&name)
}
