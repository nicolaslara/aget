# D92-D97: Markdown Lists, Links, Tables, And Chrome Startup

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
