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

pub(in crate::extraction) fn apply_markdown_body_width(
    markdown: &str,
    body_width: usize,
    wrap_links: bool,
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
        if in_code_fence || should_preserve_markdown_line(line, wrap_links) {
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

fn should_preserve_markdown_line(line: &str, wrap_links: bool) -> bool {
    let trimmed = line.trim_start();
    line.trim().is_empty()
        || line.ends_with("  ")
        || line.starts_with(char::is_whitespace)
        || trimmed.starts_with('#')
        || trimmed.starts_with('|')
        || trimmed.starts_with('>')
        || trimmed.starts_with("![")
        || trimmed.starts_with("*[")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("- ")
        || trimmed.starts_with("+ ")
        || trimmed == "* * *"
        || starts_with_ordered_list_marker(trimmed)
        || (!wrap_links && contains_inline_markdown_link(line))
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

pub(super) fn normalize_unicode_snob_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        if let Some(replacement) = crawl4ai_unicode_replacement(character) {
            output.push_str(replacement);
        } else {
            output.push(character);
        }
    }
    output
}

fn crawl4ai_unicode_replacement(character: char) -> Option<&'static str> {
    match character {
        '\u{00a9}' => Some("(C)"),
        '\u{2019}' | '\u{2018}' => Some("'"),
        '\u{201d}' | '\u{201c}' => Some("\""),
        '\u{2014}' => Some("--"),
        '\u{2013}' => Some("-"),
        '\u{2192}' => Some("->"),
        '\u{2190}' => Some("<-"),
        '\u{00b7}' => Some("*"),
        '\u{0153}' => Some("oe"),
        '\u{00e6}' => Some("ae"),
        '\u{00e0}' | '\u{00e1}' | '\u{00e2}' | '\u{00e3}' | '\u{00e4}' | '\u{00e5}' => Some("a"),
        '\u{00e8}' | '\u{00e9}' | '\u{00ea}' | '\u{00eb}' => Some("e"),
        '\u{00ec}' | '\u{00ed}' | '\u{00ee}' | '\u{00ef}' => Some("i"),
        '\u{00f2}' | '\u{00f3}' | '\u{00f4}' | '\u{00f5}' | '\u{00f6}' => Some("o"),
        '\u{00f9}' | '\u{00fa}' | '\u{00fb}' | '\u{00fc}' => Some("u"),
        '\u{200e}' | '\u{200f}' => Some(""),
        _ => None,
    }
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

pub(super) fn escape_markdown_snob_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        if matches!(
            character,
            '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '!'
        ) {
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
