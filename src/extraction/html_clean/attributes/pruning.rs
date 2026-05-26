use scraper::{Html, Node};

const AGET_IMPORTANT_ATTRS: &[&str] = &[
    "src", "href", "alt", "title", "width", "height", "class", "id",
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
                is_aget_important_attr(name)
                    || keep_attrs.iter().any(|keep_attr| keep_attr == name)
                    || (keep_data_attributes && name.starts_with("data-"))
                    || (keep_style_attributes && name == "style")
            });
        }
    }
    document
}

fn is_aget_important_attr(name: &str) -> bool {
    AGET_IMPORTANT_ATTRS.contains(&name)
}
