# D104-D108: Lists, Blockquotes, And Link Escaping

### D104: I19d uses Crawl4AI unordered list bullets

The next markdown-quality slice ports a small Crawl4AI/html2text formatting default. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.__init__` sets `self.ul_item_mark = "*"`, and `HTML2Text.handle_tag` emits that marker for unordered list items.

`OwnedExtractorBackend` now renders unordered markdown list items with `*` instead of `-`, including nested unordered lists. The static markdown parity fixture now covers top-level and nested unordered lists with the Crawl4AI/html2text marker while preserving ordered-list numbering and `crawl4ai.only_text` inline behavior.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is source-backed, deterministic, and limited to equivalent Markdown bullet syntax. It does not affect list structure, selectors, browser rendering, or session replay.

### D105: I19d preserves blockquote paragraph breaks

The next markdown-quality slice ports another Crawl4AI/html2text formatting behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` enters blockquote mode with a `> ` prefix and the output path prefixes subsequent lines while preserving paragraph breaks inside the quote.

`OwnedExtractorBackend` now renders blockquote children through the block markdown path before prefixing each resulting line with `>`. This preserves multiple paragraphs inside a blockquote instead of collapsing them through inline rendering. The static markdown parity fixture now covers a two-paragraph blockquote with inline strong text.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is source-backed and covered by a deterministic fixture. It is limited to blockquote markdown rendering and does not alter extraction selection, browser rendering, or session replay.

### D106: I19d escapes owned link and image markdown targets

The next markdown-quality slice ports Crawl4AI/html2text escaping inside markdown link/image constructs. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where link URLs and image alt/src values are passed through `escape_md`, and `references/repos/crawl4ai/crawl4ai/html2text/utils.py`, where `escape_md` backslash-escapes backslashes, square brackets, and parentheses.

`OwnedExtractorBackend` now escapes those characters in rendered link destinations, image destinations, and image alt text. The same fixture also caught and fixed an owned inline-spacing edge case where image markdown beginning with `![]` was incorrectly treated as sentence-closing punctuation and joined to the preceding word. The static markdown parity fixture now covers parenthesized URLs and image alt text containing brackets and parentheses.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is source-backed and covered by deterministic markdown output. It is limited to markdown escaping/spacing inside link and image constructs and does not alter selection, auth/session handling, or browser rendering.

### D107: I19d preserves empty markdown links

The next markdown-quality slice ports a small Crawl4AI/html2text anchor edge case. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` tracks `empty_link` and emits `[` before closing an otherwise empty inline link, resulting in `[](resolved-url)` instead of substituting the href as the label.

`OwnedExtractorBackend` now keeps the empty child-label case empty while retaining automatic-link rendering for non-empty absolute URL labels. The static markdown parity fixture covers an empty anchor alongside titled links, `mailto:` suppression, automatic absolute links, and escaped link/image targets.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is directly source-backed, covered by deterministic markdown output, and limited to link-label rendering.

### D108: I19d escapes markdown constructs in link titles

The next markdown-quality slice tightens link title escaping. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where inline link titles are passed through `escape_md` before being appended to the markdown link. `escape_md` backslash-escapes backslashes, square brackets, and parentheses.

`OwnedExtractorBackend` now applies the same Markdown-construct escaping to link titles while retaining its existing quote escaping for the quoted title delimiter. The static markdown parity fixture covers a link title containing quotes, square brackets, and parentheses.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is directly source-backed and deterministic, and the change is limited to rendered markdown link titles.
