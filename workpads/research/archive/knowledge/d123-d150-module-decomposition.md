# Knowledge Archive D123-D150: Module And Test Decomposition

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

### D133: I19i splits extraction artifacts and redaction helpers

The next extraction decomposition slice moved run artifact mechanics from `src/extraction/mod.rs` to `src/extraction/artifacts.rs`. The new module owns run ID generation, private run directories and files, success/error metadata JSON writing, backend stdout/stderr reading, sensitive-value collection, backend error sanitization, backend artifact redaction, and redaction pattern generation for literal, percent-encoded, form-encoded, and JSON-escaped secret values. The main extraction module still owns request orchestration, session loading/scope enforcement, primary/fallback selection, output limits, and owned extraction behavior.

Validation:

- `cargo check`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test extraction::tests::redacts`
- `cargo test --test get_cli get_json_success_writes_run_artifacts_with_empty_state`
- `cargo test --test get_cli get_session_backend_failure_redacts_state_secrets_from_errors_metadata_and_artifacts`
- `cargo test`

Confidence: High for the extraction artifacts split. The split is mechanical, focused artifact/redaction coverage passed, and the full standard suite is green.

### D134: I19i splits CDP rendered-page orchestration

The next `browser_cdp` decomposition slice moved rendered-page orchestration from `src/browser_cdp.rs` to `src/browser_cdp/render.rs`. The new module owns `BrowserRenderRequest`, `RenderedPage`, `PageWaitUntil`, temporary Chrome launch for page rendering, page-domain setup, optional shadow-root opening and flattening, session-state loading, navigation/wait orchestration, image waits, settle delay, rendered overlay cleanup dispatch, final URL evaluation, HTML capture fallback, target close, and browser shutdown. The parent `browser_cdp` module keeps state export and login lifecycle orchestration for separate state/login splits.

Validation:

- `cargo check`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test browser_cdp::tests::`
- `cargo test`

Confidence: High for the CDP render split. The split is mechanical, the CDP-focused unit slice passed, the owned-extractor caller path still passes, and the full standard suite is green.

### D135: I19i splits CDP Chrome profile state export

The next `browser_cdp` decomposition slice moved Chrome profile state export orchestration from `src/browser_cdp.rs` to `src/browser_cdp/state.rs`. The new module owns `BrowserStateExportRequest`, Chrome profile launch for import, page creation/domain enablement, CDP state export dispatch, target close, browser close, and shutdown waiting. The parent `browser_cdp` module now keeps login lifecycle orchestration and delegates profile import state export to the state module.

Validation:

- `cargo check`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::`
- `cargo test --test session_cli owned_session_import_chrome_classifies_profile_in_use`
- `cargo test --test session_cli owned_session_import_chrome_reports_sandbox_startup_hint`
- `cargo test`

Confidence: High for the CDP state split. The split is mechanical, the CDP-focused unit slice passed, deterministic owned Chrome import classification paths still pass, and the full standard suite is green.

### D136: I19i splits CDP login lifecycle orchestration

The next `browser_cdp` decomposition slice moved owned login lifecycle orchestration from `src/browser_cdp.rs` to `src/browser_cdp/login.rs`. The new module owns `BrowserLoginStartRequest`, `StartedLoginBrowser`, `BrowserLoginStateExportRequest`, `BrowserLoginCloseRequest`, login profile directory setup, headed login Chrome launch/detach, existing-profile CDP attach for login finish/cancel, existing-page reuse, created-page cleanup, login state export, Browser.close shutdown polling, PID-backed process cleanup, and fallback export after a closed login browser. The parent `browser_cdp` module now mostly re-exports public capability entrypoints while lower-level CDP discovery depends directly on `client::CdpClient` instead of a parent import.

Validation:

- `cargo check`
- `cargo test --test session_cli session_login_start_opens_aget_owned_browser_and_records_pending_flow`
- `cargo test --test session_cli session_login_finish_saves_only_url_scoped_state_and_cleans_temp_files`
- `cargo test --test session_cli session_login_cancel_cleans_profile_and_pending_when_close_fails`
- `cargo test browser_cdp::tests::`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the CDP login split. The split is mechanical, focused login lifecycle paths passed, the CDP unit slice passed, and the full standard suite is green.

### D137: I19i splits owned extraction orchestration

The next extraction decomposition slice moved the owned Rust extraction and owned browser fallback execution path from `src/extraction/mod.rs` to `src/extraction/owned.rs`. The new module owns `run_owned_extractor_backend`, `run_owned_browser_fallback`, static-versus-rendered routing, rendered-page handoff, owned backend option parsing, wait-selector retry behavior, output-format selection, default main-content selection, target-element extraction, cleaned HTML serialization, markdown/text extraction glue, and owned fallback warning/extractor labels. The parent extraction module now keeps the public API, session loading/composition, replay-scope checks, fallback decision, output envelope, artifact finalization, and shared error helpers.

Validation:

- `cargo check`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli backend_parity_covers_extractor_content_formats`
- `cargo test`

Confidence: High for the owned extraction split. The move is mechanical, focused owned-extraction integration tests passed, and the full standard suite is green.

### D138: I19i splits CDP unit tests from the public module

The next CDP decomposition slice moved the `browser_cdp` unit test module from `src/browser_cdp.rs` to `src/browser_cdp/tests.rs`. The parent CDP module now contains only submodule declarations, public capability re-exports, shared Chrome startup constants, private directory helpers, and shared I/O error conversion. Test-only imports moved with the tests, so the production module is no longer visually dominated by local Chrome, CDP discovery, page script, storage, and login smoke tests.

Validation:

- `cargo check`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::`
- `cargo test`

Confidence: High for the CDP test-module split. The move is mechanical, the CDP unit slice passed, and the full standard suite is green.

### D139: I19i splits CDP session data conversion helpers

The next CDP client decomposition slice moved Playwright/CDP session data conversion helpers from `src/browser_cdp/client.rs` to `src/browser_cdp/session_data.rs`. The new module owns CDP cookie payload construction, CDP cookie parsing and deduplication, allowed-domain storage-origin expansion, Runtime storage result parsing into `PlaywrightOrigin`, and target selection filtering for existing page attachment. `client.rs` now keeps socket/session command handling, navigation/wait logic, state load/export orchestration, and CDP request/response plumbing while re-exporting the moved helpers for existing unit tests.

Validation:

- `cargo check`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test browser_cdp::tests::`
- `cargo test`

Confidence: High for the CDP session data split. The move is mechanical, the CDP unit slice passed, and the full standard suite is green.

### D140: I19i splits mock-site CLI integration helpers

The next integration-test decomposition slice moved shared helper setup from `tests/mock_site_cli.rs` to `tests/support/mock_site_cli.rs`. The support module now owns mock backend and agent-browser command lookup/building, the `Aget` command-backend helper, JSON envelope extraction, reusable storage-rendered mock site setup, cookie/storage/mixed-scope session fixtures, and the failing extractor used by fallback tests. `tests/mock_site_cli.rs` now imports those helpers through `support::mock_site_cli`, which prepares the file for behavior-focused splits without duplicating helper code.

Validation:

- `cargo check`
- `cargo test --test mock_site_cli`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the mock-site helper split. The helper move is mechanical, the affected integration target passed, and the full standard suite is green.

### D141: I19i splits mock-site documentation contract tests

The next integration-test decomposition slice moved documentation/agent-contract mock-site tests from `tests/mock_site_cli.rs` to `tests/mock_site_docs_contract.rs`. The new test target owns public JSON contract coverage, output/artifact contract coverage, custom route extraction examples, session compose/scope rejection contract coverage, and the session login/list/inspect/delete lifecycle contract. Shared helper setup comes from `tests/support/mock_site_cli.rs`, and `tests/support/mod.rs` now allows dead-code because each integration target uses a different subset of shared helpers and mock-site methods.

Validation:

- `cargo test --test mock_site_cli`
- `cargo test --test mock_site_docs_contract`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the mock-site documentation contract split. The split is mechanical, both affected integration targets passed, and the full standard suite is green.

### D142: I19i splits mock-site browser/rendering tests

The next integration-test decomposition slice moved browser fallback and Chrome-rendered extraction coverage from `tests/mock_site_cli.rs` to `tests/mock_site_browser.rs`. The new test target owns the owned browser fallback replay test, ignored local-Chrome browser fallback smokes, localStorage-backed rendered extraction, waited JavaScript rendering, script auto-rendering, shadow DOM flattening, render delay, rendered overlay cleanup, image waiting, and network-idle smokes. `tests/mock_site_cli.rs` now keeps the command/default/static extraction and session-oriented mock-site coverage.

Validation:

- `cargo test --test mock_site_cli`
- `cargo test --test mock_site_browser`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the mock-site browser/rendering split. The split is mechanical, both affected integration targets passed, and the full standard suite is green.

### D143: I19i splits mock-site session/auth tests

The next integration-test decomposition slice moved mock-site session and auth scenario coverage from `tests/mock_site_cli.rs` to `tests/mock_site_sessions.rs`. The new test target owns cookie and storage replay, unauthenticated/expired/logout states, imported Chrome session replay through the compatibility mock, and login bootstrap replay. `tests/mock_site_cli.rs` is now focused on command/default/static extraction parity and is below 900 lines.

Validation:

- `cargo test --test mock_site_cli`
- `cargo test --test mock_site_sessions`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the mock-site session/auth split. The split is mechanical, both affected integration targets passed, and the full standard suite is green.

### D144: I19i splits shared session CLI helpers

The next integration-test decomposition slice moved shared `tests/session_cli.rs` fixture and fake-tool helpers to `tests/support/session_cli.rs`. The support module now owns agent-browser mock command wrapping, Chrome import state fixtures, NYTimes/provider-only state fixtures, JSON envelope helpers, reusable session model fixtures, local cmux loopback HTTP fixtures, and small wrappers for the shared mock backend/cmux commands. `tests/session_cli.rs` now imports these helpers through the support module, which prepares the file for behavior-focused splits without duplicating command or session fixture setup.

Validation:

- `cargo test --test session_cli`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the session CLI helper split. The helper move is mechanical, the affected integration target passed, and the full standard suite is green.

### D145: I19i splits session import CLI tests

The next session CLI decomposition slice moved the cmux import, Chrome import, owned Chrome import error-classification, and ignored real-cmux replay tests from `tests/session_cli.rs` into `tests/session_cli/imports.rs`. The root `tests/session_cli.rs` now declares that module with an explicit path, keeping the behavior-focused import tests out of the root file without making the submodule an independent Cargo integration target.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --test session_cli`
- `git diff --check`
- `cargo test`

Confidence: High for the import-test split. The move is mechanical, the affected integration target passed, and the full standard suite is green.

### D146: I19i splits session login CLI tests

The next session CLI decomposition slice moved login start, finish, cancel, helper API, and ignored real-login smoke coverage from `tests/session_cli.rs` into `tests/session_cli/login.rs`. The root `tests/session_cli.rs` now declares separate import and login behavior modules and is reduced to list, compose, and inspect coverage.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --test session_cli`
- `git diff --check`
- `cargo test`

Confidence: High for the login-test split. The move is mechanical, the affected integration target passed, and the full standard suite is green.

### D147: I19i splits shared get CLI helpers

The next get CLI decomposition slice moved shared fake backend, fake agent-browser, success-envelope, metadata discovery, loopback cookie echo server, and saved-session fixture helpers from `tests/get_cli.rs` into `tests/support/get_cli.rs`. `tests/get_cli.rs` now imports those helpers through the shared integration-test support module, preparing later behavior-focused get-test splits without duplicating backend command or session fixture setup.

Validation:

- `cargo fmt`
- `cargo test --test get_cli`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the get CLI helper split. The helper move is mechanical, the affected integration target passed, and the full standard suite is green.

### D148: I19i splits session-backed get CLI tests

The next get CLI decomposition slice moved session-backed replay, replay-scope rejection, repeated-session composition, provider/app cookie flow, session sensitivity, primary-backend failure redaction, agent-browser fallback, unauthenticated fallback suppression, fallback close-failure preservation, and ignored real Crawl4AI session replay coverage from `tests/get_cli.rs` into `tests/get_cli/session.rs`. The root `tests/get_cli.rs` now declares that behavior module with an explicit path and keeps non-session output, option, timeout, and backend-failure tests in the root file.

Validation:

- `cargo fmt`
- `cargo test --test get_cli`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the session-backed get-test split. The move is mechanical, the affected integration target passed, and the full standard suite is green.

### D149: I19i splits owned mock-site extractor parity tests

The next mock-site CLI decomposition slice moved the long owned static extractor parity test from `tests/mock_site_cli.rs` into `tests/mock_site_cli/owned.rs`. The root mock-site CLI test target now declares the owned behavior module with an explicit path and keeps command-backed redirect/output-shaping/default-backend/content-format parity smokes in the root file.

Validation:

- `cargo fmt`
- `cargo test --test mock_site_cli`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the owned mock-site split. The move is mechanical, the affected integration target passed, and the full standard suite is green.

### D150: I19i completed oversized module split

I19i is complete. The original production bottlenecks are now decomposed into behavior-owned extraction and CDP modules, and the largest integration-test bottlenecks are split into shared support helpers plus behavior modules. The current largest files in the touched surface are around 600-750 lines, instead of the original 2k-3k line extraction/CDP/test files. The remaining medium-sized files are coherent enough for follow-up feature work and do not need more physical splitting before moving to the next workpad task.

Final size evidence:

- `src/browser_cdp/client.rs`: 753 lines
- `src/extraction/mod.rs`: 733 lines
- `src/extraction/owned.rs`: 669 lines
- `tests/mock_site_cli/owned.rs`: 725 lines
- `tests/get_cli.rs`: 709 lines
- `tests/get_cli/session.rs`: 599 lines
- `tests/session_cli/login.rs`: 734 lines
- `tests/session_cli/imports.rs`: 652 lines
- `tests/session_cli.rs`: 417 lines

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High. Each split was committed separately after focused validation plus the full standard gate, and no intentional behavior changes were introduced.
