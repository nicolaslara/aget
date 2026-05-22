pub(super) fn html_to_text(html: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    let mut skip_depth = 0usize;

    for character in html.chars() {
        match character {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                handle_html_tag(&tag, &mut output, &mut skip_depth);
            }
            _ if in_tag => tag.push(character),
            _ if skip_depth == 0 => output.push(character),
            _ => {}
        }
    }

    normalize_text(&decode_basic_entities(&output))
}

fn handle_html_tag(tag: &str, output: &mut String, skip_depth: &mut usize) {
    let tag_name = tag
        .trim()
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let closing = tag.trim_start().starts_with('/');

    if matches!(tag_name.as_str(), "script" | "style" | "noscript") {
        if closing {
            *skip_depth = skip_depth.saturating_sub(1);
        } else {
            *skip_depth += 1;
        }
        return;
    }

    if *skip_depth == 0
        && matches!(
            tag_name.as_str(),
            "br" | "p"
                | "div"
                | "section"
                | "article"
                | "main"
                | "li"
                | "tr"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
        )
    {
        output.push('\n');
    }
}

fn decode_basic_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn normalize_text(text: &str) -> String {
    let mut normalized = String::new();
    let mut blank_lines = 0usize;
    for line in text.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 && !normalized.is_empty() {
                normalized.push('\n');
            }
        } else {
            blank_lines = 0;
            normalized.push_str(&line);
            normalized.push('\n');
        }
    }
    normalized.trim().to_string()
}
