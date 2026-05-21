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

- First owned extractor slices are implemented behind `ExtractorBackend` and documented in `knowledge.md` D55-D57, D59, D61-D63, D69-D73, D77-D82, D86-D95, D98-D100, and D104-D110. The owned backend now has Rust HTTP(S) transport, CSS selector parsing with Crawl4AI-style no-match fallback and all-match extraction, structural markdown for common static HTML elements including simple tables with captions, nested lists with Crawl4AI/html2text `*` unordered bullets, blockquotes that preserve child block breaks, Markdown hard breaks for `<br>`, horizontal rules, definition lists, strikethrough, quoted inline text, keyboard/teletype inline code, underscore emphasis markers for `em`/`i`/`u`, abbreviation title definitions, link titles with escaped Markdown constructs, `mailto:` suppression, automatic absolute links, empty anchor labels, escaped link/image markdown targets, base-URL-aware markdown links, cleaned-HTML removal of script/style/link/meta/noscript, base64 image source blanking, empty-leaf element pruning, and Crawl4AI-style important-attribute pruning, Chrome/CDP rendering for localStorage-backed primary extraction, CDP retry when CSS waits require rendered DOM, CDP rendering for detected script-bearing pages, a conservative default main-content heuristic for text/markdown/json output, and safe support for `crawl4ai.excluded_tags`, `crawl4ai.target_elements`, `crawl4ai.only_text`, `crawl4ai.word_count_threshold`, `crawl4ai.delay_before_return_html`, `crawl4ai.page_timeout`, `crawl4ai.wait_for_timeout`, `crawl4ai.wait_until` values `domcontentloaded`/`load`/`networkidle`, and `crawl4ai.wait_for_images`, but I19d remains in progress because full Crawl4AI-quality markdown/readability and richer rendered-page readiness heuristics are not yet owned.

### 🚧 Task I19e: Port `agent-browser` session/browser features into `aget`

Acceptance criteria:

- Inspect the original `agent-browser` implementation for each browser/session entrypoint before porting the corresponding `aget` behavior.
- Implement an `aget`-owned browser automation backend behind the existing browser backend interfaces.
- Preserve the current behavior used by `aget`: dedicated login profile/session startup, login finish state export, Chrome/profile import, composed session loading for fallback extraction, body HTML/text fallback output, session close, timeout handling, temp cleanup, and local-only handling of auth state.
- Preserve profile-lock, no-auth-state, login-needed, and `requires_user_action` classification.
- Pass the I19c `agent-browser` parity tests with the homegrown backend and keep command-adapter tests as compatibility coverage.

Status note:

- Started after the first I19d owned extractor slices and documented in `knowledge.md` D58, D60, D64-D68, D83-D85, D96-D97, and D101-D103. `OwnedBrowserAutomationBackend` now has an owned session-backed fallback extraction path for static and scripted cookie-backed pages, a minimal Chrome/CDP renderer for localStorage/sessionStorage-backed fallback extraction, explicit user-data-dir Chrome import, named Chrome profile resolution/copying into temporary user-data-dir imports, a first owned dedicated-profile login start/finish/cancel lifecycle, stale owned-login profile sweeping, PID-backed cleanup for detached owned login browsers, profile-in-use Chrome startup classification as `requires_user_action`, Chrome stderr `DevTools listening on ...` URL startup fallback, sandbox/namespace startup hints, and `/json/version`, `/json/list`, plus direct `/devtools/browser` CDP discovery fallbacks when attaching to an existing profile browser. I19e remains in progress because current-tab attach, broader rendered JavaScript parity, real logged-in profile/keychain smoke coverage, cross-platform close/process lifecycle parity, and fuller startup/error classification still require deeper CDP/profile work.

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

### 🚧 Task I20: Design OAuth-safe browser login and profile import flow

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
- Add deterministic mocked-site tests for the decision tree and a documented manual smoke-test recipe for real OAuth sites.
- Record lock-handling behavior and error messages for open profile directories, including "quit this browser/profile before import."

Status note:

- Current support is partial, not automatic. `aget` has the import/fetch/session pieces and agent guidance, but no first-class OAuth-safe orchestration command or browser-choice flow. See `knowledge.md` D51.

### 📋 Task I21: Implement OAuth-safe session authorization workflow

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

### 📋 Task I22: Design and implement browser-choice session import surfaces

Acceptance criteria:

- Define which browsers can be opened for user login versus which browsers can have state imported safely.
- Add explicit public terminology for browser choice, browser profile names, and profile paths without overloading Chrome-specific flags.
- Start with supported Chromium-family import paths only if they can preserve the same scoped filtering, lock handling, temp cleanup, and local-only guarantees as Chrome import.
- Record unsupported browsers and safe fallback guidance instead of pretending broad import works.
- Add mocked and ignored/manual tests for each supported browser family.

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
