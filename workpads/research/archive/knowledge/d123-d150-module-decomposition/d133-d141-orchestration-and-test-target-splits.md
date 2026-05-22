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

