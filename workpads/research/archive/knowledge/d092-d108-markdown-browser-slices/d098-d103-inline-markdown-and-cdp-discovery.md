# D98-D103: Inline Markdown And CDP Discovery

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
