use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

pub(super) fn element_to_text(element: ElementRef<'_>) -> String {
    let mut output = String::new();
    render_node(*element, &mut output, 0);
    normalize_text_output(&output)
}

fn render_node(node: NodeRef<'_, Node>, output: &mut String, skip_depth: usize) {
    match node.value() {
        Node::Text(text) if skip_depth == 0 => output.push_str(text),
        Node::Element(element) => render_element(node, element.name(), output, skip_depth),
        _ => render_children(node, output, skip_depth),
    }
}

fn render_element(node: NodeRef<'_, Node>, tag: &str, output: &mut String, skip_depth: usize) {
    if matches!(tag, "script" | "style" | "noscript") {
        render_children(node, output, skip_depth + 1);
        return;
    }
    if skip_depth == 0 && is_text_block_boundary_tag(tag) {
        output.push('\n');
    }
    render_children(node, output, skip_depth);
    if skip_depth == 0 && is_text_block_boundary_tag(tag) && tag != "br" {
        output.push('\n');
    }
}

fn render_children(node: NodeRef<'_, Node>, output: &mut String, skip_depth: usize) {
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        render_node(current, output, skip_depth);
        child = next;
    }
}

fn is_text_block_boundary_tag(tag: &str) -> bool {
    matches!(
        tag,
        "article"
            | "br"
            | "div"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "li"
            | "main"
            | "p"
            | "section"
            | "tr"
    )
}

fn normalize_text_output(text: &str) -> String {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
