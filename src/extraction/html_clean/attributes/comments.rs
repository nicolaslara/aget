use html5ever::tree_builder::TreeSink;
use scraper::{Html, HtmlTreeSink, Node};

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
