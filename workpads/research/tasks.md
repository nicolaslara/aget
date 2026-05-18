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

### 📋 Task I10: Security/privacy hardening pass

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
