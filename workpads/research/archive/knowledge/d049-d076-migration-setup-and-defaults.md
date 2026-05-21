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

### D55: I19d starts with an owned static extractor after Crawl4AI source inspection

I19d inspected the local Crawl4AI snapshot before porting extraction behavior. Relevant source paths:

- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py`: `AsyncWebCrawler.arun` composes the high-level fetch pipeline: cache/robots checks, crawler strategy navigation, response handling, HTML processing, markdown/content selection, and result metadata.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`: the Playwright strategy owns browser/page/context acquisition, navigation, and post-load DOM content retrieval.
- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `BrowserConfig` and run configuration show that `aget`'s current adapter uses only a narrow subset: headless Chromium, storage-state input, viewport/channel, CSS waits, selectors/exclusions, and cache bypass.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: scraping/content cleanup is a separable stage after rendered HTML is available.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` and `crawl4ai/html2text/`: markdown generation is a distinct HTML-to-markdown layer after cleanup.

The porting map for `aget` should keep those layers separate:

1. Transport/render layer: fetch a URL with explicit session state.
2. HTML processing layer: select/exclude/wait against page HTML.
3. Content conversion layer: produce markdown/html/text/json.
4. Finalization layer: artifacts, warnings, final URL, truncation, and stable errors remain in `src/extraction.rs`.

Do not port Crawl4AI's anti-bot/proxy/stealth paths, arbitrary JavaScript waits, LLM extraction, cache policy surface, or broad crawler features into this migration slice. Those are either outside the current `aget` dependency contract or conflict with the authorization-only safety boundary.

First implementation slice:

- Added `OwnedExtractorBackend` behind `ExtractorBackend`.
- It is not the default runtime backend yet; I19f owns the default switch.
- It initially used no new third-party dependencies and copied no upstream Crawl4AI code. The later D56 slice adds Rust dependencies for transport and parsing while preserving behavior-driven porting.
- It supports deterministic local/static HTTP extraction: redirects, cookie replay from structured `PlaywrightState`, text/html/json/markdown-as-text output, simple tag/id/class/tag.class selectors, comma-separated simple exclusions, CSS-only wait validation, artifact writes, and owned-backend error metadata.
- It deliberately does not claim browser-rendered parity yet: HTTPS/TLS, JavaScript rendering, localStorage replay through page scripts, richer CSS selectors, Crawl4AI-quality markdown/readability, screenshots, and real browser timeouts remain open for the next I19d/I19e slices.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli backend_parity_covers_extractor_content_formats`
- `cargo test --test aget_api`
- `cargo test --test get_cli get_real_helper_rejects_javascript_wait_before_crawl4ai_import`

Confidence: Medium. The first owned backend slice is narrow, local-only, and tested without Crawl4AI or `agent-browser`, but I19d remains in progress because browser-rendered extraction and localStorage-backed authenticated replay are not yet owned.

### D56: I19d replaces ad hoc owned extractor internals with Rust transport and CSS parsing crates

The second owned-extractor slice replaces the first slice's hand-rolled HTTP/selector implementation with focused Rust crates while keeping the same `ExtractorBackend` boundary:

- `ureq` v3.3.0 for blocking HTTP(S), redirects, and global request timeouts. License: MIT OR Apache-2.0.
- `scraper` v0.27.0 for HTML5 parsing and CSS selector matching. License: ISC.
- `html5ever` v0.39.0 as a direct dependency only for the `TreeSink` trait needed to detach excluded nodes from `scraper`'s parsed tree. License: MIT OR Apache-2.0.

The owned extractor now supports HTTPS-capable transport at the Rust layer and richer CSS selectors than the first static slice, including descendant/child/not-class selectors that are relevant to current `--selector`, `--exclude-selector`, and CSS-only `--wait-for-selector` behavior. It still does not execute page JavaScript and therefore does not replace Crawl4AI's browser-rendered SPA behavior yet.

Safety notes:

- Cookie replay remains explicit and scoped through `PlaywrightState`; secure cookies are only sent to HTTPS URLs.
- JavaScript wait strings are still rejected before selector parsing.
- Backend-specific `crawl4ai.*` options remain unsupported by the owned extractor until an `aget`-owned option namespace is designed; command-backed compatibility remains available.
- No Crawl4AI source or tests were copied in this slice. Upstream Crawl4AI was used only for architecture/source inspection recorded in D55.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. It closes the ad hoc selector/transport gap in the static owned extractor, but I19d remains open for JavaScript-rendered extraction, localStorage replay through page scripts, and markdown/readability quality.

### D57: I19d adds a first owned HTML-to-markdown slice

Before porting markdown behavior, I19d inspected Crawl4AI's markdown and cleaned-HTML flow:

- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py` selects the HTML source for markdown generation (`cleaned_html`, `raw_html`, or `fit_html`) after scraping/content processing.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` uses `DefaultMarkdownGenerator` plus `CustomHTML2Text`, then optionally converts links to citations and produces filtered/fit markdown.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py` removes excluded tags/selectors before cleaned HTML reaches markdown generation.
- `references/repos/crawl4ai/tests/async/test_markdown_genertor.py` and `tests/regression/test_reg_content.py` cover links/citations, content filters, and selector/exclusion behavior at a higher quality bar than the first `aget`-owned slice.

The Rust slice keeps the same layer boundary without copying Crawl4AI code. `OwnedExtractorBackend` now renders `OutputFormat::Markdown` through a small in-process DOM renderer instead of aliasing markdown to normalized text. The renderer currently handles the static/documentation structures `aget` tests directly: headings, paragraphs, emphasis, links/images, unordered/ordered lists, inline code, fenced code blocks, and blockquotes. Text output is unchanged and still uses normalized text.

License and dependency notes:

- The `html2md` Rust crate was rejected for this project because `cargo info html2md` reports GPL-3.0+.
- `ego-tree` v0.11.0 is now a direct dependency so the renderer can traverse the `scraper` DOM explicitly. License: ISC.
- No Crawl4AI source or tests were copied. The local Crawl4AI snapshot was used only to identify source-layer behavior and quality targets.

Remaining markdown/readability gaps are deliberate follow-ups: Crawl4AI-style citations/references, GFM tables, cleaned-main-content/readability pruning, fit markdown, media/link metadata, and broader edge-case parity. Browser-rendered JavaScript and localStorage-backed replay are still separate I19d/I19e gaps.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium for this slice. It replaces the most obvious markdown-as-text gap with tested structural markdown, but it is not yet a full Crawl4AI-quality markdown/readability replacement.

### D58: I19e starts with owned session-backed fallback extraction

Before porting the first `agent-browser` behavior, I19e inspected the local `agent-browser` snapshot paths that implement the current `aget get` fallback shape:

- `references/repos/agent-browser/cli/src/commands.rs`: parses `open`, `state load`, `get html body`, `get text body`, and `close` command shapes.
- `references/repos/agent-browser/cli/src/native/state.rs`: loads Playwright-style storage state by setting cookies and navigating to storage origins before setting local/session storage through CDP.
- `references/repos/agent-browser/cli/src/native/browser.rs`: navigates with CDP, tracks final page URL/title, and extracts DOM content through runtime evaluation.
- `references/repos/agent-browser/cli/src/native/actions.rs` and `native/element.rs`: implement `get html <selector>` as selected element `innerHTML` and `get text <selector>` as selected element text.

The current command fallback in `aget` uses this narrow sequence only after a session-backed primary extraction failure: load composed state into a temporary browser profile, open the URL, read body HTML/text, close the browser session, and clean up temp profile state. The first owned I19e slice therefore adds `OwnedBrowserAutomationBackend` with an owned fallback extraction path for static cookie-backed pages. It reuses the I19d owned fetch/HTML processing pipeline, uses structured `PlaywrightState` directly rather than an `agent-browser` state file, defaults fallback extraction to `body` to match the command fallback shape, and returns extractor metadata as `aget-owned-browser-fallback`.

This is intentionally not the full `agent-browser` replacement. `OwnedBrowserAutomationBackend` returns explicit unsupported-capability errors for Chrome/profile import and login start/finish/cancel until a real CDP/profile implementation is ported. It also does not execute page JavaScript or apply localStorage through a browser context, so localStorage-backed fallback pages still depend on future CDP/browser work.

Validation:

- `cargo test --test mock_site_cli owned_browser_fallback_replays_cookie_backed_session_without_agent_browser`

Confidence: Medium. The slice removes an `agent-browser` dependency path for static cookie-backed fallback extraction and is covered by deterministic MockSite evidence, but the high-risk browser automation work remains open.

### D59: I19d resolves owned markdown links against the page base URL

Before porting this behavior, I19d inspected Crawl4AI's link handling in `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`. Crawl4AI's `DefaultMarkdownGenerator` passes a base URL into its HTML-to-markdown converter and resolves relative markdown links before building citation references. Its tests in `references/repos/crawl4ai/tests/async/test_markdown_genertor.py` cover relative links and image URLs against a supplied base URL.

The owned renderer now resolves link and image URLs with the Rust `url` crate. It uses the final fetched URL as the default markdown base and honors an HTML `<base href="...">` element before extraction, matching Crawl4AI's separation between cleaned HTML and markdown generation. This keeps markdown output agent-ready when a selected content block contains relative links.

License and dependency notes:

- `url` v2.5.8 is now a direct dependency for standards-based URL joining. License: MIT OR Apache-2.0.
- No Crawl4AI source or tests were copied; the deterministic MockSite route was authored in this repo from the observed behavior target.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. It closes a concrete markdown parity gap with primary-source behavior inspection and local deterministic coverage; citation/reference formatting and broader readability quality remain open.

### D60: I19e adds a minimal owned CDP renderer for localStorage-backed fallback

Before porting this behavior, I19e inspected `agent-browser`'s storage-state and CDP paths again:

- `references/repos/agent-browser/cli/src/native/state.rs` loads Playwright-style cookies first, then navigates to each storage origin and sets `localStorage`/`sessionStorage` through `Runtime.evaluate` before the target page is opened.
- `references/repos/agent-browser/cli/src/native/element.rs` extracts element HTML through CDP after resolving the selector.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` launches Chrome with a temporary user data directory, waits for `DevToolsActivePort`, and cleans up the temp profile after shutdown.

`OwnedBrowserAutomationBackend` now keeps the fast static fallback for cookie-only sessions but switches to a minimal owned Chrome/CDP renderer when composed state contains localStorage origins. The CDP path launches a temporary local Chrome profile, attaches to a page target, sets cookies with `Network.setCookies`, navigates each localStorage origin to set storage, opens the requested URL, optionally waits for a CSS selector through an internally generated `document.querySelector(...)` expression, then feeds the rendered document HTML back into the owned extraction/formatting pipeline. This preserves the safety rule that user-provided JavaScript waits are not executed; user input is still limited to CSS selectors and is JSON-quoted inside agent-owned CDP expressions. Owned Chrome profile temp dirs are removed on normal shutdown and are now included in orphan sweeping.

Dependency note:

- `tungstenite` v0.29.0 is now a direct dependency for the blocking local CDP WebSocket transport. License: MIT OR Apache-2.0.

Remaining I19e gaps are still substantial: Chrome/profile import, login start/finish/cancel lifecycle, current-tab attach, richer process diagnostics, screenshot/debug artifacts, and broader rendered-SPA parity. The new CDP path is intentionally scoped to fallback extraction with explicit session state and a throwaway profile.

Validation:

- `cargo test browser_cdp`
- `cargo test session::store::tests::orphan_sweep`
- `cargo test --test mock_site_cli owned_browser_fallback_replays_cookie_backed_session_without_agent_browser`
- `cargo test --test mock_site_cli owned_browser_fallback_renders_local_storage_backed_session_with_chrome -- --ignored` passed locally with system Chrome and confirmed localStorage-driven rendered DOM extraction.
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The CDP command construction and cookie-only fallback path are covered by deterministic tests, and the ignored local Chrome smoke test passed on this machine. Full migration confidence still requires more lifecycle/error-path coverage before switching defaults.

### D61: I19d uses owned CDP rendering for localStorage-backed primary extraction

The D60 renderer exposed a follow-up correctness gap: once `OwnedExtractorBackend` becomes default, a localStorage-backed request could otherwise return a static app shell successfully and never invoke browser fallback. I19d now routes owned primary extraction through the same temporary Chrome/CDP renderer whenever composed session state contains localStorage origins. Cookie-only extraction stays on the static HTTP path.

This still does not make every JavaScript-heavy cookie-backed page render through Chrome; there is no reliable generic signal for that yet. The new rule only covers the explicit structured-state case where static HTTP cannot replay localStorage at all.

Validation:

- `cargo test --test mock_site_cli owned_extractor_backend_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_browser_fallback_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The local Chrome smoke test proves rendered DOM extraction for the primary owned backend on this machine; broader rendered-JavaScript default policy remains an open I19d/I19f decision.

### D62: I19d retries owned extraction through CDP when CSS waits need rendered DOM

I19d now covers a second narrow rendered-JavaScript case without adding public API: when the owned static HTTP path cannot find a requested CSS `--wait-for-selector`, it retries the same extraction through the owned Chrome/CDP renderer. This mirrors the current Crawl4AI contract for CSS waits while preserving the existing safety boundary: JavaScript wait expressions are still rejected, and the only user input evaluated in Chrome is a JSON-quoted CSS selector passed to `document.querySelector(...)`.

This is deliberately not a blanket browser-rendering default. Static pages with matching selectors still stay on the faster HTTP path, and JavaScript-heavy pages without a wait selector remain a future policy/default decision for I19f.

Validation:

- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The ignored Chrome smoke test proves the delayed-DOM wait path locally, and the normal suite keeps static wait-selector behavior covered without requiring Chrome.

### D63: I19d adds first owned markdown table rendering

Before this slice, the owned markdown renderer collapsed HTML tables into plain text. Crawl4AI's markdown behavior and tests treat tables as part of the markdown-quality target, so the owned renderer now emits GitHub-flavored markdown tables for static table structures. Header rows are detected from `<th>` cells; tables without explicit headers use the first row as the markdown header. Cell content goes through the same inline renderer as normal text, so links are still resolved against the page base URL, and pipe characters inside cells are escaped.

This is not a full markdown/readability replacement yet. Remaining quality gaps still include captions, complex row/column spans, Crawl4AI-style citations/references, fit markdown, and broader main-content cleanup.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The static parity test now covers a table with links and literal pipe characters, but complex table semantics remain an explicit follow-up.

### D64: I19e adds bounded owned Chrome profile-path import

Before porting this import slice, I19e inspected the `agent-browser` state/profile paths that back the current command adapter:

- `references/repos/agent-browser/cli/src/native/state.rs` exports Playwright-style storage state by reading cookies through CDP and collecting local/session storage from the current page plus known origins.
- `references/repos/agent-browser/cli/src/native/cookies.rs` uses `Network.getAllCookies` and URL-scoped cookie reads as the browser-state bridge.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` launches Chrome with a user-data-dir, removes stale `DevToolsActivePort` files before launch, copies named Chrome profiles to temporary directories, and uses the real keychain only for copied named-profile imports.

`OwnedBrowserAutomationBackend::import_chrome` now has a bounded owned path for explicit profile directories. It launches local Chrome against the supplied user-data-dir path, exports cookies and localStorage through the in-process CDP client, filters the resulting Playwright-style state through the same allowlist/provenance logic used for command-backed `agent-browser` state, and persists only scoped session material. LocalStorage collection uses request interception to load blank same-origin documents for allowed domains, avoiding real network requests while still reading origin storage.

This deliberately does not claim parity with named Chrome profiles such as `Default`. Named profile import still needs the higher-risk `agent-browser` behavior: resolving Chrome's user-data-dir, copying only the selected profile plus `Local State`, preserving macOS/OS keychain behavior where needed, diagnosing locked/running profiles, and cleaning copied profiles. For now, the owned backend returns a structured backend-unavailable error for named profiles and leaves the command-backed adapter as the supported path.

Validation:

- `cargo test browser_cdp`
- `cargo test session::chrome`
- `cargo test session::agent_browser`
- `cargo test --test aget_api owned_browser_backend_reports_named_profile_import_gap`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile -- --ignored` passed locally with system Chrome, proving a temporary user-data-dir profile can persist a cookie/localStorage pair and be re-imported through the owned CDP export path.

Confidence: Medium. The explicit profile-path slice is now owned and tested, but full `agent-browser` import parity remains open until named real-profile copy/keychain/lock behavior is ported.

### D65: I19e ports named Chrome profile resolution and copy setup

The next I19e Chrome-import slice ports the named-profile setup that `agent-browser --profile Default` depends on. The relevant source remains `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`, especially `get_chrome_user_data_dirs`, `find_chrome_user_data_dir`, `list_chrome_profiles`, `resolve_chrome_profile`, `copy_chrome_profile`, and `copy_dir_recursive`.

`OwnedBrowserAutomationBackend::import_chrome` now treats profile arguments without path separators as Chrome profile names. It finds a Chrome user-data directory with `Local State` (or `AGET_CHROME_USER_DATA_DIR` for deterministic tests), resolves the requested profile by exact directory, display name, or case-insensitive directory, copies `Local State` plus the selected profile subdirectory into a private temporary user-data-dir, skips large/cache/lock directories and files, launches Chrome with `--profile-directory=<resolved>`, exports scoped cookies/localStorage through CDP, and removes the temporary profile copy on drop. The copied-profile launch avoids the mock keychain flags so real Chrome profile imports have the same keychain shape as the upstream command adapter. Orphan sweeping now also removes stale `tmp/owned-chrome-import/aget-profile-*` copies.

This still needs a real logged-in profile smoke before claiming full parity with user Chrome `Default` imports. The deterministic tests cover profile resolution, ambiguous/missing profile errors, copy exclusions, private temp directory permissions, failure-to-save behavior, and a local Chrome smoke for the `--profile-directory` CDP export path. They do not prove macOS/OS keychain cookie decryption against the user's real Chrome profile or profile-lock classification for an actively running browser.

Validation:

- `cargo test session::chrome`
- `cargo test browser_cdp`
- `cargo test session::store::tests::orphan_sweep`
- `cargo test --test aget_api owned_browser_backend_does_not_save_failed_profile_path_import`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored` passed locally with system Chrome, proving the CDP export path works when Chrome is launched with a selected profile directory.

Confidence: Medium. The named-profile setup is now owned and deterministically covered, but real-profile auth/keychain and lock/error classification remain higher-risk I19e follow-ups.

### D66: I19e adds a first owned dedicated-profile login lifecycle

Before porting the login lifecycle, I19e inspected the `agent-browser` command and native paths that back the current `aget session login start|finish|cancel` flow:

- `references/repos/agent-browser/cli/src/commands.rs`: parses `open`, `state save <path>`, and `close`.
- `references/repos/agent-browser/cli/src/native/browser.rs`: sends `Browser.close` only for locally launched browsers, so external/current-browser connections are not shut down accidentally.
- `references/repos/agent-browser/cli/src/native/state.rs`: exports Playwright-style cookies and origin storage through CDP, including blank-response origin visits for storage collection.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: launches Chrome with an explicit `--user-data-dir`, uses mock keychain flags by default, omits headless mode for visible browser flows, and reads `DevToolsActivePort` for CDP attachment.

`OwnedBrowserAutomationBackend` now implements `start_login`, `finish_login`, and `cancel_login` for dedicated `aget` profiles without shelling out to `agent-browser`. Start creates a private `tmp/owned-login/aget-<name>` profile by default, launches visible Chrome at the caller-provided HTTPS URL, records pending login metadata, and leaves Chrome running for user-driven auth. Finish connects to the running profile browser through `DevToolsActivePort` when available, exports cookies/localStorage for only the pending flow's allowed domains, closes the browser, filters the state through the existing session allowlist/provenance logic, and still returns `requires_user_action` when no scoped auth state is found. If the user already closed the browser, finish falls back to a headless launch against the same dedicated profile to export state. Cancel closes the running profile browser when reachable and cleans pending metadata plus tool-owned login profiles; custom profile paths are preserved.

This slice preserves the public login API and the existing `SessionSource::AgentBrowser` shape for compatibility, even though the implementation is now owned. It does not copy upstream code. The remaining parity gaps are real manual login smoke coverage, richer process diagnostics, Windows/profile-lock behavior, current-tab attach, and complete `requires_user_action` classification for all Chrome startup/export failures.

Validation:

- `cargo test session::login::tests`
- `cargo test --test aget_api owned_browser_backend_cancels_pending_login_without_agent_browser`
- `cargo test --test aget_api owned_browser_backend`
- `cargo test browser_cdp`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`

Confidence: Medium-high. The deterministic tests cover ownership boundaries, cleanup, public backend wiring, and CDP payload helpers; the ignored headed Chrome smoke test now passes locally for disposable profile start/export/close. A real user-authorized site login has still not been run in this slice.

### D67: I19e sweeps stale owned-login profiles without deleting pending flows

The D66 lifecycle introduced `tmp/owned-login/aget-<name>` profile directories for owned dedicated login sessions. Startup orphan sweeping now includes this root, but only removes an `aget-<name>` profile when the matching `tmp/login-<name>.json` pending metadata file is absent and the profile is older than the sweep threshold. This keeps active or paused user login flows intact while still cleaning stale profiles left by crashes, manual file deletion, or failed handoffs.

Validation:

- `cargo test session::store::tests::orphan_sweep`
- `cargo test session::login::tests`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`

Confidence: High for this cleanup slice. The behavior is narrow and deterministic; broader lifecycle/process parity remains covered by D66's open gaps.

### D68: I19e hardens owned-login process cleanup after headed Chrome smoke

The ignored headed Chrome smoke initially proved state export but exposed a lifecycle flaw: `Browser.close` could return while the detached visible Chrome process was still alive, and immediate profile removal could race or leave a disposable browser process behind. The owned login start path now records the launched browser PID in pending login metadata. Finish and cancel pass that PID into the CDP close path, wait for the process to exit after `Browser.close`, and only terminate the PID/process group as a fallback when the process command line still references the expected profile path.

This keeps process cleanup scoped to `aget`-launched dedicated login browsers. Command-backed `agent-browser` flows still have no PID in pending metadata and keep their existing close behavior. Pending metadata remains backward-compatible because `browser_pid` defaults to `None` when older pending files are read.

Validation:

- `cargo test browser_cdp::tests::owned_login_browser_exports_state_from_headed_profile_and_closes -- --ignored`
- `cargo test session::login::tests`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`
- Manual process check after the ignored smoke: no remaining `login-profile`, `owned-login`, or `remote-debugging-port=0` test Chrome process.

Confidence: Medium-high. The specific detached-process leak is now covered by a real local Chrome smoke and guarded cleanup logic, but broader cross-platform process behavior still needs Windows/Linux verification before I19e can be considered complete.

### D69: I19d adds conservative default main-content selection

Before this slice, owned extraction without an explicit selector formatted the parsed document root for text, markdown, and JSON content. That kept the implementation simple, but it meant default agent-facing output could include header, global nav, sidebar, or footer chrome even when the page had a single obvious content container.

I19d inspected Crawl4AI's cleaned-content flow again before changing this behavior:

- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py` removes excluded tags/selectors, builds a `content_element` from `css_selector` or `target_elements` when supplied, and serializes that cleaned element.
- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py` feeds `cleaned_html` into markdown generation by default.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` treats markdown generation as a separate layer over the selected cleaned HTML.

The owned backend now applies a conservative default content heuristic only when there is no explicit selector, no browser-fallback selector, no CSS wait selector, and the requested output format is text, markdown, or JSON. It prefers a unique `main`, then a unique `[role="main"]`, then a unique `article`, then falls back to `body` or the root document. HTML output without a selector still returns the cleaned document shape for debugging/compatibility, and explicit selectors keep their existing exact behavior.

This is not full readability pruning. It does not score competing article candidates, remove in-content nav, generate Crawl4AI citations, or apply fit-markdown filtering. It is a narrow default cleanup that reduces obvious page chrome while keeping wait-driven rendered extraction from accidentally discarding the waited element.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. The behavior is deterministic and covered by a local fixture; broader readability quality remains an explicit I19d gap.

### D70: I19d supports safe `crawl4ai.excluded_tags` in the owned extractor

The I19c parity matrix keeps the current namespaced Crawl4AI backend-option surface alive until an `aget`-owned option namespace is designed. Before this slice, `OwnedExtractorBackend` rejected every backend option, which would make a default switch fail even for safe cleanup options that do not execute JavaScript or require browser-specific timing.

Before porting the option, I19d inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: Crawl4AI reads `excluded_tags` from the run config, removes matching tag elements before selector-based content selection and cleaned-HTML serialization, then passes that cleaned HTML into the later markdown layer.

The owned backend now accepts only `crawl4ai.excluded_tags` from the backend-option escape hatch. The value is parsed like the helper's comma-separated list, but each entry must be a plain HTML tag name so the option cannot become a general CSS selector injection path. The removal happens before explicit `--exclude-selector` and before owned text/markdown/json/html formatting. Unsupported backend options still fail explicitly and point callers at the single supported option.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This closes one safe backend-option parity gap with deterministic coverage. At this slice, other Crawl4AI options such as `only_text`, `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, and `wait_for_images` remained unsupported until they had clear owned semantics.

### D71: I19d supports safe `crawl4ai.target_elements` in the owned extractor

The next safe backend-option slice ports `crawl4ai.target_elements`, again without copying upstream code. Crawl4AI's `content_scraping_strategy.py` applies `target_elements` after optional `css_selector` narrowing by collecting matches from the current content source and serializing only those elements into cleaned HTML. This option is selector-based content narrowing, not JavaScript execution.

The owned backend now accepts `crawl4ai.target_elements` as a comma-separated CSS selector list. Selectors are parsed before fetching or rendering so invalid CSS fails early. If a normal `--selector` is also present, target selectors are evaluated inside that selected source; otherwise they are evaluated against the parsed document root. Markdown rendering now includes the selected element itself rather than only its children, so targeting a heading preserves heading syntax instead of flattening it to plain text.

Unsupported backend options still fail explicitly. At this slice, the owned backend supports `crawl4ai.excluded_tags` and `crawl4ai.target_elements` from the Crawl4AI namespace.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This closes another deterministic option-parity gap. At this slice, it intentionally did not port `only_text`, `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, or `wait_for_images` yet.

### D72: I19d renders script-bearing pages through owned CDP by default

Before this slice, the owned extractor rendered through Chrome only for localStorage-backed session state or when a CSS wait selector was missing from the static HTML. That still left a major Crawl4AI parity gap: ordinary JavaScript-rendered pages without an explicit wait selector could return a static app shell successfully and never reach the owned CDP renderer.

Before changing the policy, I19d inspected Crawl4AI's render timing in `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py` and `references/repos/crawl4ai/crawl4ai/async_configs.py`. Crawl4AI navigates in a browser, applies any configured `wait_for`, then waits `delay_before_return_html` before reading final HTML; the default delay is 0.1 seconds.

The owned extractor now validates owned options first, performs the fast Rust HTTP fetch, and escalates to the owned CDP renderer when the static response contains executable script tags. The CDP renderer now also waits 100 ms after navigation and any CSS wait before reading `document.documentElement.outerHTML`, matching Crawl4AI's default pre-return delay at a narrow level. This keeps static pages on the fast path while covering a concrete class of client-rendered pages without requiring callers to guess a wait selector.

This is still not full smart load detection. It does not wait for network idle, long async chains, virtual scrolling, image readiness, or app-specific readiness signals. Pages with scripts now require local Chrome/Chromium when using the owned backend, which is acceptable before I19f but needs to be reflected in default-runtime docs before the switch.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_scripted_page_without_wait_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`

Confidence: Medium. The local Chrome smoke proves the new no-wait script rendering path, but the readiness heuristic remains intentionally simple and should be expanded or documented before making the owned backend the default.

### D73: I19d ports `crawl4ai.delay_before_return_html` to owned CDP rendering

D72 hard-coded Crawl4AI's default 0.1 second pre-return delay in the owned CDP renderer. The next small parity step makes the public `crawl4ai.delay_before_return_html` backend option work on the owned backend as well. The source behavior remains `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI sleeps after `wait_for` and before retrieving final HTML, and `references/repos/crawl4ai/crawl4ai/async_configs.py`, where the default is 0.1 seconds.

The owned extractor now parses `crawl4ai.delay_before_return_html` as a non-negative finite number of seconds, defaults to 0.1 seconds, and passes the resulting duration into `browser_cdp::render_page`. The delay is applied after navigation and any CSS wait selector, immediately before reading `document.documentElement.outerHTML`. Unsupported backend options still fail explicitly; the supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, and `delay_before_return_html`.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_honors_render_delay_option_with_chrome -- --ignored`

Confidence: Medium-high. The option is narrow, typed, and covered by an ignored Chrome smoke with a delayed client render; broader smart readiness remains open.

### D74: I19f starts the owned-backend default runtime switch

The migration reached the point where the default facade was still the biggest contradiction: `Aget` and `get_url` still defaulted to command-backed Crawl4AI and `agent-browser` even though the owned extractor and owned browser automation paths now cover the current PoC feature set at the capability boundary.

I19f now switches the default runtime wiring to owned backends:

- `Aget::new` defaults to `OwnedExtractorBackend` plus `OwnedBrowserAutomationBackend` through small default-backend enums.
- `Aget::from_env` keeps compatibility by selecting command-backed adapters only when `AGET_CRAWL4AI_COMMAND` or `AGET_AGENT_BROWSER_COMMAND` is explicitly set.
- `get_url` and `get_url_with_backend` now use the owned browser fallback by default instead of `agent-browser`.
- The command adapters remain available for compatibility tests and explicit developer runs.

Documentation was updated so README, `.opencode/tools/aget.ts`, and `.cursor/skills/aget/SKILL.md` no longer present Crawl4AI or `agent-browser` as required default dependencies. The current default runtime still needs local Chrome/Chromium for JavaScript-rendered pages, Chrome import, and login flows. There is no implemented `aget doctor` command in this snapshot, so no doctor code surface required an update.

Validation:

- `cargo test --test aget_api`
- `cargo test --test get_cli get_json_success_writes_run_artifacts_with_empty_state`
- `cargo test --test get_cli missing_backend_returns_backend_unavailable`
- `cargo test --test get_cli get_session_backend_failure_redacts_state_secrets_from_errors_metadata_and_artifacts`
- `cargo test --test mock_site_cli default_cli_fetch_uses_owned_backend_without_command_dependencies`
- `env -u AGET_CRAWL4AI_COMMAND -u AGET_AGENT_BROWSER_COMMAND PATH="/Users/nicolas/.cargo/bin:/usr/bin:/bin" cargo test --test mock_site_cli default_cli_fetch_uses_owned_backend_without_command_dependencies`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`

Confidence: Medium-high. This is the first default-runtime switch, but full standard validation passed and the no-command-path smoke proves the default public fetch path does not need Crawl4AI or `agent-browser` on PATH. Remaining dependency cleanup is tracked by I19g because compatibility adapters, old temp-file naming, and historical research notes still intentionally mention the PoC tools.

### D75: I19g demotes command adapters by removing implicit PATH defaults

After I19f made owned backends the default facade, the remaining production-shaped leak was inside the compatibility adapters themselves: constructing a command-backed extractor or browser adapter could still fall back to repo/PATH defaults (`scripts/crawl4ai_extract.py` via `uv run --with crawl4ai`, or `agent-browser`) when no explicit command was provided.

I19g removes those implicit defaults. The command-backed extractor now requires either an API-provided command or `AGET_CRAWL4AI_COMMAND`. The command-backed browser/session adapter and command-backed browser fallback now require `AGET_AGENT_BROWSER_COMMAND`. This keeps fake-command and explicit developer compatibility coverage available, while preventing the old PoC wrappers from being selected accidentally.

The public docs were tightened around that boundary:

- README now says optional command compatibility requires `AGET_CRAWL4AI_COMMAND` or `AGET_AGENT_BROWSER_COMMAND`.
- The project aget skill says `backend_unavailable` may refer to an explicitly configured compatibility backend.
- The CLI help for Chrome import now describes the owned local Chrome/CDP path, not `agent-browser`.

Repository hygiene check: `.gitignore` still covers `target/`, `.env*`, `references/repos/`, `.aget/`, logs, local Firecrawl output, and benchmark/private-output folders. No dependency clone, run artifact, raw browser state, or authenticated benchmark output was added in this slice. `AGET_BACKEND_COMMAND` is not present in current source.

Validation:

- `cargo check`
- `cargo test --test get_cli get_noisy_backend_output_does_not_deadlock`
- `cargo test --test get_cli`
- `cargo test --test session_cli`
- `cargo test --test mock_site_cli`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`
- `! rg -n 'default_command|scripts/crawl4ai_extract.py|uv run --with crawl4ai|unwrap_or_else\(\|_\| "agent-browser"|AGET_BACKEND_COMMAND' src README.md .cursor/skills/aget/SKILL.md .opencode/tools/aget.ts`

Confidence: High for the demotion slice. The source audit confirms the implicit command defaults are gone from live code, focused command-adapter suites still pass with explicit env configuration, and the full standard suite passes with owned defaults intact.

### D76: I19h local final migration audit starts

I19h began after the owned-default and command-demotion commits. The local audit has enough evidence that the default migration is stable for deterministic and local Chrome-backed coverage, but the task cannot be marked complete yet because its acceptance criteria explicitly require review subagents for test adequacy, architecture cohesion, and security/privacy. This Codex session can only spawn subagents when the user explicitly asks for them, so that acceptance item remains pending rather than simulated.

Validation run during the local audit:

- `cargo test`
- `cargo test --test mock_site_cli owned_ -- --ignored`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored`

Manual/ignored checks intentionally not run in this pass:

- `browser_cdp::tests::owned_login_browser_exports_state_from_headed_profile_and_closes -- --ignored`, because it opens a visible browser window.
- Real `agent-browser`/Crawl4AI/HelloInterview and cmux ignored tests, because those depend on optional external tools, running local surfaces, or manual authorized site login and are no longer default-runtime requirements.

Tracked-file hygiene audit:

- `git ls-files references references/repos .aget target 'workpads/research/benchmarks/r0a' 'workpads/research/benchmarks/crawl4ai-skill-*' '.firecrawl'` only reported the tracked helper script `workpads/research/benchmarks/crawl4ai-skill-minimal.py`; no dependency clone, `.aget` run artifact, target output, private R0a benchmark output, or Firecrawl output is tracked.
- `git ls-files | rg -n '(^|/)(agent-browser-hi-auth-state\.json|raw-state|backend-stdout|backend-stderr|\.aget|references/repos|workpads/research/benchmarks/r0a|\.log$)' || true` found no tracked raw state, backend artifact, local run directory, dependency clone, private benchmark directory, or log file.
- `git status --ignored --short` shows the expected ignored local directories (`references/`, `target/`, `.firecrawl/`, `.opencode/node_modules/`, benchmark output dirs) and the pre-existing untracked `CLAUDE_REVIEW.md`.

Confidence: Medium-high for the local audit evidence. The remaining I19h blocker is review coverage, not deterministic validation.
