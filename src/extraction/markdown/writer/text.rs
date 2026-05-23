use super::MarkdownWriter;
use crate::extraction::markdown::normalize::{
    escape_markdown_line_start, escape_markdown_snob_text, escape_markdown_text_backslashes,
    is_markdown_line_start, needs_space_before_inline, normalize_inline_markdown,
    normalize_unicode_snob_text, starts_with_closing_punctuation, trailing_newline_count,
    trim_trailing_horizontal_space,
};

impl MarkdownWriter {
    pub(in crate::extraction::markdown) fn push_text(&mut self, text: &str) {
        let mut text = normalize_inline_markdown(text);
        if !self.unicode_snob {
            text = normalize_unicode_snob_text(&text);
        }
        if self.escape_backslash {
            text = escape_markdown_text_backslashes(&text);
        }
        if self.escape_snob {
            text = escape_markdown_snob_text(&text);
        }
        if is_markdown_line_start(&self.output) {
            text = escape_markdown_line_start(
                &text,
                self.escape_dot,
                self.escape_plus,
                self.escape_dash,
            );
        }
        self.push_inline(&text);
    }

    pub(in crate::extraction::markdown) fn push_inline(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if needs_space_before_inline(&self.output) && !starts_with_closing_punctuation(text) {
            self.output.push(' ');
        }
        self.output.push_str(text);
    }

    pub(in crate::extraction::markdown) fn ensure_blank_line(&mut self) {
        trim_trailing_horizontal_space(&mut self.output);
        if self.output.is_empty() {
            return;
        }
        match trailing_newline_count(&self.output) {
            0 => self.output.push_str("\n\n"),
            1 => self.output.push('\n'),
            _ => {}
        }
    }
}
