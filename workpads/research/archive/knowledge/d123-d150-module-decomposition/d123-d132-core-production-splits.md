### D123: I19i starts the extraction module split with markdown rendering

The first decomposition slice is behavior-preserving. `src/extraction.rs` became `src/extraction/mod.rs`, keeping the public `aget::extraction` module path stable, and the owned HTML-to-markdown renderer moved into `src/extraction/markdown.rs`. The parent extraction module still owns orchestration, sessions, options, HTTP/static extraction, cleanup, adapters, and artifacts for now; the new markdown module owns MarkdownWriter, DOM-to-markdown traversal, inline escaping, table/list/link/image rendering, abbreviation definitions, and markdown URL resolution.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for the first physical split. The change is mechanical, preserves the public `aget::extraction` module path, and focused plus full-suite validation passed. The remaining I19i work is to keep carving `src/extraction/mod.rs` and then split `src/browser_cdp.rs`.

### D124: I19i splits the Crawl4AI command adapter

The second decomposition slice moved the Crawl4AI compatibility process adapter from `src/extraction/mod.rs` to `src/extraction/command.rs`. The new module owns `AGET_CRAWL4AI_COMMAND` lookup, helper argument construction, backend stdout/stderr artifact files, command spawning, timeout handling, exit-status mapping, and backend JSON parsing. The parent extraction module keeps the public `CommandExtractorBackend` type and delegates to the module, so public API behavior is unchanged.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test get_cli get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import`
- `cargo test`

Confidence: High for the command-adapter split. This is another mechanical split, the focused compatibility-adapter test passed, and the full standard suite is green.

### D125: I19i splits the agent-browser fallback adapter

The third decomposition slice moved the legacy `agent-browser` fallback adapter from `src/extraction/mod.rs` to `src/extraction/fallback_command.rs`. The new module owns fallback profile/session naming, state load/open/get/close command orchestration, temporary fallback profile cleanup, temporary stdout/stderr files, fallback HTML-to-text conversion, and agent-browser exit classification. The parent extraction module still owns the public `CommandBrowserFallbackBackend` type and delegates to this compatibility module.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test get_cli get_session_backend_failure_uses_agent_browser_fallback_with_composed_state`
- `cargo test`

Confidence: High for the fallback-adapter split. The focused fallback test passed after the mechanical split, and the full standard suite is green.

### D126: I19i splits CDP page scripts

The first `browser_cdp` decomposition slice moved embedded page JavaScript helpers from `src/browser_cdp.rs` to `src/browser_cdp/page_scripts.rs`. The new module owns runtime expressions for localStorage/sessionStorage loading, selector existence waits, rendered overlay cleanup, shadow-root attach override, and shadow DOM flattening. The CDP client/rendering code still calls the same helper names through module imports, so behavior is unchanged.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::shadow_dom_flatten_expression_resolves_slots_and_skips_styles`
- `cargo test`

Confidence: High for the first CDP split. The split is mechanical, the script-contract test passed, and the full standard suite is green.

### D127: I19i splits owned HTML cleanup

The next extraction decomposition slice moved owned HTML cleanup helpers from `src/extraction/mod.rs` to `src/extraction/html_clean.rs`. The new module owns CSS selector parsing, removal of excluded tags and selected nodes, generic overlay selector cleanup, base64 image source blanking, empty-leaf pruning, code-block bypass handling, and Crawl4AI important-attribute pruning. The parent extraction module still owns orchestration and content selection, and calls the same helper names through imports.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli documents_custom_site_routes_for_extraction_features`
- `cargo test`

Confidence: High for the owned HTML cleanup split. The split is mechanical, focused owned-extractor coverage passed, and the full standard suite is green.

### D128: I19i splits owned HTTP fetch

The next extraction decomposition slice moved owned direct HTTP fetch and cookie replay helpers from `src/extraction/mod.rs` to `src/extraction/http.rs`. The new module owns URL parsing and http/https validation, `ureq` agent setup, redirect-aware final URL capture, User-Agent setup, Playwright-state cookie header construction, cookie domain/path/secure matching for requests, and `ureq` timeout/error mapping. The parent extraction module still owns the higher-level static-or-rendered decision and passes the fetched response into existing HTML extraction.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli default_cli_fetch_uses_owned_backend_without_command_dependencies`
- `cargo test --test mock_site_cli mock_site_replays_cookie_and_storage_sessions`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for the owned HTTP fetch split. The split is mechanical, focused public fetch plus cookie replay coverage passed, and the full standard suite is green.

### D129: I19i splits CDP process helpers

The next `browser_cdp` decomposition slice moved process-control helpers from `src/browser_cdp.rs` to `src/browser_cdp/process.rs`. The new module owns owned-login browser exit enforcement, process-exit polling, process-command matching, process-group or PID termination, Chrome process-group configuration, and child-process termination. CDP rendering, discovery, state export, and Chrome launch command construction still live in `src/browser_cdp.rs`.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::chrome_launch_retries_after_early_startup_exit`
- `cargo test browser_cdp::tests::wait_for_devtools_active_port_uses_stderr_fallback`
- `cargo test browser_cdp::tests::existing_profile_attach_removes_stale_devtools_active_port`
- `cargo test`

Confidence: High for the CDP process split. The split is mechanical, focused process/lifecycle coverage passed, and the full standard suite is green.

### D130: I19i splits CDP discovery and startup classification

The next `browser_cdp` decomposition slice moved Chrome DevTools endpoint discovery and startup classification from `src/browser_cdp.rs` to `src/browser_cdp/discovery.rs`. The new module owns `DevToolsActivePort` parsing, stderr `DevTools listening on ...` URL fallback, Chrome startup stderr classification and hints, existing-profile CDP attach discovery, stale `DevToolsActivePort` cleanup, `/json/version` and `/json/list` discovery, direct `/devtools/browser` WebSocket verification, WebSocket host rewriting, and profile-browser shutdown polling. The CDP protocol client and target/session helpers remain in `src/browser_cdp.rs`.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::discovers_cdp_websocket_url_from_json_version`
- `cargo test browser_cdp::tests::parses_devtools_ws_url_from_chrome_stderr`
- `cargo test`

Confidence: High for the CDP discovery split. The split is mechanical, focused discovery/startup coverage passed, and the full standard suite is green.

### D131: I19i splits Chrome process launch ownership

The next `browser_cdp` decomposition slice moved Chrome launch/profile process ownership from `src/browser_cdp.rs` to `src/browser_cdp/chrome_process.rs`. The new module owns temporary-profile directory allocation, Chrome binary discovery, launch command construction, launch retry behavior, `DevToolsActivePort` startup waiting through the discovery module, process detach, shutdown waiting, and temporary user-data-dir cleanup. The main `browser_cdp` module still owns public request/result types and high-level orchestration; CDP protocol client/session helpers remain a larger follow-up boundary.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::chrome_launch_retries_after_early_startup_exit`
- `cargo test browser_cdp::tests::wait_for_devtools_active_port_uses_stderr_fallback`
- `cargo test`

Confidence: High for the Chrome process split. The split is mechanical, focused Chrome startup coverage passed, and the full standard suite is green.

### D132: I19i splits CDP client and session plumbing

The next `browser_cdp` decomposition slice moved CDP protocol client/session plumbing from `src/browser_cdp.rs` to `src/browser_cdp/client.rs`. The new module owns `CdpClient`, `PageSession`, WebSocket connect/send/read handling, target creation/attachment, domain enablement, navigation waits including network-idle tracking, selector/image waits, runtime string evaluation, rendered overlay cleanup command dispatch, Playwright-state cookie/storage load and export, CDP cookie conversion, storage-origin probing, and page-target selection. The main `browser_cdp` module now keeps the public request/result API and high-level orchestration while delegating protocol details to the client module.

Validation:

- `cargo check`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::`
- `cargo test`

Confidence: High for the CDP client split. The split is mechanical, the CDP-focused unit slice passed, and the full standard suite is green.

