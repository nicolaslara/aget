mod escape;
mod url;
mod wrap;

pub(super) use escape::{
    escape_link_text, escape_link_title, escape_markdown_line_start, escape_markdown_link_target,
    escape_markdown_snob_text, escape_markdown_text_backslashes, escape_table_cell,
};
pub(super) use url::is_absolute_http_url;
pub(in crate::extraction) use url::resolve_markdown_url;
pub(in crate::extraction) use wrap::{apply_markdown_body_width, apply_markdown_single_line_break};

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

pub(super) fn normalize_unicode_snob_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        if let Some(replacement) = aget_unicode_replacement(character) {
            output.push_str(replacement);
        } else {
            output.push(character);
        }
    }
    output
}

fn aget_unicode_replacement(character: char) -> Option<&'static str> {
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

pub(super) fn trim_trailing_horizontal_space(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

pub(super) fn trailing_newline_count(output: &str) -> usize {
    output.chars().rev().take_while(|&c| c == '\n').count()
}
