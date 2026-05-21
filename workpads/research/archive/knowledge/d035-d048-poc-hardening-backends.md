### D35: I8b-followup confirms extraction is generic again

The I8b-followup pass searched the active source and tests for `HelloInterview`, `hellointerview`, paywall markers, premium-content markers, and site-specific retry language. No hostname/content matching or site-shaped CTA remains in `src/`; `aget get` reports generic extraction success/failure and leaves interpretation of login walls, paywalls, or gated content to the calling agent or future agent skill.

The remaining HelloInterview references are historical benchmark notes, representative manual e2e names, and future agent-skill examples. They are not binary behavior. I8b itself remains open because the manual authorized real-site login/fetch acceptance criterion is still unverified.

Verification for this pass:

- `rg "HelloInterview|hellointerview|paywall|Purchase Premium|Premium users|Sign in / Sign up|site-specific|site specific" src`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`

### D36: I8c adds a project-local agent skill for safe aget flows

I8c adds `.cursor/skills/aget/SKILL.md` as the project-local agent guide for using `aget`. The skill covers the current core flows: empty-session fetch, large/sensitive output to artifacts, user-driven login start/finish, retrying a gated page with a caller-chosen session, multi-session request-time replay, persisted session composition, cmux cookie import, and Chrome import through `agent-browser`.

The guide intentionally keeps site reasoning outside the binary. It uses HelloInterview as the first representative authorized gated-site example, then generalizes the same pattern to `ft.com`, `nytimes.com`, private docs, dashboards, and account pages. It tells agents to interpret returned content themselves, ask for explicit user action before login, never handle credentials, and avoid bypassing access controls or site policy.

Focused review tightened the skill in three places: existing-browser auth imports now require explicit approval for the surface/profile and domains, login-time session composition is documented as not directly supported by `session login start`, and extraction-tuning advice is kept as a pointer to the README rather than expanded in the skill.

Verification for this pass:

- Read `.cursor/skills/aget/SKILL.md` and checked it is under 500 lines.
- `rg "credentials|bypass|HelloInterview|ft\\.com|nytimes\\.com|session login start|session compose|import cmux|import chrome|--envelope" .cursor/skills/aget/SKILL.md`
- `cargo test`
- `git diff --check`

### D37: I8d consolidates extractor and agent-browser glue

I8d introduces `src/session/agent_browser.rs` as the shared home for agent-browser state JSON parsing, process execution, failure classification, domain/origin filtering, raw-state temp files, and private raw-state permissions. Chrome import and login finish now call the same filtering and process helpers instead of maintaining parallel implementations.

`aget get` now has a smaller extraction boundary: `run_primary_extractor` converts Crawl4AI backend output into a `SuccessfulExtraction`, `try_session_fallback` owns the session-backed agent-browser fallback, `finalize_success` owns content limits/artifact metadata, and `finalize_error` owns error metadata. This removes the previous duplicated success/failure/fallback finalization branches inside `get_url`.

Cookie identity now normalizes cookie name whitespace, leading-dot/lowercase/trailing-dot domains, and empty paths consistently for both request-time Playwright state composition and persisted `session compose`. Composed cookie output uses the same canonical name/domain/path fields, and regression tests cover normalized cookie deduplication in both paths.

### D38: I9 starts as project-local OpenCode custom tools

OpenCode supports project-local custom tools in `.opencode/tools/` using TypeScript definitions from `@opencode-ai/plugin`. I9 uses that path instead of a packaged npm plugin because the MVP only needs a thin local wrapper around the `aget` CLI and should keep Rust CLI behavior as the source of truth.

The initial OpenCode tools are `aget_fetch`, `aget_session_list`, and `aget_session_inspect`, exported from `.opencode/tools/aget.ts`. They call `aget --json` and return the structured envelope unchanged. `aget_fetch` exposes sessions and output-shaping arguments, including `max_chars: 0` for callers that want no inline page content, but does not infer ambient browser auth. `aget_session_list` returns local session names. `aget_session_inspect` intentionally omits `--show-secrets`, so tool output remains redacted unless a future explicit sensitive-inspection flow is designed.

### D39: I10 hardens session replay and subprocess boundaries

I10 adds replay-time scope checks before selected sessions are converted into Playwright state. A request with `--session <name>` now fails with `privacy_policy_blocked` if that session has no scope matching the requested host, or if it contains stored cookie/storage state for any unrelated host. This prevents accidentally loading credential-equivalent browser state from a broad or composed session into unrelated request targets. There is no override flag yet; cross-site/provider workflows must use sessions whose saved stored state matches the target URL or wait for a deliberately designed override.

Backend subprocesses now run with a minimal allowlist environment and bounded waits. Crawl4AI, agent-browser fallback, session import/login agent-browser calls, and cmux cookie import no longer inherit the full parent shell environment. Timeout termination uses a bounded post-termination wait instead of an unbounded `wait`, and agent-browser/cmux stdout/stderr are written to private temp files instead of un-drained pipes.

Sensitive artifact cleanup is tighter: backend stderr/stdout redaction now covers literal, upper/lowercase percent-encoded, form-encoded, and JSON-escaped cookie/storage values, replacing longer overlapping values first. Cookie and storage names remain visible because they are treated as provenance/debug metadata rather than bearer secrets. `SessionStore` startup sweeps old orphaned raw-state/temp output files and fallback profiles, and successful/cancelled login flows remove the default tool-owned agent-browser login profile even when cancel close fails. The structured envelope still embeds `data.content`; for sensitive fetches, callers should use `--max-chars 0` or `--out` with awareness that `--out` does not currently suppress inline content.

### D40: Mocked e2e site server is worth adding

The project has many focused fake-backend and small local-server tests, but it still lacks a reusable site-shaped fixture that exercises the end-to-end product behavior across login state, browser storage, redirects, JavaScript rendering, wait conditions, noisy page chrome, output shaping, and replay-scope privacy checks. A mocked e2e site server is worth adding because it can make most auth/session regressions deterministic without relying on HelloInterview, FT, NYT, cmux, Chrome profile state, or live network conditions.

The fixture should be generic and local-only: a small Rust test support server with deterministic routes such as public content, login form/callback, protected account/docs pages, localStorage-token pages, delayed JS content, redirect chains, expired-session responses, logout, and multi-host/provider-style scenarios where feasible. Tests should be able to seed sessions, inspect received cookies/headers, and assert that unrelated credentials were not replayed. This fixture should complement, not replace, lower-level fake-backend unit tests and ignored/manual real-site verification.

### D41: I11 adds deterministic mocked-site CLI coverage

I11 adds `tests/support/mock_site.rs` as a reusable local site fixture and `tests/mock_site_cli.rs` as the first e2e-style test suite using it. The fixture exposes generic routes for public content, protected cookie-backed content, simulated localStorage-token content, two-cookie same-site composition, redirects, delayed JavaScript-like readiness, login/callback, logout, and expired sessions. It records received requests so tests can assert which cookies and headers were replayed.

The mocked-site tests still call the real `aget` binary. A fake Crawl4AI-compatible backend reads the temporary Playwright state file, replays matching cookies to the mock site, simulates a page script reading localStorage before fetching a protected API route, applies simple selector/exclusion/wait behavior, writes artifacts, and returns the normal backend JSON. A fake `agent-browser` backend lets `session login start|finish` complete without manual interaction, then the saved session is used against the protected mock page.

Use this fixture for deterministic auth/session integration coverage where manual real-site tests would be flaky or require credentials. Keep small fake-backend/unit tests for narrow edge cases, and keep ignored/manual live-site checks only as local confidence tests for real external backends.

Review follow-up: the fake-backend mocked-site tests now assert exclusion against in-main noise, simulate localStorage through a page-script-style API fetch, verify mixed-scope rejection does not reach the server, and exercise unauthenticated, expired, and logout states. They still do not prove real Crawl4AI/Playwright JavaScript execution semantics; Task I14 tracks an opt-in real-backend smoke test against the same local fixture.

### D42: Documentation-style e2e tests should be declarative and Rust-owned

I15 improves the mocked-site e2e direction by removing generated inline backend scripts from `tests/mock_site_cli.rs`. Test backend behavior now lives in checked-in Rust helper binaries, `aget-mock-backend` and `aget-mock-agent-browser`, owned by the dev-only fixture crate at `tests/fixtures/mock-tools`. The e2e test file can focus on product behavior instead of embedding a second implementation as a string, and the helper tools no longer appear as product-visible root Cargo binaries.

The first documentation-style layer uses a small `GetSpec` harness so tests can read as route/config/result scenarios: choose a mock-site path, configure output options such as format/selectors/out/max-chars, run the real `aget` binary, and assert the structured JSON envelope. The helper tools are strict about supported argv shapes so they exercise the external backend and agent-browser process boundary without accepting accidental drift. This better matches the desired API-documentation feel while preserving process-level coverage of the CLI, state files, artifacts, session storage, and backend command boundary.

Follow-up: `MockSite` now supports test-defined routes through `MockSite::builder().route(path, MockResponse::html(...)).start()`, so extraction-oriented tests can declare custom pages directly in Rust and let the site stop on drop. Legacy tests in `tests/get_cli.rs`, `tests/session_cli.rs`, and `tests/cli.rs` still contain generated inline Python shims; Task I16 tracks removing those in favor of declarative Rust fixtures and checked-in dev-only helper tools.

### D43: `Aget` is the library facade for API-style tests and CLI reuse

The e2e-style API tests now use a production `Aget` facade instead of a test-only `GetSpec`: `Aget::new(home).with_backend_command(...).get(url).format(...).selector(...).run()`. This keeps tests close to the API agents should eventually call, avoids global `AGET_HOME`/backend env mutation in library-level tests, and still exercises the real extraction pipeline, session store, artifact writing, and backend process boundary.

The CLI `get` path now constructs `Aget` internally rather than calling `get_url` directly. `get_url(GetOptions)` remains available as the lower-level compatibility function, but the intended higher-level API surface is `Aget`.

### D44: Comments should clarify project vocabulary and boundaries

Project workflow now calls for short comments when local naming is not enough to explain a concept, boundary, or invariant. This applies especially to `backend`, `extractor`, `session`, `profile`, `artifact`, and `envelope`, because those terms can refer to local subprocesses, browser state, persisted auth data, files, or public API shapes depending on context. Comments should explain the boundary or contract, not restate the code.

### D45: `Aget` should depend on pluggable capability backends

The current `backend_command` field is a PoC leak: Crawl4AI happens to be reached through an external command today, but the real `Aget` boundary should be "extract this URL with these sessions/options," not "run this command." Future refactoring should introduce pluggable internal backends for extraction, browser automation, and session persistence. The command-backed Crawl4AI and `agent-browser` integrations should become adapters behind those boundaries, so they can later be replaced by in-process Rust implementations without changing the public `Aget` API or CLI concepts.

The same pattern should apply to session handling: the filesystem `SessionStore` remains the default local-first implementation, but `Aget` should depend on a session-store capability so tests, alternate storage, encryption-at-rest, or future profile/session implementations can be swapped in deliberately.

### D46: `Aget` backend pluggability uses static dispatch by default

`Aget` now aliases `AgetWith<CommandExtractorBackend, FilesystemSessionStoreBackend, CommandBrowserAutomationBackend>`. The generic form keeps extractor, session-store, and browser-automation/fallback capabilities swappable without `Arc<dyn ...>` or runtime dispatch in the normal path. Tests and future implementations can replace one backend at a time through typed builder methods while the CLI keeps using the default `Aget` alias.

The current command-backed adapters remain explicit PoC boundaries: Crawl4AI-compatible extraction lives behind `ExtractorBackend`, `agent-browser` login/import and authenticated fallback extraction live behind browser backend capabilities, and filesystem persistence lives behind `SessionStoreBackend`. This preserves the API shape while making later in-process Rust replacements a backend swap rather than a CLI rewrite.

### D47: Static backend contracts need API-level integration tests

The e2e/CLI suite covers current command-backed behavior well, but the new generic `AgetWith` contract also needs direct API tests. `tests/aget_api.rs` now verifies that a custom session store is used by `Aget::get`, a custom extractor receives composed session state and output options, a custom browser fallback handles authenticated extraction after primary extractor failure, and a custom browser automation backend can finish login into a custom store.

This found an important architectural gap: `Aget::get` was still reopening the filesystem `SessionStore` through `GetOptions.home`, so replacing the session-store backend did not affect session-backed fetches. The extraction pipeline now accepts an `ExtractionSessionStore` capability for `AgetWith`, while the lower-level compatibility entry points still construct the filesystem store from `AGET_HOME` or `GetOptions.home`.

### D48: Current PoC backend feature inventory

`aget` currently uses Crawl4AI through the local `scripts/crawl4ai_extract.py` command adapter for a narrow extraction contract:

- Render/fetch one URL from a Playwright-compatible storage state file.
- Return markdown by default, plus `html`, `text`, and `json` content formats.
- Apply CSS selector narrowing through `--selector`.
- Remove matching content through `--exclude-selector`.
- Wait for CSS readiness through `--wait-for`; JavaScript waits are rejected before backend execution.
- Accept a small namespaced escape hatch for Crawl4AI options through `crawl4ai.*` extractor options.
- Write content and metadata artifacts to paths controlled by `aget`.
- Report structured success/failure, warnings, final URL, malformed output, timeouts, and subprocess errors.

`aget` currently uses `agent-browser` for browser/session capabilities:

- Open a URL in a named session and optional profile for login bootstrap.
- Save browser state as Playwright-compatible cookies/localStorage for login finish and Chrome import.
- Load composed Playwright state into a temporary browser profile for authenticated fallback extraction.
- Fetch body HTML or text from the fallback browser session when Crawl4AI cannot use the session state successfully.
- Close sessions after login/import/fallback flows.
- Surface profile-lock/login-needed/browser-action failures as stable `requires_user_action` or backend errors.

Replacement direction: homegrown backends should preserve these behavior contracts before adding broader features. The extraction replacement can start with "HTML fetch/render -> content artifacts -> markdown/text/html/json output shaping." The browser replacement can start with CDP/WebDriver-backed dedicated-profile login, state import/export, and body extraction. Current-tab, screenshots, actions, crawl/map, objective narrowing, and token estimates are later product features and should not be bundled into the first replacement task unless they are needed to preserve existing behavior.
