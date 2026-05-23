pub(in crate::extraction::markdown) fn escape_markdown_text_backslashes(text: &str) -> String {
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

pub(in crate::extraction::markdown) fn escape_markdown_snob_text(text: &str) -> String {
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

pub(in crate::extraction::markdown) fn escape_markdown_line_start(
    text: &str,
    escape_dot: bool,
    escape_plus: bool,
    escape_dash: bool,
) -> String {
    if escape_dot && starts_with_ordered_list_marker(text) {
        return text.replacen('.', "\\.", 1);
    }
    if escape_dash
        && text
            .strip_prefix('-')
            .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    {
        return format!("\\{text}");
    }
    if escape_plus
        && text
            .strip_prefix('+')
            .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    {
        return format!("\\{text}");
    }
    text.to_string()
}

pub(super) fn starts_with_ordered_list_marker(text: &str) -> bool {
    let Some((prefix, rest)) = text.split_once('.') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|character| character.is_ascii_digit())
        && rest.starts_with(char::is_whitespace)
}

pub(in crate::extraction::markdown) fn escape_link_text(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}

pub(in crate::extraction::markdown) fn escape_markdown_link_target(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

pub(in crate::extraction::markdown) fn escape_link_title(text: &str) -> String {
    escape_markdown_link_target(text).replace('"', "\\\"")
}

pub(in crate::extraction::markdown) fn escape_table_cell(text: &str) -> String {
    text.replace('\n', " ").replace('|', "\\|")
}
