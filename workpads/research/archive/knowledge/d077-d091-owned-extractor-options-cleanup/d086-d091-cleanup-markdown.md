# D86-D91: Cleanup And Markdown

### D86: I19d matches Crawl4AI selector miss fallback

The next owned extraction parity slice tightens `css_selector` behavior. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `LXMLWebScrapingStrategy._scrap` builds a selected content wrapper when `css_selector` matches but falls back to the full parsed document when the selector has no matches or errors.

`OwnedExtractorBackend` now preserves that no-match behavior for `GetOptions.selector`: a valid selector with no matches falls back to the full document instead of returning `extraction_failed`. This is intentionally different from the default no-selector path, which still uses `aget`'s conservative main-content heuristic, and from `wait_for_selector`, which still must fail when the waited element is absent. The fixture verifies this by selecting a missing class on a page that has `main` plus header/footer; the result includes header and footer rather than using main-content cleanup.

Validation:

- `cargo fmt --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. The behavior is source-faithful for the concrete selector miss case and deterministically covered. Invalid selector handling and broader Crawl4AI cleaned-HTML/readability behavior remain separate I19d follow-ups.

### D87: I19d aligns owned cleaned-HTML tag removal

The next I19d output-shaping slice ports a small part of Crawl4AI's cleaned HTML contract. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `LXMLWebScrapingStrategy._scrap` removes `style`, `link`, `meta`, and `noscript` elements, then removes `script` elements, before serializing cleaned HTML.

`OwnedExtractorBackend` already removed `script`, `style`, and `noscript`; it now also removes `link` and `meta` before generating HTML/text/markdown/json output. This primarily affects `--content-format html`, where previously head/body metadata and preload/canonical links could leak into the cleaned output even though Crawl4AI would drop them. The fixture keeps `<title>` and visible body content while proving `meta`, `link`, `style`, `script`, and `noscript` are absent from owned HTML output.

Validation:

- `cargo fmt --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: High for this narrow cleanup slice. The behavior is directly source-backed and deterministically covered; broader media/link extraction metadata and full readability remain separate I19d work.

### D88: I19d prunes owned cleaned-HTML attributes like Crawl4AI

The next cleaned-output slice ports Crawl4AI's default attribute pruning. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `remove_unwanted_attributes_fast` clears every element's attributes except an important-attribute allowlist and keeps `data-*` only when `keep_data_attributes` is enabled, and `references/repos/crawl4ai/crawl4ai/config.py`, where `IMPORTANT_ATTRS` is `src`, `href`, `alt`, `title`, `width`, `height`, `class`, and `id`.

`OwnedExtractorBackend` now records the selected root/target element IDs before cleanup, then strips non-important attributes before serializing cleaned output. This preserves selector and `crawl4ai.target_elements` matching against original page attributes while making HTML output drop `data-*`, inline style, event handler, ARIA, and relation attributes by default. The fixture proves the important attributes remain on cleaned output, unwanted attributes are absent, and a selector can still match a `data-*` attribute that is later pruned from the serialized content.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for this attribute-pruning slice. The allowlist is source-backed, selection-before-cleanup is covered by a deterministic fixture, and no new backend option or authenticated-browser behavior was added.

### D89: I19d strips base64 image payloads from owned cleaned output

The next cleaned-output slice ports Crawl4AI's base64 image cleanup. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `LXMLWebScrapingStrategy` compiles `BASE64_PATTERN = data:image/[^;]+;base64,...` and, before empty-element and attribute cleanup, replaces matching `<img src="...">` payloads with an empty `src`.

`OwnedExtractorBackend` now blanks `src` on image elements whose value starts with the same `data:image/<mime>;base64,` shape before serialized cleaned output is produced. The markdown renderer also skips images whose cleaned `src` is empty, which prevents the owned URL resolver from turning an emptied image source into a page-URL image reference. The fixture proves base64 payload text is absent from HTML output and does not reappear in markdown image syntax.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for this narrow privacy/output-size slice. The behavior is source-backed, deterministic, and only removes inline image payloads from output; it does not fetch or interpret images.

### D90: I19d removes empty owned cleaned-HTML leaf elements

The next cleaned-output slice ports Crawl4AI's empty-element pruning. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `remove_empty_elements_fast(root, 1)` walks descendants bottom-up after base64 image cleanup and before attribute pruning, removes childless elements with no words, skips a bypass tag set such as `a`, `img`, `br`, table cells/rows, and preserves whitespace-only descendants inside `pre`/`code`.

`OwnedExtractorBackend` now runs a bottom-up cleanup pass in the same order. It removes empty leaf elements, recomputing childlessness after earlier removals so empty wrappers can also disappear. It protects the selected root and `crawl4ai.target_elements` IDs so selector-driven extraction cannot fail by deleting the element it is about to serialize. The fixture proves empty wrapper/span elements are removed while empty anchors, breaks, table cells/rows, and whitespace-only code spans are kept.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and deterministic, with one deliberate guard: selected roots and target elements are preserved to keep the owned extractor's public selector contract stable.

### D91: I19d expands owned markdown tags from Crawl4AI CustomHTML2Text

The next markdown-quality slice ports a small source-backed subset of Crawl4AI's HTML-to-markdown behavior. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`, where the default generator feeds cleaned HTML into `CustomHTML2Text` with links/images/emphasis/code enabled, and `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text`/`CustomHTML2Text` handle horizontal rules, definition lists, strikethrough tags, quoted inline text, and `kbd`/`tt`/`code` as inline code.

`OwnedExtractorBackend` now renders `<hr>` as a markdown horizontal rule, `<dl>/<dt>/<dd>` as term lines with indented definitions, `<del>/<strike>/<s>` as strikethrough, `<kbd>/<tt>` as inline code, and `<q>` with quotes. `crawl4ai.only_text=true` still strips these inline decorations to plain text, matching the owned option's current contract.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium-high. This is deterministic local markdown rendering backed by Crawl4AI's source behavior. It remains a bounded quality slice; nested list fidelity, richer readability scoring, and broader rendered-page readiness are still open I19d work.
