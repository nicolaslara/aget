# Research Tasks

## Phase 1: Source Project Research

### ✅ Task R0: Discover comparable existing projects

Acceptance criteria:

- Identify projects beyond `curl.md` and Firecrawl that overlap with local/auth-aware web extraction, browser-profile reuse, scraping-to-markdown, agent web context, MCP/browser tools, or local browser automation.
- Prioritize primary sources: repos, docs, architecture notes, licenses, and package manifests.
- Compare each project against `aget` goals: local-first operation, authenticated content support, browser/session strategy, extraction quality, agent integration, maintenance, and license fit.
- Record the project list and source URLs in `references.md`.
- Record a concise comparison and reuse/build recommendation in `knowledge.md`.

### ✅ Task R0a: Benchmark candidate tools on representative pages

Acceptance criteria:

- Compare `curl.md`, Firecrawl, Crawl4AI, and `agent-browser` where locally feasible.
- Use at least one easy static/content page, one JavaScript-rendered page, and one authenticated page flow with explicit user consent.
- Record setup friction, local-only behavior, auth/session handling, JavaScript rendering, markdown quality, output metadata, and failure modes.
- Do not attempt to bypass paywalls, access controls, anti-bot systems, or site policies; authenticated tests must use only user-authorized local sessions.
- Record benchmark commands, outputs or output paths, findings, and open questions in `references.md` and `knowledge.md`.

### 📋 Task R1: Analyze curl.md architecture and reusable components

Acceptance criteria:

- Document repo structure and key packages in `references.md`.
- Identify which parts are hosted-service-specific versus reusable locally.
- Identify markdown extraction/conversion implementation path.
- Identify OpenCode plugin shape and commands/tools.
- Record license implications in `knowledge.md`.

### 📋 Task R2: Analyze Firecrawl self-host architecture and reusable components

Acceptance criteria:

- Document self-host services, dependencies, and runtime requirements.
- Identify Playwright/browser service boundaries.
- Identify markdown conversion, main-content extraction, action/interact implementation paths.
- Record AGPL implications and whether code reuse is acceptable for this project.
- Compare full fork versus selective idea/code reuse.

### 📋 Task R3: Compare curl.md and Firecrawl feature models for aget MVP

Acceptance criteria:

- Produce a feature matrix in `knowledge.md`.
- Mark each feature as MVP, later, or out-of-scope.
- Define first three user journeys for agents.

## Phase 2: Local Browser/Auth Research

### ✅ Task R4a: Compare auth/session ownership models

Acceptance criteria:

- Compare direct use of existing browser data, dedicated `aget` browser/profile login, and explicit copy/import from the user's browser into `aget` storage.
- For each model, document user experience, technical feasibility, platform constraints, privacy risks, credential leakage risks, profile-locking/corruption risks, and auditability.
- Define whether each model uses the user's browser store at fetch time, `aget`'s local store at fetch time, or both.
- Recommend MVP default and advanced/deferred modes.
- Record open questions for the later security/privacy model task.

### ✅ Task R4: Research persistent browser profile strategies

Acceptance criteria:

- Compare Playwright persistent context, CDP attach, and WebDriver approaches.
- Document login reuse behavior, risks, and platform constraints.
- Recommend MVP auth/session strategy.

### 📋 Task R5: Research local header/cookie injection safely

Acceptance criteria:

- Document options for `Authorization`, `Cookie`, custom headers, and cookie jars.
- Identify how to store per-site auth config locally.
- Define redaction and provenance requirements.

### 📋 Task R6: Research current-tab extraction feasibility

Acceptance criteria:

- Determine whether CDP can reliably inspect current tab content.
- Document browser launch requirements and user setup.
- Decide whether current-tab extraction belongs in MVP or later.

## Phase 3: Rust Feasibility

### 📋 Task R7: Evaluate Rust browser automation crates

Acceptance criteria:

- Compare `chromiumoxide`, `fantoccini`, and any actively maintained CDP/WebDriver alternatives.
- Check persistent profile support, screenshots, DOM extraction, wait strategies.
- Recommend pure Rust versus Node/Playwright sidecar.

### 📋 Task R8: Evaluate HTML-to-markdown and readability quality

Acceptance criteria:

- Compare Rust crates and possible JS/WASM bindings.
- Test or document expected behavior on docs pages, articles, and app pages.
- Recommend MVP extraction pipeline.

### 📋 Task R9: Evaluate deterministic objective/keyword narrowing

Acceptance criteria:

- Research non-LLM section scoring methods.
- Define MVP algorithm for objective/keyword filtering.
- Identify when to defer to external LLM summarization, if ever.

## Phase 4: Agent Integration Research

### 📋 Task R10: Research OpenCode plugin integration for a local binary

Acceptance criteria:

- Document plugin packaging requirements.
- Define tools and command names.
- Decide whether the plugin should call CLI, run a local server, or embed logic.

### 📋 Task R11: Research MCP compatibility

Acceptance criteria:

- Determine whether `aget` should expose an MCP server.
- Sketch tool schemas for fetch/current-tab/map/crawl/status.
- Record compatibility implications for Claude, Cursor, OpenCode, and other agents.

## Phase 5: Architecture Proposal

### ✅ Task R12: Produce MVP architecture proposal

Acceptance criteria:

- Write architecture proposal in `knowledge.md`.
- Include CLI commands, config layout, cache layout, output schema, privacy model, and plugin approach.
- Include open risks and deferred features.
- Do not implement yet; leave execution tasks for user review.

## Phase 6: MVP Implementation Plan

These tasks are ready only after the R12 architecture proposal has been reviewed and accepted.

### ✅ Task I0: Create Rust CLI skeleton

Acceptance criteria:

- Add `Cargo.toml` and a Rust binary entry point.
- Implement CLI parsing for `aget <url>` aliasing `aget get <url>`.
- Add `aget get --help` and global flags: `--json`, `--timeout`, `--verbose`, `--quiet`.
- Define stable error categories.
- Add unit tests for CLI parsing and error-category serialization.

### ✅ Task I1: Implement session model and store

Acceptance criteria:

- Add `AGET_HOME` test override and default local storage under `~/.aget`.
- Create `~/.aget`, `sessions/`, `runs/`, `cache/`, and `tmp/` with restrictive `0700` permissions on Unix-like systems.
- Implement session file model with cookies, origins, allowed domains/origins, provenance, and sensitivity metadata.
- Create session JSON files with restrictive `0600` permissions on Unix-like systems.
- Implement `aget session list`, `aget session inspect`, and `aget session delete`.
- Redact secret values by default; support `--show-secrets` only for explicit inspection.
- Add unit and integration tests using isolated `AGET_HOME`, including permissions checks where supported by the platform.

### ✅ Task I2: Compose sessions into Playwright state

Acceptance criteria:

- Convert zero or one session into Playwright-compatible storage state.
- Empty session produces no cookies or origins.
- Deduplicate identical cookies and reject conflicting cookies.
- Create temporary state files with guaranteed cleanup on success and failure.
- Create temporary Playwright state files with restrictive `0600` permissions on Unix-like systems.
- Add tests for empty state, one-session state, dedupe, conflict, and cleanup.

### ✅ Task I3: Add Crawl4AI extraction adapter

Acceptance criteria:

- Add `scripts/crawl4ai_extract.py` based on the proven benchmark helper.
- Shell out to Crawl4AI with timeout handling.
- Implement `aget get <url>` with empty session.
- Write run artifacts under `~/.aget/runs/<run-id>/` unless `--out` is provided.
- Produce `metadata.json` and stable `--json` command output.
- Add local-server integration tests for public fetch and timeout/error behavior.

### ✅ Task I4: Verify local cookie replay path

Acceptance criteria:

- Add local test server that can set and echo cookies.
- Create a hand-written session fixture.
- Verify empty fetch does not send cookies.
- Verify `aget get <url> --session <name>` sends the expected cookie through Crawl4AI.
- Verify authenticated/sensitive metadata is set when a session is used.

### ✅ Task I5: Add optional cmux session import

Acceptance criteria:

- Implement `aget session import cmux --surface <surface> --name <name> --domain <domain>...`.
- Detect missing cmux and return a structured backend-unavailable error.
- Use cmux per-domain cookie export and post-filter returned cookies by allowlist.
- Do not use `cmux browser state save` by default.
- Add optional/skipped e2e test that imports a cookie from a cmux browser pane and replays it.

### ✅ Task I6: Add output shaping and limits

Acceptance criteria:

- Implement or pass through `--format`, `--selector`, `--exclude-selector`, `--wait-for`, `--max-chars`, and `--extractor-option` where feasible.
- Implement deterministic `--max-chars` truncation.
- Record output options and truncation metadata.
- Add tests for output shape and truncation metadata.

### ✅ Task I7: Add Chrome import through agent-browser

Acceptance criteria:

- Implement `aget session import chrome --profile <profile> --name <name> --domain <domain>...`.
- Use a named temporary `agent-browser` session.
- Export raw state to a temp file, filter by explicit allowlist, persist only scoped state, then delete raw state.
- Close only the named temporary `agent-browser` session.
- Return `requires_user_action` if Chrome must be quit or login is needed.
- Add manual verification steps for an authenticated page.

### ✅ Task I8: Add multi-session composition CLI

Acceptance criteria:

- Support repeated `--session` flags for per-request composition.
- Implement `aget session compose <new-name> --session <name>...`.
- Preserve provenance on composed cookies/origins.
- Reject conflicts with redacted conflict reporting.
- Add local app/provider-session tests.

### ✅ Task I8a: Review output and response API flags

Acceptance criteria:

- Clarify the distinction between page content format and agent control-plane response format.
- Evaluate whether `--json` plus `--format markdown` should be replaced or supplemented with clearer names.
- Preserve a human-friendly default that prints markdown directly.
- Preserve an agent-friendly structured mode that exposes status, errors, artifact paths, sensitivity, sessions, and warnings.
- Document the chosen CLI/API design before implementing OpenCode integration.

### ✅ Task I8a-followup: Stabilize response API before OpenCode integration

Acceptance criteria:

- Keep `--json` as a compatibility alias and add `--envelope` as the clearer structured-output flag.
- Remove, implement, or explicitly deprecate non-enforced API promises before agents depend on them. `--max-tokens` and `--only-main` have been removed from the active CLI/API until they can be enforced.
- Rename `artifacts.markdown` to a content-format-neutral key. The active envelope now uses `artifacts.content`.
- Stabilize JSON command output around one agent-friendly envelope shape before `I9` uses it as the OpenCode behavior source of truth.
- Namespace backend-specific `--extractor-option` values before adding another extractor. The active CLI requires `crawl4ai.<key>=<value>`.

### 🚫 Task I8b: Add representative gated-site login/session bootstrap

Acceptance criteria:

- Add an agent-callable login flow that opens a visible `aget`-owned browser session for user-driven login on a representative user-authorized gated site.
- Do not collect, script, or store user credentials.
- Persist only scoped cookies/storage for the selected authorized site after the user completes login.
- Support start, finish, and cancel steps with structured agent-friendly output.
- Verify the workflow can fetch an authorized gated page as markdown with the corresponding caller-chosen session name.
- Add fake-backend/local tests plus an ignored/manual real-site e2e check.

Status note:

- Login start/finish/cancel mechanics are implemented, and the extraction path has been returned to the generic fetcher boundary. The manual authorized real-site flow is still not verified.

### ✅ Task I8b-followup: Remove site-specific extraction coupling

Acceptance criteria:

- Remove hostname/content matching and HelloInterview-specific CTAs from `aget get` extraction behavior.
- Keep `session login start|finish|cancel` generic: session names are caller/user-chosen labels, not built-in site handlers.
- Update `project.md`, `README.md`, and agent-facing task notes so site-specific reasoning belongs to the calling agent or future skill, not the `aget` binary.
- Preserve generic login/session tests while removing tests that encode HelloInterview paywall heuristics in extraction.
- Verify with focused get/session tests plus the full standard check set.

### ✅ Task I8c: Write agent skill for core aget fetch/auth flows

Acceptance criteria:

- Create an agent-facing skill or equivalent guide for using `aget` from an agent.
- Cover the core `aget` fetch/auth flows: fetch an open page, fetch a page behind auth, start/finish login to get auth, combine sessions while logging in, and combine sessions while fetching.
- Use concrete examples that start with HelloInterview, then generalize to authorized gated sites such as `ft.com`, `nytimes.com`, and similar pages the user can access.
- Make clear that agents never collect credentials, bypass access controls, or use ambient browser auth without explicit user action.
- Keep advanced token-saving, crawling, and extraction-tuning guidance for later expansion.

### ✅ Task I8d: Consolidate extractor and session-glue duplication

Acceptance criteria:

- Introduce a small extractor boundary so Crawl4AI and any agent-browser fallback do not duplicate success/failure finalization inside `get_url`.
- Move duplicated agent-browser state/export/filter/process helpers from Chrome import and login flows into a shared module.
- Normalize cookie identity consistently across imported and composed sessions, including leading-dot domains, lowercase domains, and path/name normalization.
- Break `get_url` into smaller units for session loading, extraction invocation, output finalization, and error metadata writing.
- Add regression tests for cross-importer cookie deduplication and any moved shared agent-browser helpers.

### ✅ Task I9: Add OpenCode CLI-backed integration

Acceptance criteria:

- Define OpenCode tool/command wrapper around the local `aget` CLI.
- Expose at least fetch, session list, and session inspect schemas.
- Use `aget --json` as the behavior source of truth.
- Document install/setup and privacy warnings.

### ✅ Task I10: Security/privacy hardening pass

Acceptance criteria:

- Review temp-file cleanup, session redaction, `.gitignore`, authenticated output retention, and provider-session warnings.
- Add tests for secret redaction, restrictive file permissions, and no temp-state residue.
- Document plaintext-session limitations and the encryption-at-rest follow-up.
- Add subprocess timeouts to agent-browser/cmux import/login helpers, and bound child waits after termination.
- Scrub subprocess environments down to explicit allowlists before running backends.
- Tighten redaction for encoded/escaped cookie and storage values, and decide whether sensitive cookie/storage names should also be redacted.
- Sweep orphaned temp raw state/profile files on startup.
- Decide whether structured authenticated output embeds content inline, references artifact paths, or supports an explicit `--content inline|path|none` mode.
- Enforce replay-time session scope checks so selected sessions are not loaded into unrelated request targets without an explicit override.
- Decide whether storage-origin consent needs explicit `--origin` support or an inspect/confirm step separate from cookie-domain consent.
- Remove full agent-browser login profiles after successful login completion, not only pending metadata.
- Run a focused security/privacy review before marking MVP implementation complete.

### ✅ Task I11: Add mocked e2e site server for deterministic integration tests

Acceptance criteria:

- Add a reusable local test server fixture that behaves like a small multi-page site rather than one-off test handlers.
- Cover public pages, login-required pages, cookie-backed auth, localStorage-backed auth, redirects, JavaScript-rendered content, delayed content for wait behavior, noisy nav/main content, and explicit logout/expired-session states.
- Support multi-domain or host-scoped scenarios where feasible, so replay-time session scope checks and provider/app session composition can be tested without real sites.
- Expose deterministic endpoints and helper APIs for tests to create users/sessions, issue cookies/storage state, inspect received cookies/headers, and assert no unrelated credentials were replayed.
- Use the fixture in e2e-style CLI tests for `aget get`, `session login start|finish`, session compose/replay, output shaping, scope rejection, and sensitive artifact handling.
- Keep the fixture local-only, credential-free, and generic; it must not encode real site names, paywall heuristics, or bypass behavior.
- Document when to use this fixture versus smaller unit/fake-backend tests and ignored/manual real-site checks.

### 📋 Task I14: Add real-browser mocked-site smoke tests

Acceptance criteria:

- Reuse the local mocked-site fixture for ignored or environment-gated tests that run through the real browser/Crawl4AI backend instead of the fake backend.
- Cover JavaScript wait behavior and localStorage-driven page behavior against the local fixture without real credentials or external sites.
- Keep the tests opt-in when they require local browser/Playwright/Crawl4AI setup, with clear skip/ignore messaging.
- Use these tests to validate backend integration semantics that fake-backend CLI tests can only simulate.

### ✅ Task I15: Add documentation-style e2e API tests

Acceptance criteria:

- Add mocked-site CLI tests whose names and assertions read like API documentation for common agent workflows.
- Add or use a production `Aget` API facade so docs-style tests can call library behavior directly instead of shelling out through the CLI.
- Cover the public `aget get --json` envelope fields, output formats, selectors/exclusions, `--max-chars`, `--out`, artifacts, warnings, and sensitive/session metadata.
- Cover session lifecycle commands as a user-facing API: login start/finish, list, inspect with redaction, compose, replay, delete, and post-delete failure.
- Cover negative/privacy contracts with explicit assertions on structured error codes and no credential replay.
- Keep tests deterministic, local-only, credential-free, and generic; do not add site-specific login/paywall assumptions.

### ✅ Task I16: Remove inline script-based test doubles

Acceptance criteria:

- Remove inline Python and generated ad hoc test scripts from CLI/integration tests.
- Replace them with checked-in Rust fixture binaries, Rust mock-site routes, or declarative test fixture configs.
- Keep tests able to define custom URLs, responses, redirects, headers, backend failures, backend timeouts, stdout/stderr noise, and agent-browser/cmux responses directly from Rust test code.
- Preserve coverage for subprocess boundaries, environment scrubbing, redaction, timeout/descendant termination, fallback behavior, import flows, and login lifecycle.
- Ensure test fixtures are dev-only and do not appear as product binaries or require Python.

### ✅ Task I18: Introduce pluggable implementation backends behind `Aget`

Acceptance criteria:

- Define stable internal traits or equivalent interfaces for the capabilities `Aget` needs, separate from the current implementation choices:
  - **Extractor backend**: fetch/render/narrow a URL into content plus metadata, artifacts, warnings, and timing.
  - **Browser automation backend**: open login/import/fallback browser sessions, export browser state, extract page content, and close sessions.
  - **Session store backend**: persist/list/load/delete/compose local auth/session state.
- Move current Crawl4AI command execution behind an extractor backend adapter, not an `Aget` field named around commands.
- Move current `agent-browser` command execution behind a browser automation backend adapter.
- Keep the filesystem `SessionStore` as the default session-store backend, but make `Aget` depend on the store capability rather than reaching into storage directly.
- Preserve process-boundary tests for the command-backed adapters while allowing future in-process Rust implementations to be tested without shelling out.
- Keep the public `Aget` API stable enough that CLI and tests call capabilities, not implementation-specific commands.
- Add comments around each backend boundary explaining what is abstracted and why the current adapter is command-backed.

### 🚧 Task I19: Migrate PoC external backends into `aget`-owned implementations

Acceptance criteria:

- Do the migration on branch `dep-migration-homegrown-backends` unless the user redirects.
- Complete the I19a-I19h migration tasks below as separately reviewable phases.
- Preserve the current CLI/API and `Aget` backend interfaces while replacing internals; any public API changes must be separately justified and documented.
- Keep command-backed Crawl4AI and `agent-browser` adapters available until the homegrown replacements pass parity tests for every feature `aget` currently depends on.
- Before porting each entrypoint or feature, inspect the original Crawl4AI or `agent-browser` implementation in the local dependency clone and record the relevant paths/commits in `references.md` or `knowledge.md`.
- Do not copy incompatible code or tests. Record license implications before reusing code or adapting upstream tests.
- Verify the final state with tests that do not require Crawl4AI or `agent-browser` to be installed for the default path.

Status note:

- Task split started on `dep-migration-homegrown-backends`. No porting should begin until I19a-I19c establish source references, backend boundaries, and parity tests.

### ✅ Task I19a: Stage dependency source clones and migration inventory

Acceptance criteria:

- Ensure gitignored local clones exist at `references/repos/crawl4ai` and `references/repos/agent-browser`.
- Record each clone's remote URL, checked-out commit, license, and relevant source/test entrypoints in `references.md`.
- Inventory the current `aget` features that depend on Crawl4AI and `agent-browser`, including source files, scripts, tests, command-line contracts, and environment overrides.
- Record the migration inventory and first-pass risk notes in `knowledge.md`.
- Confirm `references/repos/` remains ignored and no dependency source files are tracked.

### ✅ Task I19b: Audit and tighten backend abstractions for replacement

Acceptance criteria:

- Review the existing extractor, browser automation, and session-store backend interfaces against the I19a inventory.
- Add or adjust internal abstractions only where the current interfaces leak command/process details or cannot support a homegrown backend.
- Keep command-backed adapters as one implementation behind the same interfaces.
- Add API-level tests proving a non-command extractor and non-command browser backend can be swapped in without shelling out.
- Document any abstraction gaps that are deferred rather than silently working around them.

### ✅ Task I19c: Build dependency parity tests before porting

Acceptance criteria:

- Define the behavior matrix for the Crawl4AI features `aget` uses: public fetch, authenticated replay, markdown/html/text/json output, selectors, exclusions, CSS-only waits, truncation metadata, final URL, warnings, artifacts, malformed output, and timeout/error mapping.
- Define the behavior matrix for the `agent-browser` features `aget` uses: login start/finish/cancel, Chrome/profile import, state export parsing, profile lock/no-auth classification, fallback extraction, session close, timeout handling, temp cleanup, and redaction of backend logs.
- Reuse upstream tests only after license review; otherwise adapt small cases or generate expected behavior by running the dependency through existing command adapters.
- Add deterministic parity tests around `MockSite` and checked-in mock tools that can run without network credentials.
- Add optional ignored/manual parity checks for the real dependencies and record their commands.

### 🚧 Task I19d: Port Crawl4AI-backed extraction features into `aget`

Acceptance criteria:

- Inspect the original Crawl4AI implementation for each extraction entrypoint before porting the corresponding `aget` behavior.
- Implement an `aget`-owned extraction backend behind the existing extractor interface.
- Preserve the current behavior used by `aget`: one-URL rendered fetch, explicit session-state input, markdown default, html/text/json content formats, selectors, exclusions, CSS-only waits, truncation/finalization, artifacts, warnings, final URL, and stable failure reporting.
- Preserve the authenticated safety rule that JavaScript waits or extractor options are not executed from user input.
- Pass the I19c Crawl4AI parity tests with the homegrown backend and keep command-adapter tests as compatibility coverage.

Status note:

- First owned extractor slices are implemented behind `ExtractorBackend` and documented in `knowledge.md` D55-D57, D59, D61-D63, D69-D73, D77-D82, D86-D95, D98-D100, D104-D115, D119-D122, D158-D160, and D163-D164. The owned backend now has Rust HTTP(S) transport, CSS selector parsing with Crawl4AI-style no-match and invalid-selector fallback, all-match extraction, selected-wrapper HTML preservation, invalid-exclude-selector tolerance, generic overlay/modal/cookie/dialog selector cleanup, rendered style/z-index/fixed/sticky overlay cleanup before CDP HTML capture, optional shadow DOM flattening for CDP-rendered pages, structural markdown for common static HTML elements including simple tables with captions, nested lists with Crawl4AI/html2text `*` unordered bullets, blockquotes that preserve child block breaks, Markdown hard breaks for `<br>`, escaped accidental list markers and literal backslashes in text, horizontal rules, definition lists, semantic figure/details/address block boundaries, strikethrough, quoted inline text, keyboard/teletype inline code, underscore emphasis markers for `em`/`i`/`u`, abbreviation title definitions, link titles with escaped Markdown constructs, `mailto:` and fragment-only link suppression, title-insensitive automatic absolute links, empty anchor labels, linked-image anchors, escaped link/image markdown targets, base-URL-aware markdown links, cleaned-HTML removal of script/style/link/meta/noscript, base64 image source blanking, empty-leaf element pruning, and Crawl4AI-style important-attribute pruning, Chrome/CDP rendering for localStorage-backed primary extraction, CDP retry when CSS waits require rendered DOM, CDP rendering for detected script-bearing pages, a conservative default main-content heuristic for text/markdown/json output that ranks multiple semantic candidates and labeled `section`/`div` content containers by text/link/label score, and safe support for `crawl4ai.excluded_tags`, `crawl4ai.target_elements`, `crawl4ai.only_text`, applied `crawl4ai.word_count_threshold`, `crawl4ai.delay_before_return_html`, `crawl4ai.page_timeout`, `crawl4ai.wait_for_timeout`, `crawl4ai.wait_until` values `domcontentloaded`/`load`/`networkidle`, `crawl4ai.wait_for_images`, and `crawl4ai.flatten_shadow_dom`, but I19d remains in progress because full Crawl4AI-quality markdown/readability and richer rendered-page readiness heuristics are not yet owned.

### 🚧 Task I19e: Port `agent-browser` session/browser features into `aget`

Acceptance criteria:

- Inspect the original `agent-browser` implementation for each browser/session entrypoint before porting the corresponding `aget` behavior.
- Implement an `aget`-owned browser automation backend behind the existing browser backend interfaces.
- Preserve the current behavior used by `aget`: dedicated login profile/session startup, login finish state export, Chrome/profile import, composed session loading for fallback extraction, body HTML/text fallback output, session close, timeout handling, temp cleanup, and local-only handling of auth state.
- Preserve profile-lock, no-auth-state, login-needed, and `requires_user_action` classification.
- Pass the I19c `agent-browser` parity tests with the homegrown backend and keep command-adapter tests as compatibility coverage.

Status note:

- Started after the first I19d owned extractor slices and documented in `knowledge.md` D58, D60, D64-D68, D83-D85, D96-D97, D101-D103, and D116-D118. `OwnedBrowserAutomationBackend` now has an owned session-backed fallback extraction path for static and scripted cookie-backed pages, a minimal Chrome/CDP renderer for localStorage/sessionStorage-backed fallback extraction, explicit user-data-dir Chrome import, named Chrome profile resolution/copying into temporary user-data-dir imports, a first owned dedicated-profile login start/finish/cancel lifecycle, stale owned-login profile sweeping, PID-backed cleanup for detached owned login browsers, profile-in-use Chrome startup classification as `requires_user_action`, Chrome stderr `DevTools listening on ...` URL startup fallback, sandbox/namespace startup hints, no-stderr Chrome startup hints, `/json/version`, `/json/list`, plus direct `/devtools/browser` CDP discovery fallbacks when attaching to an existing profile browser, three-attempt owned Chrome launch retries, and stale `DevToolsActivePort` removal when existing-profile attach finds a dead endpoint. I19e remains in progress because current-tab attach, broader rendered JavaScript parity, real logged-in profile/keychain smoke coverage, cross-platform close/process lifecycle parity, and fuller startup/error classification still require deeper CDP/profile work.

### ✅ Task I19f: Switch default runtime path to homegrown backends

Acceptance criteria:

- Make the homegrown extractor and browser automation backends the default `aget` runtime path.
- Keep external command adapters behind tests, feature flags, or explicit compatibility configuration until removal is safe.
- Ensure the standard test suite passes with Crawl4AI and `agent-browser` absent from PATH.
- Update `aget doctor`, README, OpenCode tool descriptions, and the agent skill so current installation guidance no longer treats Crawl4AI or `agent-browser` as required default dependencies.
- Record any remaining optional/developer dependency use clearly.

Status note:

- Completed with `knowledge.md` D74. `Aget::new`, `Aget::from_env`, `get_url`, and CLI fetch/session flows now default to owned extractor/browser automation backends unless explicit compatibility environment variables select command adapters. README, OpenCode tool descriptions, and the project aget skill no longer describe Crawl4AI or `agent-browser` as required default dependencies. Full standard-suite validation passed, and a no-command-path smoke passed with compatibility env vars removed and PATH restricted away from optional command tools. There is no implemented `aget doctor` command in this snapshot, so no doctor code surface required an update.

### ✅ Task I19g: Remove or demote PoC dependency surfaces

Acceptance criteria:

- Remove production-only assumptions around `scripts/crawl4ai_extract.py`, Crawl4AI command configuration, and `AGET_AGENT_BROWSER_COMMAND`/`AGET_BACKEND_COMMAND` defaults once replacements are proven.
- Keep only deliberately supported compatibility/test hooks, with names and docs that make their non-default status clear.
- Delete obsolete docs, recipes, or warnings that describe the PoC wrappers as required runtime dependencies.
- Verify `.gitignore`, artifact retention, temp cleanup, and sensitive-output rules still cover the new implementation.

Status note:

- Completed with `knowledge.md` D75. Implicit command defaults were removed from compatibility adapters: Crawl4AI-compatible extraction now requires an API-provided command or `AGET_CRAWL4AI_COMMAND`, and `agent-browser` compatibility paths require `AGET_AGENT_BROWSER_COMMAND`. README, CLI help, and the project skill now describe these as explicit compatibility surfaces. Full `cargo test`, formatting, diff hygiene, live-source default audit, and `.gitignore` review passed.

### 🚧 Task I19h: Final migration review and cleanup

Acceptance criteria:

- Run the full standard check set and all new parity tests.
- Run any ignored/manual real-backend checks that remain useful for comparison, recording results in `references.md` or `knowledge.md`.
- Use review subagents for test adequacy, architecture cohesion, and security/privacy before marking the migration complete.
- Resolve or explicitly record all material review feedback.
- Confirm no dependency source clone contents, sensitive state, authenticated artifacts, or raw browser state are tracked.

Status note:

- Started with `knowledge.md` D76 after I19f/I19g stable commits. Full deterministic suite and local Chrome ignored smokes passed, and tracked-file hygiene audit found no dependency clones, raw browser state, `.aget` artifacts, private benchmark outputs, or logs tracked. I19h remains in progress because the review-subagent acceptance item is still pending; this Codex session may only spawn subagents when explicitly requested by the user.

### ✅ Task I19i: Split oversized extraction and CDP modules

Acceptance criteria:

- Preserve the current CLI/API, backend traits, and default owned runtime behavior while moving code into smaller Rust modules.
- Split `src/extraction.rs` into behavior-owned submodules first, prioritizing markdown rendering, HTML cleanup, owned HTTP/static extraction, command adapters, fallback adapters, and artifacts.
- Split `src/browser_cdp.rs` after the extraction split stabilizes, prioritizing public request/result types, Chrome process lifecycle, CDP client/session plumbing, page rendering, browser state, discovery, and process helpers.
- Keep each split mechanical and separately committable, with no intentional behavior changes unless recorded as a separate follow-up.
- Update workpad knowledge/references with the resulting boundaries and run focused plus full validation before each stable commit.

Status note:

- Completed after D149, with later CLI/binary follow-ups recorded in D161 and D162. The original agent-context bottlenecks are now split into behavior-owned modules: extraction orchestration, owned extraction, markdown, command adapters, fallback adapters, HTML cleanup, HTTP/static fetch, artifacts/redaction, CDP page scripts, process helpers, discovery, Chrome process launch, CDP client/session plumbing, session data conversion, rendered-page orchestration, profile state export, login lifecycle orchestration, and CDP unit tests. The large integration targets were also split into support helpers plus behavior modules for mock-site docs/browser/session/owned coverage, session CLI import/login coverage, and get CLI session-backed replay/fallback coverage. The post-split size profile keeps the formerly 2k-3k line files below the original problem range, the CLI parser is now split into top-level parser, get command, session command, and parser-test modules, and binary session-command execution now lives outside `src/main.rs`.

### ✅ Task I19j: Refactor local replacement engines into `AgetExtractor` and `AgetBrowser`

Acceptance criteria:

- Read `workpads/research/aget-engine-refactor-plan.md` before making code changes.
- Update `workpads/research/aget-engine-refactor-plan.md` as needed when implementation reveals better boundaries, naming conflicts, or policy that belongs at a different layer.
- Introduce self-contained internal engines:
  - `AgetExtractor` for the local Crawl4AI-like extraction behavior that `aget` depends on.
  - `AgetBrowser` for the local `agent-browser`-like browser/CDP behavior that `aget` depends on.
- Keep `Aget` as the public product facade for named sessions, persistence, artifacts, JSON envelopes, authorization workflow, and CLI-facing policy.
- Keep compatibility command adapters explicit and separate; do not add automatic fallback to Crawl4AI or `agent-browser`.
- Stop using `owned` for active local backend names in live code and current docs. Prefer `AgetExtractor`, `AgetExtractorBackend`, `AgetBrowser`, and `AgetBrowserBackend`. Historical workpad notes may keep `owned` when describing past commits.
- Preserve existing public CLI/API behavior and current backend trait contracts unless a follow-up task explicitly approves a behavior change.
- Add or adjust direct engine-level tests so extraction/browser behavior can be tested without full `Aget` orchestration when the behavior is engine-local.
- Verify each mechanical slice with focused tests plus the standard check set, and record final boundaries and validation evidence in `knowledge.md`.

Status note:

- Completed with D170-D176. The browser slices introduced `AgetBrowser`/`AgetBrowserBackend`, routed the default browser automation path through them, and added direct cancellation coverage without the `Aget` facade or command backend. The extractor slices introduced `AgetExtractor`/`AgetExtractorBackend`, routed default extraction and standalone `get_url` helpers through them, removed active `Owned*Backend` shim types, and added direct static plus selector/exclusion/target-elements coverage. Final validation passed with `cargo fmt --check`, `cargo test`, `git diff --check`, and a live-code/docs search confirming no remaining `OwnedExtractorBackend`, `OwnedBrowserAutomationBackend`, or `Default*Backend::Owned` references outside historical workpad notes.

### ✅ Task I19k: Compact support workpad reference files without compacting tasks

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the active planned-task source of truth; do not replace it with a compact summary.
- Move dense support-reference detail out of top-level workpad files into referenced archive files.
- Keep top-level support files focused on current routing, active architecture pointers, and where to open deeper evidence.
- Record the compaction decision in `knowledge.md` and keep historical reference rows available.
- Validate that archived reference files exist, top-level routing is readable, and Markdown/link formatting is coherent.

Status note:

- Completed with D174 after the user clarified that task compaction is not wanted. `workpads/research/references.md` is now a compact routing index, dense historical reference rows live under `workpads/research/archive/references/`, and `workpads/research/tasks.md` remains the planned-task source of truth.

### ✅ Task I19l: Split oversized markdown renderer module

Acceptance criteria:

- Preserve current extraction output behavior while splitting `src/extraction/markdown.rs` into smaller, behavior-owned modules.
- Keep the public extraction module interface unchanged for callers.
- Split mechanically first, prioritizing table rendering and normalization/escaping helpers because they are self-contained.
- Run focused extractor/markdown coverage plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D177 after I19j completed and committed. The mechanical split moved the renderer into `src/extraction/markdown/`, keeping core rendering in `mod.rs` and extracting normalization/escaping helpers plus table rendering into smaller modules. The caller-facing extraction module interface stayed unchanged, and validation passed with focused extractor/markdown tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19m: Split owned extraction orchestration internals

Acceptance criteria:

- Preserve current AgetExtractor behavior while splitting `src/extraction/owned.rs` into smaller, behavior-owned modules.
- Keep the existing `extraction::owned` module interface unchanged for callers.
- Split mechanically first, prioritizing backend option parsing and content selection/rendering helpers because they are cohesive subdomains.
- Run focused owned-extractor tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D178 after I19l completed and committed. The mechanical split moved owned extraction into `src/extraction/owned/`, keeping orchestration in `mod.rs` and extracting backend option parsing plus content selection/rendering helpers into smaller modules. Validation passed with focused owned-extractor tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19n: Split CDP client transport and navigation internals

Acceptance criteria:

- Preserve current AgetBrowser/CDP behavior while splitting `src/browser_cdp/client.rs` into smaller, behavior-owned modules.
- Keep the existing `browser_cdp::client` module interface unchanged for callers.
- Split mechanically first, prioritizing transport/send-read plumbing, navigation/wait behavior, and state load/export helpers.
- Run focused CDP/browser tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D179 after I19m completed and committed. The mechanical split moved CDP client internals into `src/browser_cdp/client/`, keeping target/session setup in `mod.rs` and extracting transport/send-read plumbing, navigation/wait behavior, and state load/export helpers into smaller modules. Validation passed with focused CDP tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19o: Split extraction orchestration helper internals

Acceptance criteria:

- Preserve current `aget get` behavior while splitting helper subdomains out of `src/extraction/mod.rs`.
- Keep the existing `extraction` module interface unchanged for callers.
- Split mechanically first, prioritizing replay-scope enforcement and output/limit metadata helpers because they are cohesive and low-risk.
- Run focused extraction/get tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D180 after I19n completed and committed. The mechanical split kept `src/extraction/mod.rs` as the `aget get` orchestration surface while moving output/limit metadata helpers into `src/extraction/output.rs` and replay-scope enforcement into `src/extraction/replay_scope.rs`. Validation passed with focused extraction/replay-scope tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19p: Split Playwright session composition internals

Acceptance criteria:

- Preserve current session composition and temporary Playwright state file behavior while splitting `src/session/playwright.rs` into smaller modules.
- Keep the existing `session::playwright` public interface unchanged for callers.
- Split mechanically first, prioritizing composition/conflict helpers and temp state file/private-permission helpers because they are cohesive and low-risk.
- Run focused session/playwright tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D181 after I19o completed and committed. The mechanical split moved Playwright session composition into `src/session/playwright/compose.rs`, temp state file/private-permission handling into `state_file.rs`, and existing unit coverage into `tests.rs`, while keeping public Playwright state types and re-exports in `mod.rs`. Validation passed with focused `session::playwright` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19q: Split `Aget` facade helper modules

Acceptance criteria:

- Preserve current public `Aget` API, backend trait contracts, and default runtime behavior while splitting helper subdomains out of `src/aget.rs`.
- Keep `aget::aget::*` public paths and top-level `lib.rs` re-exports compatible for callers.
- Split mechanically first, prioritizing backend adapters, session-store adapter, authorization predicate types/helpers, and `GetRequest` builder because they are cohesive and low-risk.
- Run focused API/facade tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D182 after I19p completed and committed. The mechanical split kept `src/aget/mod.rs` as the public facade/orchestration surface while moving authorization DTOs/helpers, backend adapters, `GetRequest`, and session-store adapter code into focused submodules. Validation passed with focused `aget_api` tests, `cargo fmt --check`, a rerun of one transiently failed loopback `session_cli` authorization test, full `cargo test`, and `git diff --check`.

### ✅ Task I19r: Split agent-browser compatibility session helpers

Acceptance criteria:

- Preserve current compatibility command, session filtering, domain matching, and raw-state temp-file behavior while splitting `src/session/agent_browser.rs`.
- Keep existing `crate::session::agent_browser::*` internal paths unchanged for callers.
- Split mechanically first, prioritizing command execution/failure classification, state filtering/domain helpers, raw state files, and tests because they are cohesive subdomains.
- Run focused agent-browser/session compatibility tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D183 after I19q completed and committed. The mechanical split kept `crate::session::agent_browser::*` as the compatibility surface while moving subprocess execution/failure classification, state filtering/domain helpers, raw-state temp files, and existing tests into focused submodules. Validation passed with focused `session::agent_browser` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19s: Split Chrome session import internals

Acceptance criteria:

- Preserve current compatibility Chrome import and owned Chrome profile import behavior while splitting `src/session/chrome.rs`.
- Keep existing public `session::chrome` import paths unchanged for callers.
- Split mechanically first, prioritizing compatibility command import, owned CDP import, profile discovery/copy helpers, and tests because they are cohesive subdomains.
- Run focused Chrome/session tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D184 after I19r completed and committed. The mechanical split kept `session::chrome` import paths stable while moving compatibility command import, owned CDP import, profile discovery/copy helpers, and existing tests into focused submodules. Validation passed with focused `session::chrome` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19t: Split login session lifecycle internals

Acceptance criteria:

- Preserve current compatibility login, owned login, pending metadata, cleanup, and merge behavior while splitting `src/session/login.rs`.
- Keep existing public `session::login` paths and `src/session/mod.rs` re-exports unchanged for callers.
- Split mechanically first, prioritizing type definitions, pending-login persistence/cleanup helpers, compatibility flow, owned flow, merge logic, and tests because they are cohesive subdomains.
- Run focused login/session tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D185 after I19s completed and committed. The mechanical split kept `session::login` exports stable while moving public types, pending metadata/profile cleanup helpers, compatibility flow, owned flow, merge logic, and existing tests into focused submodules. Validation passed with focused `session::login` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19u: Split binary session command helpers

Acceptance criteria:

- Preserve current `aget session` CLI behavior while splitting helper subdomains out of `src/main_session.rs`.
- Keep `main_session::run_session` as the binary-facing dispatcher used by `src/main.rs`.
- Split mechanically first, prioritizing profile argument normalization, session JSON envelope shaping, and inspect/redaction view helpers because they are cohesive and low-risk.
- Run focused session CLI tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D186 after I19t completed and committed. The mechanical split kept `main_session::run_session` as the `src/main.rs` dispatcher while moving stable command-name mapping, browser/profile argument normalization, authorization envelope data, and inspect/redaction views into focused submodules. Validation passed with focused `session_cli` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19v: Split mock backend support binary internals

Acceptance criteria:

- Preserve current checked-in mock backend behavior while splitting `tests/support/bin/aget_mock_backend.rs`.
- Keep the Cargo mock-tool binary path unchanged for tests and helper commands.
- Split mechanically first, prioritizing argument parsing, behavior dispatch, config/expectation validation, HTTP fetching, and backend result printing.
- Run focused get/mock-site tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D187 after I19u completed and committed. The mechanical split kept the Cargo mock-tool binary path stable while moving argument parsing, configured behavior execution, JSON config/expectation validation, loopback HTTP helpers, and result printing into focused support modules. The first focused run caught module discovery through the relative mock-tool `path`; explicit `#[path = "aget_mock_backend/..."]` module paths fixed it. Validation passed with focused get/mock-site tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19w: Split session login CLI tests by behavior

Acceptance criteria:

- Preserve current session login CLI and direct login lifecycle test behavior while splitting `tests/session_cli/login.rs`.
- Keep the parent `tests/session_cli.rs` test module routing stable.
- Split mechanically first, prioritizing login start, finish, cancel, direct lifecycle API, and ignored real-smoke coverage because they are cohesive subdomains.
- Run focused session CLI tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D188 after I19v completed and committed. The mechanical split kept `tests/session_cli/login.rs` as the parent module route while moving start, finish, cancel, direct lifecycle API, and ignored real-smoke coverage into focused submodules. Validation passed with focused `session_cli` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19x: Split get CLI non-session tests by behavior

Acceptance criteria:

- Preserve current `aget get` CLI test behavior while splitting non-session tests out of `tests/get_cli.rs`.
- Keep the existing `tests/get_cli/session.rs` module route and shared test support route stable.
- Split mechanically first, prioritizing success/artifact behavior, output limits/options, backend validation, and failure/timeout handling.
- Run focused get CLI tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D189 after I19w completed and committed. The mechanical split kept `tests/get_cli.rs` as the parent module route, preserved the existing session-backed get tests, and moved public success/artifacts, output limits/options, backend validation, and failure/timeout behavior into focused submodules. Validation passed with focused `get_cli` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19y: Split get CLI session-backed tests by behavior

Acceptance criteria:

- Preserve current session-backed `aget get` CLI behavior while splitting `tests/get_cli/session.rs`.
- Keep the parent `tests/get_cli.rs` module route stable.
- Split mechanically first, prioritizing replay/scope behavior, fallback/redaction behavior, and ignored real Crawl4AI replay smoke coverage.
- Run focused get CLI tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D190 after I19x completed and committed. The mechanical split kept `tests/get_cli/session.rs` as the parent route while moving session replay/scope, fallback/redaction, and ignored real Crawl4AI replay smoke coverage into focused submodules. Validation passed with focused `get_cli` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19z: Split Aget facade/backend API tests

Acceptance criteria:

- Preserve current `Aget` facade and backend abstraction test behavior while splitting `tests/aget_api.rs`.
- Keep the integration test target name stable.
- Split mechanically first, prioritizing extractor/session-store wiring, browser fallback, authorization, session import/login, and default AgetBrowser backend behavior.
- Run focused API tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D191 after I19y completed and committed. The mechanical split kept `tests/aget_api.rs` as the integration-test route while moving shared fixtures, extraction wiring, authorization, session backend, and default browser backend behavior into focused submodules. Validation passed with focused `aget_api` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19aa: Split session import CLI tests by behavior

Acceptance criteria:

- Preserve current `aget session import` CLI behavior while splitting `tests/session_cli/imports.rs`.
- Keep the parent `tests/session_cli.rs` module route stable.
- Split mechanically first, prioritizing command-backed Chrome import, generic browser import validation, owned Chrome startup classification, and ignored real-browser smoke coverage.
- Run focused session CLI import tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D192 after I19z completed and committed. The mechanical split kept `tests/session_cli.rs` routing through `tests/session_cli/imports.rs` while moving browser import, command-backed Chrome import, owned Chrome startup classification, and ignored real-browser smoke coverage into focused submodules. Validation passed with focused import tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ab: Split extraction public types and backend adapters

Acceptance criteria:

- Preserve current extraction API paths and default runtime behavior while reducing `src/extraction/mod.rs`.
- Move public extraction request/result DTOs and backend traits into a focused submodule.
- Move command/default backend adapter structs into a focused submodule.
- Keep orchestration, replay-scope enforcement, artifact finalization, and error helpers in `src/extraction/mod.rs`.
- Run focused extraction/API tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D193 after I19aa completed and committed. The mechanical split kept public `aget::extraction::*` paths stable while moving extraction DTOs/traits into `src/extraction/types.rs` and backend adapter structs into `src/extraction/backends.rs`; orchestration stayed in `src/extraction/mod.rs`, which dropped to 380 lines. Validation passed with focused extraction/API/mock-site tests, `cargo fmt --check`, `cargo test`, and `git diff --check`. The first full-suite run hit a transient `session_cli::authorize` localhost connection refusal; the affected authorize subset and a second full-suite run passed.

### ✅ Task I19ac: Split markdown renderer writer and inline helpers

Acceptance criteria:

- Preserve current AgetExtractor markdown output behavior while reducing `src/extraction/markdown/mod.rs`.
- Move renderer state/writer helpers into a focused submodule.
- Move inline link/image/abbreviation/text helpers into a focused submodule.
- Keep block/list/table orchestration behavior unchanged.
- Run focused markdown/extractor coverage plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D194 after I19ab completed and committed. The mechanical split kept markdown output behavior unchanged while moving renderer state/blank-line/abbreviation helpers into `src/extraction/markdown/writer.rs` and link/image/abbreviation/raw-text helpers into `src/extraction/markdown/inline.rs`; `src/extraction/markdown/mod.rs` now focuses on block/list/render dispatch and is 283 lines. Validation passed with focused markdown/extractor coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ad: Split browser CDP test modules by behavior

Acceptance criteria:

- Preserve current browser/CDP behavior coverage while reducing `src/browser_cdp/tests.rs`.
- Keep existing discovery test submodule routing stable.
- Split mechanically first, prioritizing CDP state conversion, Chrome/profile lifecycle tests, and page script expression tests.
- Run focused browser CDP tests plus the standard validation set before committing.
- Record the resulting module boundaries in `knowledge.md`.

Status note:

- Completed with D195 after I19ac completed and committed. The mechanical split kept discovery routing stable and moved CDP state conversion, Chrome/profile lifecycle smokes, and page-script expression tests into focused submodules; the parent `src/browser_cdp/tests.rs` now only routes modules and owns shared test helpers. Validation passed with focused browser CDP tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ae: Improve owned main-content scoring with Crawl4AI pruning signals

Acceptance criteria:

- Inspect Crawl4AI source for readability/content pruning signals before changing scoring.
- Preserve current `AgetExtractor` public behavior while improving the default main-content candidate choice.
- Add deterministic mocked coverage for a noisy link-dense candidate competing with a denser article body.
- Keep the heuristic local and transparent; do not introduce LLM or site-specific rules.
- Run focused extractor coverage plus the standard validation set before committing.
- Record source paths, heuristic boundaries, and validation in `knowledge.md`.

Status note:

- Completed with D196 after I19ad completed and committed. The scorer change was based on Crawl4AI `PruningContentFilter` signals from `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`: text density, link density, tag weights, and text length. The owned extractor now keeps its existing transparent label/tag scoring while adding a deterministic density score so link-heavy candidates lose to denser article bodies. Validation passed with focused extractor coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19af: Compact top-level support workpad files without compacting tasks

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the detailed executable backlog; do not shorten completed or planned task entries.
- Move dense historical support material out of top-level workpad files into referenced archive files.
- Keep top-level support files focused on current routing, active decisions, and where to open deeper evidence.
- Preserve existing paths with small routing stubs when old paths are useful entrypoints for agents.
- Validate that archived files exist, Markdown references resolve locally, and `git diff --check` passes.

Status note:

- Completed with D197 after the user clarified that `tasks.md` should not be compacted. The full historical session-wrapper PoC spec and implementation plan moved under `workpads/research/archive/support/`, their old top-level paths now route to the archived versions, and dense `knowledge.md` decision routing moved into `workpads/research/archive/knowledge/current-decision-index.md`. `workpads/research/tasks.md` remains the detailed executable backlog.

### ✅ Task I19ag: Penalize noisy class/id main-content candidates

Acceptance criteria:

- Inspect Crawl4AI source for class/id noise handling before changing the scorer.
- Preserve current `AgetExtractor` public behavior while improving default main-content candidate choice.
- Add deterministic mocked coverage for a comments/promotional candidate that uses otherwise content-like labels.
- Keep the heuristic generic and source-backed; do not introduce site-specific rules.
- Run focused extractor coverage plus the standard validation set before committing.
- Record source paths, heuristic boundaries, and validation in `knowledge.md`.

Status note:

- Completed with D198 after inspecting Crawl4AI `PruningContentFilter._compute_class_id_weight` and `RelevantContentFilter.negative_patterns` in `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`. `AgetExtractor` main-content scoring now applies a generic class/id noise penalty for Crawl4AI-style navigation, advertising, comments, promo, social, and sharing labels, and mocked coverage verifies that a dense comments block does not beat a primary article. Validation passed with focused extractor coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ah: Support direct page CDP sessions for existing-page attach

Acceptance criteria:

- Inspect `agent-browser` source for direct-page CDP connection behavior before changing the CDP client.
- Preserve current browser-level CDP behavior while adding direct page WebSocket support for discovered page targets.
- Ensure direct page sessions enable page/runtime/network domains without flattened `sessionId` parameters.
- Keep browser-level target creation/attach/close behavior unchanged.
- Add deterministic CDP mock coverage for direct page domain enabling.
- Run focused browser CDP coverage plus the standard validation set before committing.
- Record source paths, behavior boundaries, and validation in `knowledge.md`.

Status note:

- Completed with D199 after inspecting `agent-browser` direct-page CDP handling in `references/repos/agent-browser/cli/src/native/browser.rs`. The owned CDP client now detects direct page/webview WebSocket connections, treats them as already attached page sessions, enables Page/Runtime/Network without flattened `sessionId`, and keeps browser-level target creation/attach/close unchanged. This is a lower-level prerequisite for current-tab/existing-page extraction and does not add a public CLI surface. Validation passed with focused browser CDP coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ai: Enable CDP target discovery and auto-attach for existing pages

Acceptance criteria:

- Inspect `agent-browser` source for target discovery and auto-attach behavior before porting.
- Preserve direct page CDP WebSocket behavior without target/session IDs.
- Enable target discovery before reading browser-level target lists.
- Best-effort enable flattened target auto-attach after page domains are enabled for non-direct sessions.
- Add deterministic mock CDP coverage for discovery-before-attach and non-direct auto-attach.
- Verify with focused browser CDP tests plus the standard check set.

Status note:

- Completed with D200 after inspecting `agent-browser` target discovery and domain enablement behavior in `references/repos/agent-browser/cli/src/native/browser.rs`. Browser-level existing-page attach now enables CDP target discovery before reading targets, non-direct page-domain setup best-effort enables flattened target auto-attach, and direct page/webview CDP sessions keep their no-session behavior. Validation passed with focused browser CDP coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19aj: Handle CDP same-document navigation results

Acceptance criteria:

- Inspect `agent-browser` source for `Page.navigate` result handling before porting.
- Preserve current browser/CDP public behavior while improving navigation wait semantics.
- Treat `Page.navigate` responses without `loaderId` as same-document navigation and do not wait for load/domcontentloaded/network-idle events that will not fire.
- Surface `Page.navigate` `errorText` as a stable extraction failure.
- Add deterministic mock CDP coverage for same-document navigation and `errorText`.
- Verify with focused browser CDP tests plus the standard check set.

Status note:

- Completed with D201 after inspecting `agent-browser` `BrowserManager::navigate` behavior in `references/repos/agent-browser/cli/src/native/browser.rs`. The owned CDP navigation path now treats `Page.navigate` responses without `loaderId` as same-document navigations that do not wait for lifecycle/network-idle events, and it reports result-level `errorText` as an extraction failure. Validation passed with focused browser CDP coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ak: Start CDP network-idle waits from load events

Acceptance criteria:

- Inspect `agent-browser` source for network-idle polling behavior before porting.
- Preserve current browser/CDP public behavior while improving rendered-page readiness.
- Let `Page.loadEventFired` start the network-idle quiet window when no network requests are in flight.
- Preserve existing `Page.domContentEventFired` and request tracking behavior.
- Add deterministic mock CDP coverage for a load-only network-idle completion path.
- Verify with focused browser CDP tests plus the standard check set.

Status note:

- Completed with D202 after inspecting `agent-browser` `poll_network_idle` behavior in `references/repos/agent-browser/cli/src/native/browser.rs`. The owned CDP `networkidle` wait now starts its quiet window from either `Page.domContentEventFired` or `Page.loadEventFired` when no requests are in flight, while preserving request tracking. Validation passed with focused browser CDP coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19al: Remove HTML comments during owned cleanup

Acceptance criteria:

- Inspect Crawl4AI source for comment cleanup behavior before changing owned extraction.
- Preserve current `AgetExtractor` public behavior while removing HTML comments from cleaned output.
- Ensure `--content-format html` does not retain comment-only private/debug text.
- Preserve markdown/text behavior.
- Add deterministic mocked coverage for comment removal.
- Verify with focused extractor coverage plus the standard check set.

Status note:

- Completed with D203 after inspecting Crawl4AI comment cleanup in `references/repos/crawl4ai/crawl4ai/utils.py` and `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`. The owned cleanup path now removes HTML comment nodes before selector and output shaping, so `--content-format html` does not retain comment-only debug/private text while markdown/text behavior remains unchanged. Validation passed with focused owned-extractor coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19am: Split AgetExtractor parity test helpers by behavior

Acceptance criteria:

- Preserve `tests/mock_site_cli.rs` public test coverage and test names.
- Split the large `tests/mock_site_cli/aget_extractor.rs` parity body into smaller behavior-focused helper modules.
- Keep fixture routes and assertions behavior-identical; this is a mechanical test-local decomposition only.
- Do not compact `workpads/research/tasks.md`.
- Verify with the focused AgetExtractor parity test plus the standard check set.

Status note:

- Completed with D204. The large AgetExtractor mock-site parity body now routes through smaller behavior-focused helper modules for basic formats/auth, markdown rendering, cleanup, main-content scoring, selector/target options, and backend options/waits. The public integration test name, fixture routes, and assertions remain behavior-identical. `workpads/research/tasks.md` was not compacted. Validation passed with focused AgetExtractor parity coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19an: Split browser CDP Chrome tests by behavior

Acceptance criteria:

- Preserve the existing browser CDP Chrome test behavior and test names.
- Split `src/browser_cdp/tests/chrome.rs` into smaller behavior-focused helper modules.
- Keep mock WebSocket helpers local to the Chrome test namespace.
- Do not compact `workpads/research/tasks.md`.
- Verify with focused browser CDP Chrome coverage plus the standard check set.

Status note:

- Completed with D205. The browser CDP Chrome tests now route through smaller behavior-focused modules for mock CDP client/navigation behavior, Chrome launch retry behavior, and ignored real-Chrome profile/login smokes. The WebSocket request/reply helpers remain local to the Chrome test namespace, existing test behavior is preserved, and `workpads/research/tasks.md` was not compacted. Validation passed with focused browser CDP Chrome coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ao: Render an already-attached CDP page without launching Chrome

Acceptance criteria:

- Inspect `agent-browser` source for existing-page CDP attach and content extraction behavior before porting.
- Preserve current browser/CDP public behavior while adding an owned current-tab prerequisite.
- Add a lower-level owned renderer that connects to an existing CDP WebSocket, attaches to the preferred page, and reads final URL plus document HTML without creating, navigating, or closing a browser.
- Preserve current wait-selector, wait-for-images, overlay cleanup, settle delay, and optional shadow-DOM flattening behavior where applicable.
- Add deterministic mock CDP coverage for attached-page HTML extraction and no-existing-page failure.
- Verify with focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D206 after inspecting `agent-browser` existing-page attach and content behavior in `references/repos/agent-browser/cli/src/native/browser.rs`. The owned CDP layer now has a tested lower-level attached-page renderer that connects to an existing CDP WebSocket, attaches the preferred page, enables page/runtime/network domains, preserves selector/image/settle/overlay/shadow-DOM capture behavior, reads `location.href` plus document HTML, and does not create, navigate, close pages, or close the browser. This is a current-tab prerequisite and does not add a public CLI surface. Validation passed with focused browser CDP coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ap: Expose attached-page rendering at the AgetBrowser engine seam

Acceptance criteria:

- Preserve the public CLI/API surface; do not add a current-tab command before consent UX is chosen.
- Keep the lower-level attached-page renderer available through `AgetBrowser`, not only private CDP tests.
- Keep endpoint ownership explicit: callers must provide the CDP WebSocket URL.
- Add direct `AgetBrowser` engine coverage that does not use the `Aget` facade or command backend.
- Verify with focused AgetBrowser and browser CDP coverage plus the standard check set.

Status note:

- Completed with D207. `AgetBrowser` now has a crate-internal attached-page rendering seam that delegates to the owned CDP renderer, preserving the no-launch/no-navigation/no-close current-tab prerequisite behavior from D206 while keeping public CLI/API UX deferred. Direct engine coverage exercises the seam with a mock CDP WebSocket and no `Aget` facade or command backend. Validation passed with focused AgetBrowser coverage, focused attached-page CDP coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19aq: Expose CDP endpoint discovery at the AgetBrowser engine seam

Acceptance criteria:

- Inspect `agent-browser` source for CDP endpoint discovery behavior before porting.
- Preserve the public CLI/API surface; do not add a current-tab command before consent UX is chosen.
- Keep endpoint ownership explicit: callers provide a local CDP debugging port and receive the resolved browser WebSocket URL.
- Reuse the owned CDP discovery order: `/json/version`, `/json/list`, then direct `/devtools/browser` WebSocket verification.
- Add direct `AgetBrowser` engine coverage that does not use the `Aget` facade or command backend.
- Verify with focused AgetBrowser and browser CDP discovery coverage plus the standard check set.

Status note:

- Completed with D208 after inspecting `agent-browser` CDP discovery behavior in `references/repos/agent-browser/cli/src/native/browser.rs`, `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`, and `references/repos/agent-browser/cli/src/native/cdp/discovery.rs`. `AgetBrowser` now has a crate-internal CDP endpoint discovery seam that accepts an explicit local debugging port and returns the resolved browser WebSocket URL, reusing the owned `/json/version`, `/json/list`, then direct `/devtools/browser` discovery order. Public CLI/API current-tab UX remains deferred pending consent design. Validation passed with focused AgetBrowser coverage, focused browser CDP discovery coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ar: Compose current-tab render through AgetBrowser explicit CDP port

Acceptance criteria:

- Inspect `agent-browser` source for external CDP connection, current page attachment, and external-browser close behavior before porting.
- Preserve the public CLI/API surface; do not add a current-tab command before consent UX is chosen.
- Keep endpoint ownership explicit: callers provide a local CDP debugging port, not ambient profile or port scanning.
- Compose owned CDP endpoint discovery with owned attached-page rendering behind the `AgetBrowser` engine seam.
- Do not navigate, create, or close pages/browsers in this composed current-tab path.
- Add direct `AgetBrowser` engine coverage that proves discovery and attached-page render happen together without the `Aget` facade or command backend.
- Verify with focused AgetBrowser and browser CDP coverage plus the standard check set.

Status note:

- Completed with D209 after inspecting `agent-browser` external CDP connection and current-page behavior in `references/repos/agent-browser/cli/src/native/browser.rs`. `AgetBrowser` now has a crate-internal current-tab render seam that composes explicit local CDP-port discovery with attached-page rendering, returns the resolved browser WebSocket URL for internal provenance, and preserves the no-navigation/no-page-create/no-browser-close boundary. Public CLI/API current-tab UX remains deferred pending consent design. Validation passed with focused AgetBrowser coverage, focused browser CDP coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I20: Design OAuth-safe browser login and profile import flow

Acceptance criteria:

- Redesign login/import around a real user browser flow rather than headless automation for OAuth-sensitive sites.
- Define the default decision tree:
  - Try explicit/session-scoped browser profile import first.
  - Verify imported scoped auth is actually usable before reporting success.
  - If auth is missing, warn that user login/OAuth is required before opening a browser.
  - Suggest importing existing OAuth sessions from the user's real browser/profile whenever possible.
  - Open the user's chosen browser/profile for login and ask the user to confirm completion.
  - Re-import and verify scoped auth before using the session.
- Decide whether a dedicated `aget` browser profile should be the default, optional, or deferred:
  - Dedicated profiles are desirable for isolation.
  - Manual Hello Interview testing showed a fresh Chrome `--user-data-dir` profile did not persist the expected auth cookies after attempted OAuth/login.
  - Normal Chrome `Default` profile import did work when the user was already logged in.
- Add browser-choice terminology and flags to the proposed public API, e.g. default browser, Chrome profile, Arc/Brave support, and explicit profile path.
- Preserve the safety boundary: never ask the agent to handle user passwords or OAuth prompts; the user completes login in their browser.
- Define deterministic mocked-site tests for the decision tree and a documented manual smoke-test recipe for real OAuth sites. Executable tests belong to I21 because they require the first-class orchestration command/API.
- Record lock-handling behavior and error messages for open profile directories, including "quit this browser/profile before import."

Status note:

- Completed after D151/D152. `workpads/research/oauth-safe-browser-login-design.md` records the import-first decision tree, public vocabulary, browser-choice boundary, lock/error wording, mocked-test plan, and manual OAuth smoke recipe. I20 acceptance is design-scoped; executable mocked tests and the first-class orchestration command/API are tracked by I21.

### ✅ Task I21: Implement OAuth-safe session authorization workflow

Acceptance criteria:

- Add a first-class command or API flow that models the desired agent workflow without adding site-specific login/paywall heuristics to the binary.
- Start with an unauthenticated fetch/result artifact, then guide explicit real-browser session import, verification fetch, and re-import after user login when needed.
- Preserve the generic fetcher boundary: the binary can report extraction/import/verification outcomes, while the calling agent interprets page content unless the user supplies generic verification predicates.
- Support the current Chrome import path first, while leaving room for browser-choice terminology such as default browser, Chrome, Arc, Brave, Firefox, Safari, and explicit profile path.
- Add deterministic mocked-site tests that cover:
  - unauthenticated fetch appears gated to the calling agent,
  - import succeeds but verification still looks unauthenticated,
  - import returns `requires_user_action` for locked/no-auth profiles,
  - re-import plus verification succeeds,
  - sensitive/session-backed envelopes omit inline content by default.
- Document the manual real OAuth smoke-test recipe and expected user prompts.

Status note:

- Completed after D155. D153 added the first API-level authorization workflow: `Aget::authorize_chrome_session` performs an unauthenticated baseline fetch, imports scoped Chrome state, verifies with the saved session, evaluates caller-supplied generic predicates, preserves `requires_user_action` import failures, and keeps executable coverage in `tests/aget_api.rs`. D154 added the first CLI surface, `aget session authorize`, with Chrome profile import, browser-neutral `--browser-profile`, sanitized JSON envelopes that omit baseline/verification inline content, mocked session CLI coverage for verified, verification-failed, and profile-lock states, and documented prompt wording in `workpads/research/oauth-safe-browser-login-design.md`. D155 added explicit re-import-after-user-login mocked CLI coverage and closed the remaining deterministic test-plan gap.

### ✅ Task I22: Design and implement browser-choice session import surfaces

Acceptance criteria:

- Define which browsers can be opened for user login versus which browsers can have state imported safely.
- Add explicit public terminology for browser choice, browser profile names, and profile paths without overloading Chrome-specific flags.
- Start with supported Chromium-family import paths only if they can preserve the same scoped filtering, lock handling, temp cleanup, and local-only guarantees as Chrome import.
- Record unsupported browsers and safe fallback guidance instead of pretending broad import works.
- Add mocked and ignored/manual tests for each supported browser family.

Status note:

- Completed after D157. D156 added browser-choice terminology without broadening auth claims: legacy `session import chrome` remains, new `session import browser --browser chrome` maps to the proven Chrome path, unsupported browser families parse but return `usage_error` before backend access, and `workpads/research/browser-choice-session-import-design.md` records the current support matrix and fallback guidance. I22 intentionally stops at the Chrome-only browser-neutral import surface because broader Chromium, Firefox, and Safari import support lacks source-specific discovery, lock-handling, state-export, and manual smoke evidence.

### ✅ Task I17: Redesign public CLI/API and README around coherent concepts

Acceptance criteria:

- Redesign the public CLI/API around four clear concepts:
  - **Session/auth**: explicit local session creation, import, composition, inspection, deletion, and replay.
  - **Extraction**: which URL is fetched and how page content is narrowed or waited for.
  - **Parsing/content format**: how extracted page content is represented.
  - **Presentation/envelope**: how command results are returned to a human or agent.
- Adopt this target `get` API shape:

```bash
aget get <url> \
  --session <session-name> \
  --envelope json \
  --content-format markdown \
  --inline-content auto \
  --output /tmp/page.md \
  --selector main \
  --exclude-selector nav \
  --wait-for-selector main \
  --max-chars 12000
```

- Use these names for the public API:
  - `--envelope <json|none>` controls the command response presentation. `json` returns the structured agent envelope; `none` means human/default output.
  - `--content-format <markdown|html|text|json>` controls the extracted page content format. Keep `json` as a valid content format, but document that it is page content, not the response envelope.
  - `--inline-content <auto|always|never>` only applies when `--envelope json` is used. It controls whether extracted page content appears inline as `data.content` in the JSON envelope.
  - `--output <path>` writes the extracted page content artifact.
  - `--selector <css>` narrows extracted page content. Keep this name; do not rename it to `--include-selector`.
  - `--exclude-selector <css>` removes matching content before output.
  - `--wait-for-selector <css>` waits for a CSS selector. Do not expose generic JavaScript wait wording in the v1 public API.
  - `--allow-domain <domain>` is the explicit import/session scope allowlist for browser-derived session material.
  - `--chrome-profile <profile>` names a Chrome profile for Chrome import, avoiding the overloaded generic `--profile`.
  - `--backend-option <backend.key=value>` is the advanced backend-specific escape hatch; document it as unstable PoC surface.
- Define `--inline-content` behavior precisely:
  - It has no effect in normal human output mode unless `--envelope json` is also selected.
  - `auto` should inline content for non-sensitive fetches and avoid inlining session-backed/sensitive content by default.
  - `always` explicitly includes extracted content in `data.content`, even for sensitive/session-backed fetches.
  - `never` omits `data.content` and returns metadata plus artifact paths only.
  - README must explain that `--inline-content` is separate from `--content-format`: content format controls what the content is; inline content controls whether it is embedded in the JSON envelope.
- Remove or rename ambiguous current names:
  - Replace public `--json` usage with `--envelope json`.
  - Replace `--format` with `--content-format`.
  - Replace `--out` with `--output`.
  - Replace `--wait-for` with `--wait-for-selector`.
  - Replace import `--domain` with `--allow-domain`.
  - Replace Chrome import `--profile` with `--chrome-profile`.
  - Replace `--extractor-option` with `--backend-option`.
- Do not preserve pre-1.0 compatibility aliases unless they are needed temporarily to complete the change. This is still PoC, so breaking API cleanup is acceptable.
- Add serious generated help text for every public command, argument, and flag. The `aget --help`, `aget get --help`, and session subcommand help output should explain the concepts without requiring README context.
- Update structured output naming:
  - Add a schema/version marker such as `schema_version: "aget.envelope.v1"`.
  - Use `data.content_format` rather than `data.format`.
  - Preserve stable error codes and the existing command-bearing success/error envelope shape.
  - Ensure sensitive/session-backed structured output does not embed private page content by default.
- Update OpenCode tool schemas and calls after the CLI names are settled:
  - Use `--envelope json`.
  - Expose `content_format`, `inline_content`, `output`, `selector`, `exclude_selector`, `wait_for_selector`, `sessions`, and backend options with clear descriptions.
  - Do not expose ambient browser auth or secret inspection.
- Rewrite README as a serious open-source-facing README:
  - Start with the product promise: local, auth-aware URL extraction for agents.
  - State clearly that `aget` is still a PoC.
  - Explain that installation currently requires backend dependencies because the project is proving the workflow before bundling or rewriting those pieces.
  - Document current runtime dependencies and their roles: Rust binary, `uv`/Crawl4AI/Playwright for extraction, `agent-browser` for login/Chrome import/fallback browser flows, and optional `cmux` for cmux import.
  - Explain the later installation direction: either bundle dependencies or replace PoC backends once the API/workflow is validated.
  - Present concepts in this order: sessions/auth, extraction, content format, envelope/presentation, artifacts/inline content, privacy.
  - Show clear recipes for public fetch, JSON envelope output, saved artifacts, authenticated/session-backed fetches, login start/finish, Chrome import, and cmux import.
  - Move internal project/workpad notes out of the main quick-start path.
  - Add a license note and follow-up if a standalone `LICENSE` file is still missing.
- Add or split a follow-up task for install tooling if it is too large for this pass:
  - `aget doctor` should check the Rust binary, `uv`, Crawl4AI import, Playwright/browser setup, `agent-browser`, optional `cmux`, `AGET_HOME`, and private storage permissions.
  - `aget setup` or another setup helper can be considered after the README/API cleanup lands.

### 📋 Task I12: Add agent integrations beyond OpenCode

Acceptance criteria:

- Inventory the practical integration surfaces for Cursor, Claude, Codex, and other likely agent hosts, distinguishing native custom tools, MCP tools, shell/CLI wrappers, project skills, slash commands, and documentation-only guidance.
- Decide which integrations should be first-class in this repo versus deferred to MCP or external packages.
- For each recommended first-class integration, define the tool names, schemas, install/setup steps, and privacy warnings.
- Preserve the Rust CLI and structured envelope as the behavior source of truth unless a host integration has a strong reason to call a server/API directly.
- Ensure any authenticated/session-backed integration keeps explicit session selection and does not read ambient browser auth by default.
- Document unsupported hosts and the recommended fallback path, such as using the CLI directly or waiting for MCP support.

### 📋 Task I13: Review default fetch replacement and signature compatibility

Acceptance criteria:

- Evaluate whether `aget` should replace default fetch/webfetch tools in OpenCode or other agent hosts, remain an explicit `aget_fetch` tool, or support both modes.
- Compare existing fetch/webfetch signatures across OpenCode, Cursor, Claude/MCP conventions, Codex-style agent environments, and curl.md's OpenCode plugin where primary docs are available.
- Decide whether `aget_fetch` should use a host-compatible signature, an aget-specific signature, or a compatibility wrapper that maps default fetch arguments onto `aget get`.
- Record privacy and product risks of transparent replacement, especially for authenticated content, inline `data.content`, local artifact paths, session selection, and consent boundaries.
- If replacement is recommended, define the minimal safe behavior: unauthenticated default, explicit `sessions`, content limits, error shape, and whether sensitive output should default to path-only or bounded inline content.
- Update README/agent guidance with the chosen recommendation before implementing a replacement tool name such as `webfetch`.
