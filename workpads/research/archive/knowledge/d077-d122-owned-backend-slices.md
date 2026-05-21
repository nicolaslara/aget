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

### D92: I19d preserves nested list structure in owned markdown

The next markdown-quality slice ports Crawl4AI's depth-aware list behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` maintains a list stack, emits ordered or unordered list markers for each `<li>`, and indents nested lists according to list nesting. `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` still confirms this path is the default markdown generator for cleaned HTML.

`OwnedExtractorBackend` now tracks markdown list depth while rendering `<ol>` and `<ul>`, preserving nested list structure instead of flattening child list items to top-level markers. The deterministic fixture covers an ordered list containing an unordered child list and verifies the existing simple list output still passes.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium. The behavior is source-inspired and deterministic, but it intentionally implements a compact CommonMark-readable indentation rather than claiming byte-for-byte `html2text` whitespace parity for every nested-list shape.

### D93: I19d aligns owned markdown link title and mailto defaults

The next markdown-quality slice ports Crawl4AI's default link behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`, where the default generator uses inline links, and `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `CustomHTML2Text` sets `ignore_mailto_links = True` and `HTML2Text.handle_tag` appends an escaped title string to inline links when the `<a>` has a non-empty `title`.

`OwnedExtractorBackend` now includes link titles in markdown as `[label](url "title")` and renders `mailto:` anchors as plain child text rather than clickable links. This improves markdown fidelity without changing URL fetching, session replay, or the generic safety boundary.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and deterministically covered for escaped title text plus default `mailto:` suppression. It does not claim full `html2text` link-reference or automatic-link parity.

### D94: I19d emits automatic absolute links in owned markdown

The next markdown-quality slice ports Crawl4AI/html2text automatic-link behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/config.py`, where `USE_AUTOMATIC_LINKS` defaults to true, and `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_data` emits `<absolute-url>` when the link text exactly matches the absolute URL and automatic links are enabled.

`OwnedExtractorBackend` now renders an untitled HTTP(S) anchor whose label exactly equals its href as `<https://...>` instead of `[https://...](https://...)`. Other links still use normal inline markdown with resolved base URLs and optional titles.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and deterministically covered for the absolute HTTP(S) case. The owned renderer intentionally keeps non-HTTP schemes and titled links on the existing explicit-link path.

### D95: I19d preserves table captions in owned markdown

The next markdown-quality slice ports Crawl4AI/html2text table-caption behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` emits caption text and a soft break before table rows when it sees `<caption>`.

`OwnedExtractorBackend` now preserves the first table caption ahead of the generated GFM table. This closes a real content-loss case in the owned markdown renderer: previously captions were dropped because table rendering only collected `tr` rows. The deterministic mock-site fixture now verifies captions in normal markdown and `crawl4ai.only_text=true` markdown.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and deterministic. It intentionally handles the primary caption text case without trying to reproduce every upstream whitespace variant for unusual nested table/caption shapes.

### D96: I19e uses Chrome stderr as a CDP startup fallback

The next owned browser/session parity slice ports a small but important Chrome startup behavior from `agent-browser`. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`, where Chrome launch first waits for `DevToolsActivePort` and then falls back to parsing stderr for a `DevTools listening on ...` WebSocket URL when the active-port file path is unavailable.

`OwnedBrowserAutomationBackend` now keeps the existing `DevToolsActivePort` startup path as primary, but also polls the private Chrome stderr capture for a `DevTools listening on ws://...` or `wss://...` URL before treating startup as failed. This makes owned Chrome startup more tolerant of platform-specific active-port behavior while preserving the existing private stderr capture and startup error classification.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and deterministically tested with a fake Chrome child process, without requiring local Chrome. Broader current-tab attach, real logged-in profile/keychain smoke coverage, and cross-platform process lifecycle checks remain open I19e work.

### D97: I19e adds sandbox startup diagnostics for owned Chrome

The next owned Chrome startup slice reuses the same `agent-browser` source path inspected for D96: `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`. Its `chrome_launch_error` helper promotes Chrome stderr lines containing `sandbox` or `namespace` and adds an explicit container/VM hint instead of returning an opaque early-exit failure.

`OwnedBrowserAutomationBackend` now appends an `AGET_CHROME_COMMAND`-oriented sandbox/namespace hint when Chrome startup stderr indicates a sandbox or namespace failure. The error code is unchanged, and profile-lock/user-action classification remains separate; this only makes non-user-action startup failures more actionable for local Chrome and CI/container environments.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::chrome_stderr_detail_includes_sandbox_hint`
- `cargo test --test session_cli owned_session_import_chrome_reports_sandbox_startup_hint`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and covered by deterministic helper and CLI tests using a fake Chrome executable. It still does not prove every platform-specific Chrome startup failure shape, so fuller cross-platform process/error classification remains an I19e follow-up.

### D98: I19d aligns owned emphasis markdown markers

The next markdown-quality slice ports a small Crawl4AI/html2text formatting default. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.__init__` sets `self.emphasis_mark = "_"`, and `references/repos/crawl4ai/crawl4ai/html2text/config.py`, where inline markdown output is the default for links and images.

`OwnedExtractorBackend` now renders `<em>` and `<i>` text with underscore emphasis (`_text_`) instead of asterisk emphasis (`*text*`). The static markdown parity fixture now includes an emphasized phrase alongside the existing deletion, inline-code, quote, horizontal-rule, and definition-list cases.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is source-backed, deterministic, and limited to equivalent Markdown emphasis syntax. It does not affect strong emphasis, links, tables, session replay, or browser rendering.

### D99: I19d treats underline tags as emphasis

The next markdown-quality slice ports another Crawl4AI/html2text inline-tag behavior that survives the default cleaned-HTML pipeline. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` treats `em`, `i`, and `u` the same when emphasis is enabled, and `HTML2Text.__init__` sets the emphasis marker to `_`.

`OwnedExtractorBackend` now renders `<u>` with underscore emphasis, matching the already-owned `<em>`/`<i>` marker behavior. The static markdown parity fixture now covers underlined text beside deletion, emphasis, inline-code, quote, horizontal-rule, and definition-list cases.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High. The behavior is source-backed, deterministic, and limited to equivalent Markdown emphasis syntax for one inline tag. It does not affect strong emphasis, links, tables, session replay, or browser rendering.

### D100: I19d preserves abbreviation title definitions

The next markdown-quality slice ports a Crawl4AI/html2text content-preservation behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` records `<abbr title="...">` text and `finish()` emits markdown abbreviation definitions at the end of the document. This survives Crawl4AI's default cleaned-HTML pipeline because `title` is in the important-attribute allowlist.

`OwnedExtractorBackend` now records abbreviation text/title pairs while rendering markdown and appends markdown definition lines such as `  *[HTML]: HyperText Markup Language` at the end of the output. Duplicate abbreviation text updates the stored title, matching html2text's dictionary-shaped behavior. Plain-text mode still emits only visible text.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and deterministic for the primary abbreviation-title case. It does not attempt to reproduce every whitespace variant in html2text's final reference block.

### D101: I19e falls back to CDP HTTP discovery for existing profiles

The next owned browser/session parity slice ports an `agent-browser` attachment reliability behavior. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`, where `resolve_cdp_from_active_port` first tries the exact `DevToolsActivePort` WebSocket path and then falls back to HTTP CDP discovery, and `references/repos/agent-browser/cli/src/native/cdp/discovery.rs`, where `/json/version` supplies `webSocketDebuggerUrl`.

`OwnedBrowserAutomationBackend` now keeps direct `DevToolsActivePort` attachment as the primary path for running login/profile browsers. If that WebSocket path cannot be connected, it queries `http://127.0.0.1:<port>/json/version`, rewrites the discovered WebSocket host and port to the local target, and tries that URL before giving up. This improves owned login-finish/profile attachment reliability without adding ambient current-browser access or a new public command.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::`
- `cargo test`

Confidence: Medium-high. The discovery parsing and URL rewrite are deterministic and source-backed. Real Chrome path-staleness behavior still needs ignored/manual Chrome smoke coverage before current-tab attach or broader profile attachment can be considered complete.

### D102: I19e falls back to CDP target-list discovery

The next owned browser/session parity slice ports another `agent-browser` attachment reliability behavior. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/discovery.rs`, where `discover_cdp_url_with_timeout` tries `/json/version`, then `/json/list`, and `fetch_cdp_list` extracts a `webSocketDebuggerUrl` from the browser target or another available target.

`OwnedBrowserAutomationBackend` now keeps direct `DevToolsActivePort` attachment as the primary path, then tries `/json/version`, then tries `http://127.0.0.1:<port>/json/list` before giving up. The owned `/json/list` parser prefers a `type == "browser"` target with a WebSocket URL and otherwise falls back to the first target that has a WebSocket URL, then rewrites the discovered host and port to the local target. This improves existing-profile attach reliability while still avoiding ambient current-tab attach or site-specific browser behavior.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and covered by deterministic local HTTP tests for `/json/version`, `/json/list` browser-target preference, and first-WebSocket target fallback. Real Chrome path-staleness behavior still needs ignored/manual Chrome smoke coverage before current-tab attach or broader profile attachment can be considered complete.

### D103: I19e verifies direct CDP WebSocket discovery

The next owned browser/session parity slice ports the final `agent-browser` CDP discovery fallback. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/discovery.rs`, where `discover_cdp_url_with_timeout` falls back from `/json/version` and `/json/list` to a direct `ws://host:port/devtools/browser` connection and verifies the endpoint with `Browser.getVersion`.

`OwnedBrowserAutomationBackend` now tries direct `ws://127.0.0.1:<port>/devtools/browser` discovery after the direct `DevToolsActivePort` path, `/json/version`, and `/json/list` fail. The direct fallback opens the WebSocket, sends `Browser.getVersion`, and only returns the URL if the endpoint replies through the existing CDP response path. Discovery errors remain classified as backend-unavailable so existing-profile attach can still give up without leaking site-specific advice or ambient browser-control behavior.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and covered by a deterministic local WebSocket test that first returns 404 for both HTTP discovery endpoints, then verifies `Browser.getVersion` over `/devtools/browser`. It still does not prove real Chrome UI-remote-debugging behavior, so manual/ignored Chrome smoke coverage remains a follow-up.

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
