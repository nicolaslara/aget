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

### D116: I19e retries owned Chrome launch startup failures

The next browser lifecycle slice ports `agent-browser`'s Chrome launch retry behavior. Before changing the owned browser backend, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` at local commit `3bb1d43`, where `launch_chrome` retries `try_launch_chrome` up to three times and waits 500ms between failed attempts before returning the last startup error.

`OwnedBrowserAutomationBackend` now applies the same three-attempt, 500ms retry policy when launching local Chrome for fallback rendering, Chrome profile import, and dedicated login browsers. Each attempt still removes stale `DevToolsActivePort`, captures fresh stderr, uses the existing process-group cleanup path on failure, and preserves the final startup classification/sandbox hints when all attempts fail.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test chrome_launch_retries_after_early_startup_exit`
- `cargo test`

Confidence: High for transient early-exit retry behavior. The behavior is source-backed and covered by a deterministic fake-Chrome test that fails the first launch, writes `DevToolsActivePort` on the second launch, and proves the owned backend returns the discovered CDP URL. Broader real-Chrome/keychain smoke coverage remains separate I19e work.

### D117: I19e removes stale DevToolsActivePort files after failed attach

The next existing-profile attach slice ports `agent-browser`'s stale CDP runtime-file cleanup. Before changing the owned browser backend, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` at local commit `3bb1d43`, where `auto_connect_cdp` reads `DevToolsActivePort`, tries to resolve the live CDP endpoint, and removes the file when the port is dead so future discovery skips stale state.

`OwnedBrowserAutomationBackend` now removes `DevToolsActivePort` from an owned/dedicated profile when `connect_existing_profile_browser` cannot connect through the exact WebSocket path or the `/json/version`, `/json/list`, and direct `/devtools/browser` discovery fallbacks. Non-CDP errors still propagate normally; only a dead/unavailable endpoint is treated as stale attach state.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test existing_profile_attach_removes_stale_devtools_active_port`
- `cargo test`

Confidence: High for dead-port stale-file cleanup. The behavior is source-backed and covered by a deterministic closed-port test; broader current-tab discovery and real-profile attach UX remain separate I19e work.

### D118: I19e adds silent Chrome startup diagnostics

The next startup-classification slice ports `agent-browser`'s no-stderr Chrome launch hint. Before changing the owned browser backend, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` at local commit `3bb1d43`, where `chrome_launch_error` adds an explicit no-stderr diagnostic and sandbox hint when Chrome exits before reporting a DevTools URL without producing stderr lines.

`OwnedBrowserAutomationBackend` now appends a no-stderr startup hint when Chrome exits or times out before CDP startup and the captured stderr file is empty. Existing profile-lock `requires_user_action` classification, relevant stderr lines, and sandbox/namespace hints still take precedence when diagnostic output exists.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test chrome_startup_error_adds_silent_exit_hint_without_stderr`
- `cargo test`

Confidence: High for silent-startup classification. The behavior is source-backed and covered at the classifier boundary; platform-specific real Chrome crashes still need opt-in smoke coverage.

### D119: I19d removes generic overlays before owned extraction

The next extraction-cleanup slice ports behavior that `aget` currently requested from Crawl4AI through `scripts/crawl4ai_extract.py`: `CrawlerRunConfig(remove_overlay_elements=True)`. Before changing the owned extractor, I19d re-inspected `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py` and `references/repos/crawl4ai/crawl4ai/js_snippet/remove_overlay_elements.js` at local commit `1debe5f`. Crawl4AI removes generic popup/modal/cookie overlay elements before capturing HTML; the JS snippet includes generic close-button, cookie-banner/consent, newsletter/subscribe, popup/modal/overlay/dialog, and dialog-role selectors.

`OwnedExtractorBackend` now applies the same generic selector cleanup before selector/exclusion extraction and before HTML/markdown/text serialization. This is intentionally generic and not site-specific: it removes DOM elements matching broad overlay/modal/cookie/dialog patterns, but it does not add built-in site names or paywall/login handling.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for generic selector-backed overlay removal. The behavior is source-backed and covered by a deterministic fixture that places cookie-banner and dialog-role elements inside the selected `<main>` and verifies both text and HTML outputs remove them. Style/z-index-based overlay removal from Crawl4AI's browser JS remains a rendered-page parity follow-up.

### D120: I19d removes rendered style overlays before CDP HTML capture

The next rendered-page cleanup slice ports the style/computed-layout side of Crawl4AI's `remove_overlay_elements` behavior. Before changing the owned CDP renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/js_snippet/remove_overlay_elements.js` at local commit `1debe5f`. The upstream snippet clicks generic close/dismiss buttons, removes visible high-z-index/fixed/absolute overlay-like elements, removes elements matching generic popup/modal/cookie/dialog selectors, removes fixed/sticky elements, and resets body modal padding/overflow before HTML capture.

`OwnedExtractorBackend` now asks the owned CDP renderer to run a generic overlay cleanup script after navigation/waits/render-settle and before reading `document.documentElement.outerHTML`. The script is intentionally generic: it includes broad close/cookie/newsletter/popup/modal/overlay/dialog selectors plus computed style checks for high z-index, fixed/absolute positioning, overlay-like size/background/opacity, and fixed/sticky chrome. Cleanup failure is reported as an extraction warning rather than failing the whole fetch, so an overlay-cleanup regression does not turn an otherwise fetchable page into a hard error.

Validation:

- `cargo fmt --check`
- `cargo test rendered_overlay_cleanup_expression_uses_generic_crawl4ai_rules`
- `cargo test --test mock_site_cli owned_extractor_backend_removes_rendered_style_overlays_with_chrome -- --ignored`
- `git diff --check`
- `cargo test`

Confidence: High for rendered style-overlay cleanup on this machine. The behavior is source-backed, the script contract is covered by a deterministic unit test, and a local Chrome ignored smoke proves a style-only fixed overlay is removed before text extraction. Broader Crawl4AI-quality markdown/readability and richer rendered-readiness heuristics remain separate I19d work.

### D121: I19d preserves linked image markdown

The next markdown-quality slice ports a Crawl4AI/html2text image-inside-link behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` opens a link label before an `<img>` child and emits image markdown before the anchor close renders the outer link target. With inline links enabled by default, an anchor wrapping a single image becomes linked-image markdown such as `[![alt](image)](href)`.

`OwnedExtractorBackend` now detects the narrow single-image-anchor case and renders it as linked-image markdown instead of escaping the image markdown into the link label text. Mixed text/image anchors continue through the existing generic link path until a broader html2text inline-label model is justified.

Validation:

- `cargo fmt --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `git diff --check`
- `cargo test`

Confidence: High for single-image anchors. The behavior is source-backed and covered by the static markdown parity fixture; broader mixed inline link-image formatting remains a separate markdown-quality follow-up.

### D122: I19d supports optional shadow DOM flattening in owned CDP rendering

The next rendered-readiness slice ports Crawl4AI's opt-in shadow DOM flattening behavior. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `CrawlerRunConfig.flatten_shadow_dom` defaults to false; `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI injects an `attachShadow` override before page work and evaluates `js_snippet/flatten_shadow_dom.js` instead of normal page content capture when the option is enabled; and `references/repos/crawl4ai/crawl4ai/js_snippet/flatten_shadow_dom.js`, which serializes shadow roots, resolves slots, skips shadow-scoped styles, and falls back to normal capture when flattening returns no content.

`OwnedExtractorBackend` now accepts `crawl4ai.flatten_shadow_dom=true` and passes it to the owned CDP renderer. The renderer injects an `attachShadow` override before navigation so newly-created closed roots become open in the controlled, temporary browser, then uses a shadow-aware serializer before HTML capture. The compatibility helper also accepts the same namespaced option and forwards it to Crawl4AI when the installed version supports it. The option remains false by default to match Crawl4AI and avoid changing ordinary rendered fetches.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test shadow_dom_flatten_expression_resolves_slots_and_skips_styles`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_flattens_shadow_dom_with_chrome -- --ignored`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and covered by a deterministic script-contract test plus a local Chrome smoke that verifies projected shadow DOM text reaches extraction. It remains opt-in, and broader rendered-readiness behaviors such as virtual scrolling, app-specific readiness, and full Crawl4AI readability quality remain separate I19d work.
