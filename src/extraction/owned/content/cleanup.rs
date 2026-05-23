use ego_tree::NodeId;
use html5ever::tree_builder::TreeSink;
use scraper::{ElementRef, Html, HtmlTreeSink};

pub(super) fn remove_owned_line_through_elements(
    document: Html,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
) -> Html {
    let node_ids = document
        .tree
        .nodes()
        .filter_map(ElementRef::wrap)
        .filter(|element| should_remove_owned_line_through_element(*element, root_ids, target_ids))
        .map(|element| element.id())
        .collect::<Vec<_>>();
    let sink = HtmlTreeSink::new(document);
    for id in node_ids {
        sink.remove_from_parent(&id);
    }
    sink.finish()
}

fn should_remove_owned_line_through_element(
    element: ElementRef<'_>,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
) -> bool {
    let id = element.id();
    !root_ids.contains(&id)
        && !target_ids.contains(&id)
        && element
            .value()
            .attr("style")
            .is_some_and(|style| style.to_ascii_lowercase().contains("line-through"))
}
