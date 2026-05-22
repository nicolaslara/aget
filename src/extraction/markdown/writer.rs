use std::cell::RefCell;
use std::rc::Rc;

use url::Url;

use super::normalize::{
    escape_markdown_line_start, escape_markdown_snob_text, escape_markdown_text_backslashes,
    is_markdown_line_start, needs_space_before_inline, normalize_inline_markdown,
    resolve_markdown_url, starts_with_closing_punctuation, trailing_newline_count,
    trim_trailing_horizontal_space,
};

pub(super) struct MarkdownWriter {
    pub(super) output: String,
    base_url: Option<Url>,
    pub(super) only_text: bool,
    pub(super) skip_internal_links: bool,
    pub(super) ignore_images: bool,
    pub(super) ignore_emphasis: bool,
    pub(super) ignore_links: bool,
    pub(super) ignore_mailto_links: bool,
    pub(super) ignore_tables: bool,
    pub(super) bypass_tables: bool,
    pub(super) protect_links: bool,
    escape_snob: bool,
    pub(super) include_sup_sub: bool,
    pub(super) inside_link: bool,
    pub(super) list_depth: usize,
    abbreviations: Rc<RefCell<Vec<(String, String)>>>,
}

impl MarkdownWriter {
    pub(super) fn new(
        base_url: &str,
        only_text: bool,
        skip_internal_links: bool,
        ignore_images: bool,
        ignore_emphasis: bool,
        ignore_links: bool,
        ignore_mailto_links: bool,
        ignore_tables: bool,
        bypass_tables: bool,
        protect_links: bool,
        escape_snob: bool,
        include_sup_sub: bool,
    ) -> Self {
        Self {
            output: String::new(),
            base_url: Url::parse(base_url).ok(),
            only_text,
            skip_internal_links,
            ignore_images,
            ignore_emphasis,
            ignore_links,
            ignore_mailto_links,
            ignore_tables,
            bypass_tables,
            protect_links,
            escape_snob,
            include_sup_sub,
            inside_link: false,
            list_depth: 0,
            abbreviations: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(super) fn child(&self) -> Self {
        Self {
            output: String::new(),
            base_url: self.base_url.clone(),
            only_text: self.only_text,
            skip_internal_links: self.skip_internal_links,
            ignore_images: self.ignore_images,
            ignore_emphasis: self.ignore_emphasis,
            ignore_links: self.ignore_links,
            ignore_mailto_links: self.ignore_mailto_links,
            ignore_tables: self.ignore_tables,
            bypass_tables: self.bypass_tables,
            protect_links: self.protect_links,
            escape_snob: self.escape_snob,
            include_sup_sub: self.include_sup_sub,
            inside_link: self.inside_link,
            list_depth: self.list_depth,
            abbreviations: Rc::clone(&self.abbreviations),
        }
    }

    pub(super) fn link_child(&self) -> Self {
        let mut child = self.child();
        child.inside_link = true;
        child
    }

    pub(super) fn resolve_url(&self, raw: &str) -> String {
        self.base_url
            .as_ref()
            .map(|base| resolve_markdown_url(base.as_str(), raw))
            .unwrap_or_else(|| raw.to_string())
    }

    pub(super) fn push_text(&mut self, text: &str) {
        let mut text = normalize_inline_markdown(text);
        text = escape_markdown_text_backslashes(&text);
        if self.escape_snob {
            text = escape_markdown_snob_text(&text);
        }
        if is_markdown_line_start(&self.output) {
            text = escape_markdown_line_start(&text);
        }
        self.push_inline(&text);
    }

    pub(super) fn push_inline(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if needs_space_before_inline(&self.output) && !starts_with_closing_punctuation(text) {
            self.output.push(' ');
        }
        self.output.push_str(text);
    }

    pub(super) fn ensure_blank_line(&mut self) {
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

    pub(super) fn record_abbreviation(&mut self, text: String, title: String) {
        if text.is_empty() || title.is_empty() {
            return;
        }
        let mut abbreviations = self.abbreviations.borrow_mut();
        if let Some((_, existing_title)) = abbreviations
            .iter_mut()
            .find(|(existing_text, _)| existing_text == &text)
        {
            *existing_title = title;
        } else {
            abbreviations.push((text, title));
        }
    }

    pub(super) fn append_abbreviation_definitions(&mut self) {
        let abbreviations = self.abbreviations.borrow().clone();
        if abbreviations.is_empty() {
            return;
        }
        self.ensure_blank_line();
        for (text, title) in abbreviations {
            self.output.push_str("  *[");
            self.output.push_str(&text);
            self.output.push_str("]: ");
            self.output.push_str(&title);
            self.output.push('\n');
        }
    }
}
