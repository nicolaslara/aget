use super::escape::starts_with_ordered_list_marker;

pub(in crate::extraction) fn apply_markdown_body_width(
    markdown: &str,
    body_width: usize,
    wrap_links: bool,
    wrap_list_items: bool,
    wrap_tables: bool,
) -> String {
    if body_width == 0 || markdown.is_empty() {
        return markdown.to_string();
    }

    let mut output = String::new();
    let mut in_code_fence = false;
    for line in markdown.lines() {
        if is_code_fence_line(line) {
            push_wrapped_line(&mut output, line);
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence
            || should_preserve_markdown_line(line, wrap_links, wrap_list_items, wrap_tables)
        {
            push_wrapped_line(&mut output, line);
            continue;
        }
        for wrapped_line in wrap_markdown_line(line, body_width) {
            push_wrapped_line(&mut output, &wrapped_line);
        }
    }
    output.trim_end().to_string()
}

pub(in crate::extraction) fn apply_markdown_single_line_break(
    markdown: &str,
    single_line_break: bool,
) -> String {
    if !single_line_break || markdown.is_empty() {
        return markdown.to_string();
    }

    let mut output = String::new();
    let mut in_code_fence = false;
    for line in markdown.lines() {
        if is_code_fence_line(line) {
            push_wrapped_line(&mut output, line);
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            push_wrapped_line(&mut output, line);
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        push_wrapped_line(&mut output, line);
    }
    output.trim_end().to_string()
}

fn is_code_fence_line(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

fn should_preserve_markdown_line(
    line: &str,
    wrap_links: bool,
    wrap_list_items: bool,
    wrap_tables: bool,
) -> bool {
    let trimmed = line.trim_start();
    line.trim().is_empty()
        || line.ends_with("  ")
        || line.starts_with(char::is_whitespace)
        || trimmed.starts_with('#')
        || (!wrap_tables && is_markdown_table_line(trimmed))
        || trimmed.starts_with('>')
        || trimmed.starts_with("![")
        || trimmed.starts_with("*[")
        || trimmed == "* * *"
        || (!wrap_list_items && is_markdown_list_item_line(trimmed))
        || (!wrap_links && contains_inline_markdown_link(line))
}

fn is_markdown_list_item_line(trimmed: &str) -> bool {
    trimmed.starts_with("* ")
        || trimmed.starts_with("- ")
        || trimmed.starts_with("+ ")
        || starts_with_ordered_list_marker(trimmed)
}

fn is_markdown_table_line(trimmed: &str) -> bool {
    trimmed.starts_with('|') || trimmed.contains(" | ")
}

fn contains_inline_markdown_link(line: &str) -> bool {
    let bytes = line.as_bytes();
    bytes.windows(2).any(|window| window == b"](")
}

fn wrap_markdown_line(line: &str, body_width: usize) -> Vec<String> {
    let mut wrapped = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        let word_len = word.chars().count();
        let current_len = current.chars().count();
        let separator_len = usize::from(!current.is_empty());
        if !current.is_empty() && current_len + separator_len + word_len > body_width {
            wrapped.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        wrapped.push(current);
    }
    wrapped
}

fn push_wrapped_line(output: &mut String, line: &str) {
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(line);
}
