# Knowledge Archive D49-D76: Migration Setup, Parity, And Default Runtime

### D49: OAuth login should prefer real user browsers and verify persisted auth

Manual release testing against Hello Interview showed three distinct auth behaviors:

- OAuth in an automation-controlled `agent-browser` window can be rejected by Google with "This browser or app may not be secure."
- Importing from the user's normal Chrome `Default` profile worked when that profile was already logged in: `aget session import chrome` captured scoped Hello Interview cookies, and the browser fallback extractor fetched the premium article.
- Opening Chrome with a fresh `--user-data-dir` under `AGET_HOME` created a dedicated profile directory and anonymous Hello Interview cookies, but the expected auth cookies (`hi.session-token-2`, `hi.csrf-token`, `hi.callback-url`) did not persist there after the attempted login. Reopening that profile still rendered the logged-out/paywalled page.

Do not treat a custom browser profile as the default login design until it has a proven persistence/import path. The safer product flow is: first detect/import usable existing browser auth, then if auth is missing warn the user that OAuth/user login is needed, suggest importing an existing OAuth session from the user's real browser/profile whenever possible, open the chosen real browser/profile only when user action is required, and verify persisted scoped auth before claiming login success. Dedicated `aget` profiles remain attractive for isolation, but need targeted research around browser choice, OAuth redirects, profile paths, lock handling, and state export before becoming the default.

### D50: API cleanup makes envelope, content format, and inline content explicit

The public CLI/API now separates three concepts that were previously overloaded:

- `--envelope <json|none>` controls the response envelope. `--json` is no longer part of the public API.
- `--content-format <markdown|html|text|json>` controls extracted page content format.
- `--inline-content <auto|always|never>` controls whether the extracted content is embedded in the JSON envelope.

The envelope includes `schema_version: "aget.envelope.v1"`, and `get` data now reports `content_format` instead of `format`. The default `inline-content=auto` omits `data.content` for session-backed/sensitive fetches while still writing local content artifacts, so agents have a safer default for authenticated pages. README now states clearly that `aget` is a proof of concept and that current backend dependencies are part of validating the workflow.

### D51: Current OAuth-safe workflow support is partial, not automatic

Current `aget` supports the building blocks for the desired workflow:

- Fetch first with an empty session and structured envelope output.
- Import a scoped Chrome profile session with explicit `--allow-domain` values.
- Return `requires_user_action` when Chrome/profile import cannot proceed cleanly, including locked-profile and no-auth-state cases.
- Fetch again with an explicit named session.
- Omit `data.content` by default for session-backed/sensitive JSON envelopes while keeping local content artifacts.
- Guide agents, through the project skill and OpenCode tool descriptions, to prefer real-browser session import for OAuth-backed sites.

It does **not** yet support the full workflow as a first-class product/API:

- There is no single `aget` command or tool that runs the complete decision tree: fetch unauthenticated, import a real browser session, verify scoped auth, open the user's chosen browser only if auth is missing, re-import, and verify again.
- Chrome import is the only implemented real-browser import surface. Browser-choice terminology is not designed beyond `--chrome-profile`, and there is no first-class Arc/Brave/Firefox/Safari/default-browser flow.
- Verification is agent-driven rather than `aget`-driven: the agent runs a follow-up fetch and interprets content. That respects the generic-fetcher boundary, but the product still needs a generic verification command or recipe that records whether a session was usable for a target URL without encoding site-specific login/paywall rules in the binary.
- `session login start` is still an automation-owned fallback, not the OAuth-safe default, and it should remain clearly secondary for OAuth-backed sites.
- Deterministic tests cover the pieces (`session import chrome`, profile-lock/no-state `requires_user_action`, session-backed fetch, sensitive inline-content defaults, and mocked-site protected fetch), but not the whole OAuth-safe decision tree as one agent workflow.

Target flow for agents:

1. Fetch the URL without a session and save content to an artifact.
2. If the result appears gated and the user authorizes access, ask which real browser/profile already has access.
3. Import only explicit allowed domains from that user-approved browser/profile into a named local session.
4. Verify by fetching the same target URL with that session, preferably writing content to an artifact and using `--inline-content auto`.
5. If import returns `requires_user_action` or verification still appears gated, ask the user to sign in through their real browser, then re-import and re-verify.
6. Use `session login start` only as an explicit fallback for non-OAuth or controlled flows where automation-owned login is acceptable.

Future public API direction:

- Keep current `aget session import chrome --chrome-profile <profile> --allow-domain <domain>` as the proven PoC path.
- Add an OAuth-safe orchestration layer, likely `aget session authorize` or `aget auth prepare`, that models the decision tree without classifying site-specific content itself.
- Add browser-choice terminology for user-facing flows: `--browser <default|chrome|brave|arc|edge|firefox|safari>`, `--browser-profile <name>`, and `--profile-path <path>`. Only expose import implementations that are actually supported; opening a real browser for user login can support more browsers earlier than state import.
- Add generic session verification, either as `aget session verify <name> --url <url>` or as a documented fetch recipe with caller-provided checks such as selectors or must-contain/must-not-contain text. Verification must remain generic and must not hardcode paywall/login rules.

Confidence: Medium-high. The assessment is grounded in current code, tests, README, and agent skill behavior. The remaining uncertainty is product design: how much of the decision tree belongs in `aget` versus host-agent guidance, and how to verify auth usability generically without introducing site-specific heuristics.

### D52: Dependency migration should proceed through parity-first backend replacement

The validated PoC can now be migrated away from required Crawl4AI and `agent-browser` runtime dependencies, but the migration should preserve the current public CLI/API and backend capability boundaries first. Task I19 is now an umbrella split into source staging, abstraction audit, parity tests, Crawl4AI feature porting, `agent-browser` feature porting, default-backend switch, PoC surface cleanup, and final review.

Source snapshots are local and gitignored:

- Crawl4AI: `references/repos/crawl4ai`, remote `https://github.com/unclecode/crawl4ai.git`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`, Apache-2.0.
- `agent-browser`: `references/repos/agent-browser`, remote `https://github.com/vercel-labs/agent-browser.git`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, Apache-2.0.

Both upstream projects are permissively licensed, so adapting small tests or code is possible if attribution/license obligations are handled. The safer default remains behavior-driven porting: inspect upstream source for implementation strategy, generate or adapt focused parity tests for the `aget` feature subset, then implement owned Rust backends behind the existing interfaces.

Current Crawl4AI-dependent `aget` feature inventory:

- `scripts/crawl4ai_extract.py` is the current Python bridge. It creates `BrowserConfig` with headless Chromium, storage-state input, viewport defaults, optional channel, and a small allowlist of namespaced `crawl4ai.*` options. It creates `CrawlerRunConfig` with cache bypass, CSS wait, overlay removal, optional selector/exclusion, writes private output/metadata files, and selects markdown/html/text/json content from Crawl4AI result fields.
- `src/extraction.rs` owns `ExtractorBackend`, `CommandExtractorBackend`, session-state composition, primary extraction, fallback trigger, final success/error metadata, timeout/error mapping, artifact writing, truncation metadata, redaction, and the constant extractor name `crawl4ai`.
- `src/cli.rs`, README, `.opencode/tools/aget.ts`, and `.cursor/skills/aget/SKILL.md` expose the current Crawl4AI-shaped option surface through `--content-format`, selectors, CSS-only wait, and namespaced backend options.
- Tests that define the parity baseline include `tests/get_cli.rs`, `tests/mock_site_cli.rs`, `tests/aget_api.rs`, `tests/cli.rs`, `tests/support/bin/aget_mock_backend.rs`, and the ignored real Crawl4AI replay test.

Current `agent-browser`-dependent `aget` feature inventory:

- `src/session/agent_browser.rs` owns command execution through `AGET_AGENT_BROWSER_COMMAND`, backend-unavailable/timeout handling, profile-lock/login-needed classification, raw Playwright-style state parsing, cookie/storage allowlist filtering, duplicate conflict detection, domain normalization, private temp files, and raw-state cleanup helpers.
- `src/session/chrome.rs` uses `agent-browser` to open a Chrome profile, save raw state, filter it into a scoped local session, detect no-auth state, close the temp session, and remove raw state.
- `src/session/login.rs` uses `agent-browser` to start visible login sessions, finish by saving/filtering state, close sessions, remove tool-owned login profiles, and cancel pending login state.
- `src/extraction.rs` uses `agent-browser` as the session-backed fallback when Crawl4AI cannot consume composed state, loading state into a temp browser profile, opening the URL, extracting body HTML/text, closing the session, and cleaning fallback profiles.
- `src/aget.rs` already hides these operations behind `BrowserAutomationBackend` and `BrowserFallbackBackend`; I19b should verify those traits are sufficient before new backend work starts.
- Tests that define the parity baseline include `tests/session_cli.rs`, `tests/get_cli.rs`, `tests/mock_site_cli.rs`, `tests/aget_api.rs`, `tests/support/bin/aget_mock_agent_browser.rs`, and the ignored real/manual agent-browser checks.

Upstream implementation paths to inspect before porting:

- Crawl4AI extraction: `crawl4ai/async_webcrawler.py`, `crawl4ai/async_configs.py`, `crawl4ai/browser_manager.py`, `crawl4ai/async_crawler_strategy.py`, `crawl4ai/markdown_generation_strategy.py`, `crawl4ai/content_scraping_strategy.py`, `crawl4ai/html2text/`, `tests/async/`, `tests/browser/`, and `tests/cli/`.
- `agent-browser` browser/session: `cli/src/commands.rs`, `cli/src/connection.rs`, `cli/src/native/browser.rs`, `cli/src/native/state.rs`, `cli/src/native/cookies.rs`, `cli/src/native/storage.rs`, `cli/src/native/cdp/`, `cli/src/native/e2e_tests.rs`, and `cli/tests/doctor_cli.rs`.

Risk notes for I19:

- Replacement must avoid importing Crawl4AI anti-bot/stealth/proxy escalation patterns that conflict with `aget`'s authorization-only boundary.
- Browser/session replacement is security-sensitive because cookies, localStorage, raw browser state, screenshots, stdout/stderr, and authenticated content are credential-equivalent or private artifacts.
- The standard suite should eventually pass with Crawl4AI and `agent-browser` absent from PATH, but command adapters should remain until homegrown backends pass equivalent parity coverage.
- Do not port an entrypoint from memory. For each behavior, inspect the upstream implementation snapshot first and record the source path used for inspiration.

Confidence: Medium. I19a now has local source snapshots and a concrete migration inventory, but no parity tests or implementation have started.

### D53: I19b keeps backend swaps static and adds structured state to extraction requests

I19b audited the current `Aget` backend boundaries against the migration inventory in D52. The existing static-dispatch design is still the right shape for the migration:

- `ExtractorBackend` owns the "URL plus composed session state to extracted content" capability.
- `BrowserAutomationBackend` owns login, Chrome/profile import, and login cancellation/finish flows.
- `BrowserFallbackBackend` owns session-backed browser extraction when the primary extractor fails.
- `SessionStoreBackend` owns local session persistence and lets API-style tests use a non-filesystem store.

The main abstraction leak was that `ExtractorRequest` and `BrowserFallbackRequest` only exposed `state_path`, forcing future in-process backends to parse the temporary Playwright storage-state file that exists for command adapters. The request structs now also carry a borrowed structured `PlaywrightState`, while keeping `state_path` for the command-backed Crawl4AI and `agent-browser` adapters. This lets the homegrown backends start from structured cookies/storage without changing the public CLI/API or removing command compatibility.

API-level swappability coverage now verifies:

- A custom non-command extractor can fetch through `Aget` using a custom in-memory session store.
- A custom non-command browser fallback can handle session-backed fallback extraction.
- A custom non-command browser automation backend can finish login, import a Chrome/profile session, and start/cancel login through `Aget` without shelling out.
- The custom extractor and fallback see both the compatibility state file and the structured composed state.

Deferred abstraction notes:

- `PlaywrightState` remains the internal session-state interchange format for now because current adapters and tests already use that shape. I19d/I19e may rename or wrap it if homegrown backends need a browser-neutral state model.
- `AgetWith` currently uses one browser backend value for both `BrowserAutomationBackend` and `BrowserFallbackBackend`. That matches the current `agent-browser` replacement scope; split values can be introduced later only if extraction fallback and login/import need different implementations.
- `get_url_with_backend` and `get_url_with_backends` still construct a filesystem `SessionStore` from `GetOptions.home`. API callers that need custom stores should use `AgetWith` or `get_url_with_session_store`; keeping the compatibility functions avoids widening this migration pass.

Validation:

- `cargo test --test aget_api`
- `cargo test`

Confidence: High for I19b. The change is narrow, tested, and keeps command-backed adapters available while removing the concrete file-only state dependency for future homegrown backends.

### D54: I19c parity matrix is behavior-driven, not upstream-test copying

I19c defines the dependency parity target for the features `aget` actually uses today. No upstream test code has been copied. Crawl4AI and `agent-browser` are both Apache-2.0 in the local snapshots, but the safer first pass is still behavior-driven parity using `MockSite`, checked-in mock tools, current command-adapter behavior, and existing ignored real-backend smoke tests.

Extractor parity matrix:

| Feature | Required parity | Deterministic coverage |
| --- | --- | --- |
| Public fetch | Fetch a URL without session state and write private content/metadata artifacts. | `tests/mock_site_cli.rs::documents_public_get_json_contract_for_agents`, `tests/get_cli.rs::get_json_success_writes_run_artifacts_with_empty_state` |
| Authenticated replay | Compose selected sessions into request state and replay cookies/localStorage only to matching scopes. | `tests/mock_site_cli.rs::mock_site_replays_cookie_and_storage_sessions`, `tests/mock_site_cli.rs::documents_session_compose_replay_and_scope_rejection_contract`, `tests/get_cli.rs::get_session_uses_named_session_state_and_marks_sensitive` |
| Content formats | Preserve markdown default plus text/html/json output contracts. | `tests/mock_site_cli.rs::backend_parity_covers_extractor_content_formats` |
| Selectors/exclusions/waits | Apply CSS selector, exclusion, and CSS-only wait behavior; reject JavaScript waits. | `tests/mock_site_cli.rs::mock_site_fetch_handles_redirect_output_shaping_and_waits`, `tests/get_cli.rs::get_real_helper_rejects_javascript_wait_before_crawl4ai_import` |
| Backend options | Preserve namespaced `crawl4ai.*` option validation until a replacement namespace is designed. | `tests/get_cli.rs::get_forwards_supported_output_options_and_records_limits`, `tests/get_cli.rs::get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import` |
| Final URL and warnings | Surface backend final URL and warnings in the stable envelope and metadata. | `tests/mock_site_cli.rs::documents_output_limits_out_file_and_warning_contract`, `tests/mock_site_cli.rs::documents_custom_site_routes_for_extraction_features` |
| Limits and artifacts | Truncate only final content, record limit metadata, write content and metadata artifacts. | `tests/mock_site_cli.rs::documents_output_limits_out_file_and_warning_contract`, `tests/get_cli.rs::get_json_format_truncates_only_content_not_response_envelope` |
| Failure mapping | Preserve malformed output, structured failure, missing backend, timeout, and descendant termination behavior. | `tests/get_cli.rs::get_nonzero_and_malformed_backend_results_are_extraction_failed`, `tests/get_cli.rs::missing_backend_returns_backend_unavailable`, `tests/get_cli.rs::get_timeout_returns_stable_error_and_error_metadata`, `tests/get_cli.rs::get_timeout_terminates_backend_descendants` |

Browser/session parity matrix:

| Feature | Required parity | Deterministic coverage |
| --- | --- | --- |
| Chrome/profile import | Import scoped browser state, filter by explicit allowed domains, save session, clean raw state. | `tests/mock_site_cli.rs::mock_site_imported_chrome_session_can_fetch_protected_page`, `tests/session_cli.rs::session_import_chrome_saves_filtered_state_and_cleans_raw_file` |
| Login start/finish/cancel | Open a user-visible login flow, finish by filtering state, cancel/cleanup pending flows. | `tests/mock_site_cli.rs::documents_session_lifecycle_contract_for_agents`, `tests/mock_site_cli.rs::mock_site_login_bootstrap_can_fetch_protected_page_without_manual_action`, `tests/session_cli.rs` login tests |
| State parsing/filtering | Parse Playwright-style cookies/localStorage, accept float cookie expiries, reject conflicts, preserve provenance. | `src/session/agent_browser.rs` unit tests, `tests/session_cli.rs::session_login_finish_saves_only_url_scoped_state_and_cleans_temp_files` |
| Requires-user-action classification | Preserve profile-lock, no-auth-state, and login-needed classification. | `tests/session_cli.rs::session_import_chrome_requires_user_action_for_profile_lock`, `tests/session_cli.rs::session_login_finish_rejects_provider_only_state_without_saving_session` |
| Fallback extraction | On session-backed primary extraction failure, load composed state into browser fallback, extract body content, close/cleanup. | `tests/get_cli.rs::get_session_backend_failure_uses_agent_browser_fallback_with_composed_state`, `tests/get_cli.rs::get_session_fallback_close_failure_preserves_original_sanitized_crawl4ai_error` |
| Redaction and temp safety | Redact state secrets from errors/artifacts and remove raw state/temp profiles. | `tests/get_cli.rs::get_session_backend_failure_redacts_state_secrets_from_errors_metadata_and_artifacts`, `tests/session_cli.rs` raw-state cleanup tests |

Optional real-backend comparison commands:

```bash
cargo test --test get_cli real_crawl4ai_replays_named_session_cookie -- --ignored
cargo test --test session_cli real_hellointerview_login_flow_fetches_paywalled_markdown -- --ignored
cargo test --test session_cli real_cmux_import_replays_loopback_cookie_through_crawl4ai -- --ignored
```

The first command is the direct Crawl4AI replay check. The second exercises the real `agent-browser` plus Crawl4AI authenticated flow and requires manual authorized login. The third remains useful for optional cmux-to-Crawl4AI replay comparison, but cmux is not one of the two replacement targets for I19.

Validation:

- `cargo test --test mock_site_cli backend_parity_covers_extractor_content_formats`

Confidence: Medium-high. The parity target is explicit and mostly backed by existing deterministic tests plus one new content-format parity test. Remaining risk is that I19d/I19e may reveal additional edge cases from upstream implementation inspection; those should extend this matrix before porting each specific behavior.

