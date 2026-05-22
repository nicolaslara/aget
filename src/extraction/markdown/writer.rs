use std::cell::RefCell;
use std::rc::Rc;

use url::Url;

use super::normalize::{
    escape_markdown_line_start, escape_markdown_snob_text, escape_markdown_text_backslashes,
    is_markdown_line_start, needs_space_before_inline, normalize_inline_markdown,
    normalize_unicode_snob_text, resolve_markdown_url, starts_with_closing_punctuation,
    trailing_newline_count, trim_trailing_horizontal_space,
};

pub(super) struct MarkdownWriter {
    pub(super) output: String,
    base_url: Option<Url>,
    default_image_alt: String,
    pub(super) open_quote: String,
    pub(super) close_quote: String,
    pub(super) ul_item_mark: String,
    pub(super) emphasis_mark: String,
    pub(super) strong_mark: String,
    pub(super) only_text: bool,
    pub(super) skip_internal_links: bool,
    pub(super) ignore_images: bool,
    pub(super) images_as_html: bool,
    pub(super) images_to_alt: bool,
    pub(super) images_with_size: bool,
    pub(super) ignore_emphasis: bool,
    pub(super) ignore_links: bool,
    pub(super) ignore_mailto_links: bool,
    pub(super) ignore_tables: bool,
    pub(super) bypass_tables: bool,
    pub(super) hide_strikethrough: bool,
    pub(super) protect_links: bool,
    pub(super) use_automatic_links: bool,
    unicode_snob: bool,
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
        default_image_alt: &str,
        open_quote: &str,
        close_quote: &str,
        ul_item_mark: &str,
        emphasis_mark: &str,
        strong_mark: &str,
        ignore_images: bool,
        images_as_html: bool,
        images_to_alt: bool,
        images_with_size: bool,
        ignore_emphasis: bool,
        ignore_links: bool,
        ignore_mailto_links: bool,
        ignore_tables: bool,
        bypass_tables: bool,
        hide_strikethrough: bool,
        protect_links: bool,
        use_automatic_links: bool,
        unicode_snob: bool,
        escape_snob: bool,
        include_sup_sub: bool,
    ) -> Self {
        Self {
            output: String::new(),
            base_url: Url::parse(base_url).ok(),
            default_image_alt: default_image_alt.to_string(),
            open_quote: open_quote.to_string(),
            close_quote: close_quote.to_string(),
            ul_item_mark: ul_item_mark.to_string(),
            emphasis_mark: emphasis_mark.to_string(),
            strong_mark: strong_mark.to_string(),
            only_text,
            skip_internal_links,
            ignore_images,
            images_as_html,
            images_to_alt,
            images_with_size,
            ignore_emphasis,
            ignore_links,
            ignore_mailto_links,
            ignore_tables,
            bypass_tables,
            hide_strikethrough,
            protect_links,
            use_automatic_links,
            unicode_snob,
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
            default_image_alt: self.default_image_alt.clone(),
            open_quote: self.open_quote.clone(),
            close_quote: self.close_quote.clone(),
            ul_item_mark: self.ul_item_mark.clone(),
            emphasis_mark: self.emphasis_mark.clone(),
            strong_mark: self.strong_mark.clone(),
            only_text: self.only_text,
            skip_internal_links: self.skip_internal_links,
            ignore_images: self.ignore_images,
            images_as_html: self.images_as_html,
            images_to_alt: self.images_to_alt,
            images_with_size: self.images_with_size,
            ignore_emphasis: self.ignore_emphasis,
            ignore_links: self.ignore_links,
            ignore_mailto_links: self.ignore_mailto_links,
            ignore_tables: self.ignore_tables,
            bypass_tables: self.bypass_tables,
            hide_strikethrough: self.hide_strikethrough,
            protect_links: self.protect_links,
            use_automatic_links: self.use_automatic_links,
            unicode_snob: self.unicode_snob,
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

    pub(super) fn image_alt<'a>(&'a self, raw: Option<&'a str>) -> &'a str {
        raw.filter(|value| !value.is_empty())
            .unwrap_or(&self.default_image_alt)
    }

    pub(super) fn push_text(&mut self, text: &str) {
        let mut text = normalize_inline_markdown(text);
        if !self.unicode_snob {
            text = normalize_unicode_snob_text(&text);
        }
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
