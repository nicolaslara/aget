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

