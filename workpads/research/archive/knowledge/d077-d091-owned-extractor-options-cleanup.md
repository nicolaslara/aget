# Knowledge Archive D77-D122: Owned Extraction And Browser Backend Slices

### D77: I19d ports `crawl4ai.only_text` to owned markdown rendering

The next safe Crawl4AI option to port was `crawl4ai.only_text`. Before changing the owned extractor, I19d inspected the upstream behavior in `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `only_text` replaces text-formatting inline tags from `ONLY_TEXT_ELIGIBLE_TAGS` with their text content, and `references/repos/crawl4ai/crawl4ai/config.py`, where that tag allowlist is defined. The local command helper already parsed this option as a boolean.

The owned extractor now parses `crawl4ai.only_text` with the same boolean spelling set used by the command helper (`true/false`, `1/0`, `yes/no`, `on/off`). When enabled, owned markdown rendering treats Crawl4AI's text-formatting inline tags such as `strong`, `em`, `code`, `span`, `mark`, and `time` as plain text while preserving structural markdown such as headings, lists, links, tables, and preformatted code blocks. This mirrors the safe part of Crawl4AI's option without adding JavaScript execution or broader cleanup policy.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, and `delay_before_return_html`. `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This is a narrow renderer option with deterministic fixture coverage; it does not change broader readability or rendered-page readiness behavior.

### D78: I19d ports `crawl4ai.page_timeout` and `crawl4ai.wait_for_timeout` to owned CDP rendering

The next render-control slice ports the timeout options that have clear browser-operation semantics in Crawl4AI. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `page_timeout` is an integer millisecond timeout for page operations and `wait_for_timeout` optionally overrides the timeout used for `wait_for`, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where `wait_for_timeout` falls back to `page_timeout` when absent.

The owned extractor now parses `crawl4ai.page_timeout` and `crawl4ai.wait_for_timeout` as non-negative integer millisecond values. `page_timeout` is applied to CDP page creation, domain enabling, state loading, navigation, and final DOM reads in `browser_cdp::render_page`; browser process startup still uses the outer `aget --timeout` budget. `wait_for_timeout` applies only to the CSS selector wait and falls back to the page timeout when absent. The CSS-only wait safety rule remains unchanged.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, and `wait_for_timeout`. `word_count_threshold`, `wait_until`, and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`

Confidence: Medium-high. The parser is deterministically covered, and the ignored Chrome smoke covers the positive rendered wait path with explicit page and wait timeouts. This does not add network-idle or image-readiness behavior.

### D79: I19d ports bounded `crawl4ai.wait_until` support to owned CDP rendering

The next render-control slice ports the part of `crawl4ai.wait_until` that has direct owned CDP semantics. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where the default `wait_until` is `domcontentloaded`, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI passes that value into Playwright navigation.

The owned renderer now accepts `crawl4ai.wait_until=domcontentloaded` and `crawl4ai.wait_until=load`. These map to CDP `Page.domContentEventFired` and `Page.loadEventFired` respectively. Unsupported values such as `networkidle` fail explicitly instead of being ignored, because the owned renderer does not yet implement network-idle tracking. The default owned behavior remains the existing load-event wait until a broader readiness policy is chosen.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, `wait_for_timeout`, and bounded `wait_until`. `word_count_threshold` and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`

Confidence: Medium-high. This covers the two CDP lifecycle events the owned renderer can currently prove. It intentionally does not claim Playwright `networkidle` parity.

### D80: I19d ports `crawl4ai.wait_for_images` to owned CDP rendering

The next render-readiness slice ports the safe part of `crawl4ai.wait_for_images`. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `wait_for_images` is a boolean navigation/timing option defaulting to false, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI waits for `domcontentloaded`, sleeps briefly, then checks that all `<img>` elements are complete with a one-second timeout. Crawl4AI logs a warning and continues if images do not finish.

The owned extractor now parses `crawl4ai.wait_for_images` as a boolean using the same boolean spelling set as other owned options. When enabled, it forces the owned CDP rendering path, waits up to one second for `Array.from(document.images).every((img) => img.complete)`, and continues with an agent-visible warning if the image wait times out. This remains a browser-readiness option only; it does not add image description extraction, screenshot capture, external image fetching outside the browser, or JavaScript waits from user input.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, `wait_for_timeout`, bounded `wait_until`, and `wait_for_images`. `word_count_threshold` remained unsupported at this slice until source inspection established whether the current Crawl4AI default scraper applies it to the behavior `aget` actually depends on.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_honors_wait_for_images_option_with_chrome -- --ignored`

Confidence: Medium. The behavior is source-faithful for the explicit image-completion wait and covered by a local Chrome smoke, but broader rendered-page readiness remains a larger I19d gap.

### D81: I19d accepts `crawl4ai.word_count_threshold` with current Crawl4AI default semantics

The last Crawl4AI helper option still rejected by the owned extractor was `crawl4ai.word_count_threshold`. Before changing the owned backend, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `CrawlerRunConfig` accepts and stores `word_count_threshold`, and `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where the default `LXMLWebScrapingStrategy._scrap` receives that parameter but the cleaned-content path currently calls `remove_empty_elements_fast(body, 1)` with a hardcoded threshold. The upstream regression tests also cover config serialization/defaults and browser-context reuse for varying `word_count_threshold`, not output pruning in the default markdown path.

The owned extractor now accepts and validates `crawl4ai.word_count_threshold` as an integer so existing command-helper callers can switch to the owned backend without hitting an unsupported-option failure. It intentionally does not use the value to prune output because that would be stricter than the inspected Crawl4AI default path. Unsupported backend options still fail explicitly.

At this point the owned backend accepts every namespaced Crawl4AI option that the PoC command helper allowed: `excluded_tags`, `target_elements`, `only_text`, `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, `delay_before_return_html`, and `wait_for_images`.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This closes the compatibility-option gap without inventing behavior upstream does not currently prove; the remaining I19d gaps are broader quality/readiness work rather than a helper-option mismatch.

### D82: I19d ports bounded `crawl4ai.wait_until=networkidle` support to owned CDP rendering

The next rendered-readiness slice completes the safe `wait_until` value set that the PoC helper exposed. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `CrawlerRunConfig.wait_until` is the navigation wait condition, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI passes that value into Playwright navigation before later image waits or HTML capture.

The owned CDP renderer now accepts `crawl4ai.wait_until=networkidle`. It sends `Page.navigate` without discarding interleaved CDP events, requires the navigation response plus `Page.domContentEventFired`, tracks same-target `Network.requestWillBeSent`, `Network.loadingFinished`, and `Network.loadingFailed` events, and considers the page idle after there are no in-flight tracked requests for 500 ms. This is a bounded CDP implementation of the Playwright concept, not a broader smart-readiness system: long-polling, websockets, service-worker behavior, virtual scrolling, and app-specific readiness still belong to later I19d work.

The supported owned `crawl4ai.wait_until` values are now `domcontentloaded`, `load`, and `networkidle`.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_honors_networkidle_wait_until_with_chrome -- --ignored`

Confidence: Medium. The local Chrome smoke proves the owned network-idle wait for a delayed same-origin fetch, but this should still be treated as bounded parity rather than full Playwright readiness equivalence.

### D83: I19e classifies Chrome profile-in-use startup as user action

The next owned browser/session parity slice tightens Chrome startup diagnostics for profile imports. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`, where `agent-browser` captures Chrome startup stderr, reports early `DevToolsActivePort` failures with stderr context, copies named profiles into temporary user-data-dir roots, and treats profile reuse/lock situations as launch failures that a caller can surface to the user.

`OwnedBrowserAutomationBackend` now captures Chrome stderr in a private temp file during CDP startup. If Chrome exits before writing `DevToolsActivePort` and stderr indicates a profile-in-use or singleton/lock condition, the owned backend returns `requires_user_action` instead of a generic backend failure. Non-user-action startup failures still keep their original error code but include relevant Chrome stderr lines for diagnosis. This improves owned Chrome/profile import parity without touching the user's running browser or adding ambient current-browser access.

Validation:

- `cargo fmt --check`
- `cargo test owned_session_import_chrome_classifies_profile_in_use_as_requires_user_action --test session_cli`
- `cargo test browser_cdp::tests::parses_devtools_active_port_file`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored`

Confidence: Medium-high. The new deterministic CLI test proves the classification and no-session-saved behavior with a fake Chrome executable. Real Chrome lock/keychain behavior still needs the ignored/manual smoke coverage tracked under I19e/I19h.

### D84: I19e covers scripted owned browser fallback rendering

The next I19e verification slice covers an `agent-browser` parity behavior without adding new production surface. The command-backed fallback path in `src/extraction.rs` loads composed state into a browser session, opens the requested URL, then reads body HTML/text from the rendered page. The upstream `agent-browser` navigation path in `references/repos/agent-browser/cli/src/native/browser.rs` waits for a CDP lifecycle event before callers request page content.

`tests/mock_site_cli.rs` now includes an ignored local-Chrome smoke proving the owned browser fallback renders a cookie-backed script-bearing page after the primary extractor fails. This specifically exercises the fallback path rather than the default owned primary extractor, and verifies the session cookie is replayed to the scripted page.

Validation:

- `cargo test --test mock_site_cli owned_browser_fallback_renders_cookie_backed_scripted_page_with_chrome -- --ignored`

Confidence: Medium-high for this parity slice. The smoke uses local Chrome and a deterministic local site, but broader SPA readiness, current-tab attach, and real logged-in profile/keychain behavior remain open.

### D85: I19e preserves owned browser sessionStorage state

The next I19e state-parity slice ports the sessionStorage behavior that was still only partially represented in `aget`. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/state.rs`, where `StorageState` includes per-origin `sessionStorage`, state export collects `sessionStorage` through `Runtime.evaluate`, and state load navigates to each origin before calling `sessionStorage.setItem(...)`.

`aget` now keeps sessionStorage as first-class scoped session state. `SessionOrigin` stores it beside localStorage, agent-browser/Playwright-state imports preserve it through the same allowlist filtering, composed state merges it with duplicate-key conflict checks, owned CDP fallback loading replays it after navigating to the origin, and CDP state export collects origins that contain either localStorage or sessionStorage. Live owned-login export now attaches to an existing non-internal page target before state export instead of creating a fresh tab, because sessionStorage is tab-scoped and would otherwise be lost. Normal session inspection and extraction redaction now treat sessionStorage values as credential-equivalent bearer material, while `--show-secrets` can still reveal them intentionally.

Validation:

- `cargo check`
- `cargo test session_storage`
- `cargo test --test session_cli session_import_chrome_saves_filtered_state_and_cleans_raw_file`
- `cargo test browser_cdp::tests::parses_origin_storage_runtime_value`
- `cargo test browser_cdp::tests::prefers_existing_non_internal_page_target`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored`
- `cargo test browser_cdp::tests::owned_login_browser_exports_state_from_headed_profile_and_closes -- --ignored`

Confidence: Medium-high. Deterministic tests cover import filtering, redaction, composition, JS expression quoting, target selection, and CDP result parsing. The local-Chrome ignored smokes cover persistent profile export plus live headed-login sessionStorage export, but current-tab attach and broader cross-platform browser-process behavior remain I19e follow-ups.

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

