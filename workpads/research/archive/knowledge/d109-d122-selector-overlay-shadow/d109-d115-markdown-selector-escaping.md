# D109-D115: Markdown, Selector, And Escaping

### D109: I19d preserves markdown hard breaks for `<br>`

The next markdown-quality slice ports Crawl4AI/html2text line-break behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` emits `  \n` for a starting `<br>` tag, with a blockquote-specific `> ` prefix variant.

`OwnedExtractorBackend` now emits Markdown hard breaks for `<br>` and keeps those two trailing spaces through final markdown normalization. The static markdown parity fixture covers this in both normal markdown and `crawl4ai.only_text=true` output. Blockquote prefixing is already handled by the owned blockquote renderer, so the same hard-break line is preserved before quote-line prefixing.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is directly source-backed, covered by deterministic fixture output, and limited to Markdown line-break preservation.

### D110: I19d preserves all CSS selector matches

The next extraction-behavior slice ports a Crawl4AI selector contract rather than another markdown tag edge case. Before changing the owned extractor, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `LXMLWebScrapingStrategy._scrap` calls `body.cssselect(css_selector)`, wraps all selected elements in a temporary `<div>`, and then applies `target_elements` inside that selected wrapper. The upstream regression `references/repos/crawl4ai/tests/test_issue_1484_css_selector.py` covers both multiple `css_selector` matches and `css_selector` combined with `target_elements`.

`OwnedExtractorBackend` now collects every element matched by `--selector` instead of only the first match. When `crawl4ai.target_elements` is also set, the owned extractor applies each target selector within every selected root before rendering. No-match fallback still uses the full document, preserving D86.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is source-backed, part of the user-facing selector contract, and covered by deterministic tests for all-match selection and selector-scoped target elements.

### D111: I19d preserves selected wrappers in multi-element HTML output

The D110 selector change exposed an adjacent cleaned-HTML parity detail. Crawl4AI's `LXMLWebScrapingStrategy._scrap` copies selected elements into a temporary wrapper and serializes `content_element` with `lhtml.tostring(...)`, so selected element tags are retained in `cleaned_html` rather than returning only their children.

`OwnedExtractorBackend` now serializes multi-element selections and target-element selections with each selected element's outer HTML. Single-root extraction keeps the existing inner-HTML behavior so unselected full-page output remains stable. The deterministic selector fixture verifies that multi-match HTML output keeps `<section class="result">` wrappers and excludes unselected sidebar content.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is source-backed and covered for the multi-match selector case; exact pretty-print wrapper formatting remains intentionally simpler than Crawl4AI's lxml serialization.

### D112: I19d tolerates invalid include/exclude selectors like Crawl4AI

The next selector-behavior slice ports Crawl4AI's invalid-selector handling. Before changing the owned extractor, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `LXMLWebScrapingStrategy._scrap` catches exceptions from `body.cssselect(css_selector)` and falls back to the full body, and also catches exceptions from `body.cssselect(excluded_selector)` and continues without removing anything.

`OwnedExtractorBackend` now treats an invalid normal `--selector` the same as a no-match selector by falling back to the full parsed document, and treats an invalid `--exclude-selector` as a no-op. Strict parsing is still retained for `--wait-for-selector` and `crawl4ai.target_elements`, because waits must be actionable and D71 intentionally validates target selectors before fetch/rendering.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is source-backed and covered by deterministic tests for both invalid include and exclude selector cases. It is limited to Crawl4AI-compatible selector tolerance and does not relax JavaScript wait safety.

### D113: I19d suppresses fragment-only markdown links

The next markdown link-default slice ports Crawl4AI/html2text's internal-link behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/config.py`, where `SKIP_INTERNAL_LINKS` defaults to true, and `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where an anchor is not pushed onto the link stack when `href` starts with `#`.

`OwnedExtractorBackend` now renders fragment-only anchors such as `<a href="#details">within page</a>` as plain child text instead of resolving them against the page URL. This matches the existing owned `mailto:` suppression path and keeps markdown output focused on fetchable external/page URLs rather than in-document targets.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for fragment-only anchors. The behavior is source-backed and covered by the existing deterministic link-default fixture; broader Crawl4AI link/reference formatting remains separate I19d work.

### D114: I19d escapes accidental list markers in plain text

The next markdown text-escaping slice ports Crawl4AI/html2text's default protection against plain text being misread as markdown lists. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/utils.py`, where `escape_md_section` escapes ordered-list dots plus leading `+` and `-` markers when they appear at the start of a markdown section, and `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`, where the default generator leaves `escape_snob` false but does not disable those marker-specific escapes.

`OwnedExtractorBackend` now escapes plain text that begins a markdown line with an ordered-list marker like `1. `, a dash bullet marker, or a plus bullet marker. Generated list syntax is unchanged because the escaping only applies to raw text nodes rendered at the start of a markdown line. The static markdown fixture now proves those plain-text markers remain text instead of becoming unintended markdown list items.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for start-of-line marker protection. The behavior is source-backed and deterministically covered; broader html2text escaping such as backslash preservation remains a separate markdown-quality follow-up.

### D115: I19d preserves literal backslashes before Markdown constructs

The next markdown text-escaping slice ports another Crawl4AI/html2text plain-text default. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/utils.py`, where `escape_md_section` first applies `RE_MD_BACKSLASH_MATCHER` to double a literal backslash when it precedes Markdown-sensitive characters, and `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `handle_data` calls `escape_md_section` for non-code, non-pre text with that default enabled.

`OwnedExtractorBackend` now doubles literal backslashes in raw text nodes when they precede Markdown-sensitive characters such as `*`, `[`, or `]`. This preserves source text like `\*stars\*` as literal backslash-plus-marker text instead of allowing Markdown parsing to consume the backslash as only an escape. Generated Markdown constructs are unchanged because the escaping is only applied in the raw text path.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for text-node backslash preservation. The behavior is source-backed and covered in the deterministic markdown fixture; this intentionally does not alter generated links, images, emphasis, code, or tables.
