use std::cell::RefCell;
use std::rc::Rc;

use url::Url;

use super::normalize::resolve_markdown_url;

mod references;
mod text;

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
    preserve_tags: Vec<String>,
    pub(super) handle_code_in_pre: bool,
    pub(super) ignore_emphasis: bool,
    pub(super) ignore_links: bool,
    pub(super) inline_links: bool,
    pub(super) links_each_paragraph: bool,
    pub(super) ignore_mailto_links: bool,
    pub(super) ignore_tables: bool,
    pub(super) bypass_tables: bool,
    pub(super) hide_strikethrough: bool,
    pub(super) google_doc: bool,
    pub(super) google_list_indent: usize,
    pub(super) protect_links: bool,
    pub(super) use_automatic_links: bool,
    unicode_snob: bool,
    escape_backslash: bool,
    escape_snob: bool,
    escape_dot: bool,
    escape_plus: bool,
    escape_dash: bool,
    pub(super) include_sup_sub: bool,
    pub(super) inside_link: bool,
    pub(super) list_depth: usize,
    reference_links: Rc<RefCell<Vec<ReferenceLink>>>,
    abbreviations: Rc<RefCell<Vec<(String, String)>>>,
}

#[derive(Clone)]
pub(super) struct ReferenceLink {
    href: String,
    title: String,
    emitted: bool,
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
        preserve_tags: &[String],
        handle_code_in_pre: bool,
        ignore_emphasis: bool,
        ignore_links: bool,
        inline_links: bool,
        links_each_paragraph: bool,
        ignore_mailto_links: bool,
        ignore_tables: bool,
        bypass_tables: bool,
        hide_strikethrough: bool,
        google_doc: bool,
        google_list_indent: usize,
        protect_links: bool,
        use_automatic_links: bool,
        unicode_snob: bool,
        escape_backslash: bool,
        escape_snob: bool,
        escape_dot: bool,
        escape_plus: bool,
        escape_dash: bool,
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
            preserve_tags: preserve_tags.to_vec(),
            handle_code_in_pre,
            ignore_emphasis,
            ignore_links,
            inline_links,
            links_each_paragraph,
            ignore_mailto_links,
            ignore_tables,
            bypass_tables,
            hide_strikethrough,
            google_doc,
            google_list_indent,
            protect_links,
            use_automatic_links,
            unicode_snob,
            escape_backslash,
            escape_snob,
            escape_dot,
            escape_plus,
            escape_dash,
            include_sup_sub,
            inside_link: false,
            list_depth: 0,
            reference_links: Rc::new(RefCell::new(Vec::new())),
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
            preserve_tags: self.preserve_tags.clone(),
            handle_code_in_pre: self.handle_code_in_pre,
            ignore_emphasis: self.ignore_emphasis,
            ignore_links: self.ignore_links,
            inline_links: self.inline_links,
            links_each_paragraph: self.links_each_paragraph,
            ignore_mailto_links: self.ignore_mailto_links,
            ignore_tables: self.ignore_tables,
            bypass_tables: self.bypass_tables,
            hide_strikethrough: self.hide_strikethrough,
            google_doc: self.google_doc,
            google_list_indent: self.google_list_indent,
            protect_links: self.protect_links,
            use_automatic_links: self.use_automatic_links,
            unicode_snob: self.unicode_snob,
            escape_backslash: self.escape_backslash,
            escape_snob: self.escape_snob,
            escape_dot: self.escape_dot,
            escape_plus: self.escape_plus,
            escape_dash: self.escape_dash,
            include_sup_sub: self.include_sup_sub,
            inside_link: self.inside_link,
            list_depth: self.list_depth,
            reference_links: Rc::clone(&self.reference_links),
            abbreviations: Rc::clone(&self.abbreviations),
        }
    }

    pub(super) fn link_child(&self) -> Self {
        let mut child = self.child();
        child.inside_link = true;
        child
    }

    pub(super) fn should_preserve_tag(&self, tag: &str) -> bool {
        self.preserve_tags
            .iter()
            .any(|preserve_tag| preserve_tag.eq_ignore_ascii_case(tag))
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
}
