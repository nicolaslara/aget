use super::{MarkdownWriter, ReferenceLink};

impl MarkdownWriter {
    pub(in crate::extraction::markdown) fn record_abbreviation(
        &mut self,
        text: String,
        title: String,
    ) {
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

    pub(in crate::extraction::markdown) fn reference_link(
        &mut self,
        href: &str,
        title: &str,
    ) -> usize {
        let href = self.resolve_url(href);
        let title = title.trim().to_string();
        let mut reference_links = self.reference_links.borrow_mut();
        if let Some((index, _)) = reference_links.iter().enumerate().find(|(_, link)| {
            link.href == href
                && link.title == title
                && (!self.links_each_paragraph || !link.emitted)
        }) {
            return index + 1;
        }
        reference_links.push(ReferenceLink {
            href,
            title,
            emitted: false,
        });
        reference_links.len()
    }

    pub(in crate::extraction::markdown) fn append_reference_link_definitions(&mut self) {
        self.append_pending_reference_link_definitions();
    }

    pub(in crate::extraction::markdown) fn append_paragraph_reference_link_definitions(&mut self) {
        if !self.links_each_paragraph {
            return;
        }
        self.append_pending_reference_link_definitions();
    }

    fn append_pending_reference_link_definitions(&mut self) {
        let pending = {
            let reference_links = self.reference_links.borrow();
            reference_links
                .iter()
                .enumerate()
                .filter(|(_, link)| !link.emitted)
                .map(|(index, link)| (index + 1, link.href.clone(), link.title.clone()))
                .collect::<Vec<_>>()
        };
        if pending.is_empty() {
            return;
        }
        self.ensure_blank_line();
        for (index, href, title) in pending {
            self.output.push_str("   [");
            self.output.push_str(&index.to_string());
            self.output.push_str("]: ");
            self.output.push_str(&href);
            if !title.is_empty() {
                self.output.push_str(" (");
                self.output.push_str(&title);
                self.output.push(')');
            }
            self.output.push('\n');
        }
        let mut reference_links = self.reference_links.borrow_mut();
        for link in reference_links.iter_mut() {
            link.emitted = true;
        }
    }

    pub(in crate::extraction::markdown) fn append_abbreviation_definitions(&mut self) {
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
