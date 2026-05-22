use url::Url;

pub(in crate::extraction) fn normalize_markdown(markdown: &str) -> String {
    let mut output = String::new();
    let mut blank_lines = 0usize;
    let mut in_code_fence = false;
    for raw_line in markdown.lines() {
        if in_code_fence {
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(raw_line);
            output.push('\n');
            if raw_line.trim_start().starts_with("```") {
                in_code_fence = false;
                blank_lines = 0;
            }
            continue;
        }

        let line = normalize_markdown_line_end(raw_line);
        if line.trim_start().starts_with("```") {
            blank_lines = 0;
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&line);
            output.push('\n');
            in_code_fence = true;
            continue;
        }

        if line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 && !output.is_empty() {
                output.push('\n');
            }
            continue;
        }
        blank_lines = 0;
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&line);
        output.push('\n');
    }
    output.trim().to_string()
}

fn normalize_markdown_line_end(line: &str) -> String {
    let without_tabs = line.trim_end_matches('\t');
    if without_tabs.ends_with("  ") {
        let without_spaces = without_tabs.trim_end_matches(' ');
        if !without_spaces.is_empty() {
            return format!("{without_spaces}  ");
        }
    }
    line.trim_end().to_string()
}

pub(super) fn normalize_inline_markdown(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn needs_space_before_inline(output: &str) -> bool {
    output
        .chars()
        .last()
        .is_some_and(|character| !character.is_whitespace())
}

pub(super) fn starts_with_closing_punctuation(text: &str) -> bool {
    if text.starts_with("![") {
        return false;
    }
    text.chars()
        .next()
        .is_some_and(|character| matches!(character, '.' | ',' | ':' | ';' | '!' | '?' | ')' | ']'))
}

pub(super) fn is_markdown_line_start(output: &str) -> bool {
    output.is_empty() || output.ends_with('\n')
}

pub(super) fn escape_markdown_text_backslashes(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(text.len());
    for (index, character) in chars.iter().copied().enumerate() {
        if character == '\\'
            && chars
                .get(index + 1)
                .is_some_and(|next| is_markdown_backslash_sensitive(*next))
        {
            output.push('\\');
        }
        output.push(character);
    }
    output
}

fn is_markdown_backslash_sensitive(character: char) -> bool {
    matches!(
        character,
        '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!'
    )
}

pub(super) fn escape_markdown_line_start(text: &str) -> String {
    if starts_with_ordered_list_marker(text) {
        return text.replacen('.', "\\.", 1);
    }
    if text
        .strip_prefix(['-', '+'])
        .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    {
        return format!("\\{text}");
    }
    text.to_string()
}

fn starts_with_ordered_list_marker(text: &str) -> bool {
    let Some((prefix, rest)) = text.split_once('.') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|character| character.is_ascii_digit())
        && rest.starts_with(char::is_whitespace)
}

pub(super) fn trim_trailing_horizontal_space(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

pub(super) fn trailing_newline_count(output: &str) -> usize {
    output.chars().rev().take_while(|&c| c == '\n').count()
}

pub(super) fn escape_link_text(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}

pub(super) fn escape_markdown_link_target(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

pub(super) fn escape_link_title(text: &str) -> String {
    escape_markdown_link_target(text).replace('"', "\\\"")
}

pub(super) fn is_absolute_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

pub(super) fn escape_table_cell(text: &str) -> String {
    text.replace('\n', " ").replace('|', "\\|")
}

pub(in crate::extraction) fn resolve_markdown_url(base: &str, raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("mailto:") {
        return raw.to_string();
    }
    Url::parse(base)
        .and_then(|base| base.join(raw))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| raw.to_string())
}
