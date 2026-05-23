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

- First owned extractor slices are implemented behind `ExtractorBackend` and documented in `knowledge.md` D55-D57, D59, D61-D63, D69-D73, D77-D82, D86-D95, D98-D100, D104-D115, D119-D122, D158-D160, D163-D164, D217-D231, D233-D234, D236-D240, D266-D267, D275, D277-D279, D281-D282, D285-D286, D293-D307, D309-D311, D345, D348, D355-D356, D358-D359, and D361-D363. The owned backend now has Rust HTTP(S) transport, CSS selector parsing with Crawl4AI-style no-match and invalid-selector fallback, all-match extraction, selected-wrapper HTML preservation, invalid-exclude-selector tolerance, generic overlay/modal/cookie/dialog selector cleanup, rendered style/z-index/fixed/sticky overlay cleanup before CDP HTML capture, optional shadow DOM flattening for CDP-rendered pages, source-backed iframe body replacement when `crawl4ai.process_iframes` forces CDP rendering, explicit raw/local HTML input handling, structural markdown for common static HTML elements including simple tables with captions, nested lists with Crawl4AI/html2text `*` unordered bullets and opt-in unordered marker strings, blockquotes that preserve child block breaks, Markdown hard breaks for `<br>`, escaped accidental list markers and literal backslashes in text, horizontal rules, definition lists, semantic figure/details/address block boundaries, strikethrough, quoted inline text with opt-in quote marker strings, keyboard/teletype inline code, raw fenced-code line preservation with opt-in Crawl4AI-style `<code>` markers inside `<pre>`, raw HTML subtree preservation with opt-in `crawl4ai.preserve_tags`, Crawl4AI-style plain anchor labels for linked inline code, Crawl4AI-style linked-heading markdown for anchors wrapping a single heading, underscore emphasis markers for `em`/`i`/`u` with opt-in emphasis/strong marker strings, abbreviation title definitions, link titles with escaped Markdown constructs, `mailto:` suppression with opt-in mailto link rendering, fragment-link preservation with opt-in `crawl4ai.skip_internal_links`, opt-in literal sup/sub wrappers, opt-in anchor link suppression, opt-in image suppression, opt-in image-alt-only rendering, opt-in raw image HTML rendering for all images or sized images, opt-in default image alt fallback, opt-in emphasis marker suppression, opt-in link target protection, opt-in reference-style link definitions including paragraph-scoped definition flushing, opt-in table syntax suppression, opt-in HTML-style table bypass rendering, opt-in automatic absolute-link suppression, opt-in broad normal-text markdown character escaping, opt-in line-start marker escape controls, opt-in literal backslash escape controls, opt-in Google Docs-style inline CSS emphasis and list indentation, title-insensitive automatic absolute links, empty anchor labels, linked-image anchors, escaped link/image markdown targets, base-URL-aware markdown links including `crawl4ai.base_url` for raw/local content, generic page metadata propagation including owned browser fallback successes and Crawl4AI-compatible missing-title fallback to prefixed title metadata, cleaned-HTML removal of script/style/link/meta/noscript, base64 image source blanking, Crawl4AI-thresholded empty-leaf element pruning that stays independent of `crawl4ai.word_count_threshold`, Crawl4AI-style `only_text` inline-tag replacement in cleaned HTML, Crawl4AI compatibility-helper-style block boundaries for owned text/json content payloads, Crawl4AI-style important-attribute pruning with opt-in `data-*` preservation, source-backed coverage that `ol[start]` is stripped by default cleaned-HTML extraction, Chrome/CDP rendering for localStorage-backed primary extraction, CDP retry when CSS waits require rendered DOM, CDP rendering for detected executable script-bearing pages, Crawl4AI-style explicit `css:` wait-prefix normalization for owned selector waits, bounded Crawl4AI-style `scan_full_page` CDP scrolling before rendered HTML capture, a conservative default main-content heuristic for text/markdown/json output that ranks multiple semantic candidates and labeled or sufficiently dense unlabeled `section`/`div` content containers by text/link/label/density score while skipping candidates nested inside generic page-chrome ancestors, candidates with Crawl4AI-negative class/id labels, candidates below explicit `crawl4ai.word_count_threshold`, and generic page-chrome tags when automatic main-content falls back to body/root, agent-facing OpenCode option docs aligned with the owned option surface, and safe support for `crawl4ai.base_url`, `crawl4ai.bypass_tables`, `crawl4ai.close_quote`, `crawl4ai.default_image_alt`, `crawl4ai.emphasis_mark`, `crawl4ai.open_quote`, `crawl4ai.strong_mark`, `crawl4ai.ul_item_mark`, `crawl4ai.excluded_tags`, `crawl4ai.target_elements`, `crawl4ai.preserve_tags`, `crawl4ai.handle_code_in_pre`, `crawl4ai.exclude_all_images`, `crawl4ai.exclude_domains`, `crawl4ai.exclude_external_images`, `crawl4ai.exclude_external_links`, `crawl4ai.exclude_internal_links`, `crawl4ai.skip_internal_links`, `crawl4ai.escape_backslash`, `crawl4ai.escape_dash`, `crawl4ai.escape_dot`, `crawl4ai.escape_plus`, `crawl4ai.escape_snob`, `crawl4ai.google_doc`, `crawl4ai.google_list_indent`, `crawl4ai.hide_strikethrough`, `crawl4ai.ignore_images`, `crawl4ai.ignore_anchors`, `crawl4ai.images_as_html`, `crawl4ai.images_to_alt`, `crawl4ai.images_with_size`, `crawl4ai.ignore_links`, `crawl4ai.inline_links`, `crawl4ai.links_each_paragraph`, `crawl4ai.ignore_mailto_links`, `crawl4ai.ignore_tables`, `crawl4ai.ignore_emphasis`, `crawl4ai.include_sup_sub`, `crawl4ai.protect_links`, `crawl4ai.use_automatic_links`, `crawl4ai.exclude_social_media_links`, `crawl4ai.exclude_social_media_domains`, `crawl4ai.only_text`, `crawl4ai.process_iframes`, `crawl4ai.remove_forms`, `crawl4ai.remove_overlay_elements`, `crawl4ai.keep_data_attributes`, applied `crawl4ai.word_count_threshold`, `crawl4ai.delay_before_return_html`, `crawl4ai.page_timeout`, `crawl4ai.wait_for_timeout`, `crawl4ai.wait_until` values `domcontentloaded`/`load`/`networkidle`, `crawl4ai.wait_for_images`, `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, `crawl4ai.max_scroll_steps`, and `crawl4ai.flatten_shadow_dom`, but I19d remains in progress because full Crawl4AI-quality markdown/readability and still-richer rendered-page readiness heuristics are not yet owned.
- Recent follow-up slices D326-D339, D345, D348, and D358-D359 add `crawl4ai.body_width`, `crawl4ai.mark_code`, `crawl4ai.single_line_break`, `crawl4ai.unicode_snob`, `crawl4ai.wrap_links`, `crawl4ai.wrap_list_items`, `crawl4ai.wrap_tables`, `crawl4ai.pad_tables`, `crawl4ai.hide_strikethrough`, `crawl4ai.inline_links`, `crawl4ai.links_each_paragraph`, `crawl4ai.escape_dot`/`crawl4ai.escape_plus`/`crawl4ai.escape_dash`, `crawl4ai.escape_backslash`, `crawl4ai.google_doc`, `crawl4ai.google_list_indent`, `crawl4ai.preserve_tags`, and `crawl4ai.handle_code_in_pre` compatibility with source-backed caveats where owned defaults intentionally remain stable. `workpads/research/tasks.md` was not compacted.

### 🚧 Task I19e: Port `agent-browser` session/browser features into `aget`

Acceptance criteria:

- Inspect the original `agent-browser` implementation for each browser/session entrypoint before porting the corresponding `aget` behavior.
- Implement an `aget`-owned browser automation backend behind the existing browser backend interfaces.
- Preserve the current behavior used by `aget`: dedicated login profile/session startup, login finish state export, Chrome/profile import, composed session loading for fallback extraction, body HTML/text fallback output, session close, timeout handling, temp cleanup, and local-only handling of auth state.
- Preserve profile-lock, no-auth-state, login-needed, and `requires_user_action` classification.
- Pass the I19c `agent-browser` parity tests with the homegrown backend and keep command-adapter tests as compatibility coverage.

Status note:

- Started after the first I19d owned extractor slices and documented in `knowledge.md` D58, D60, D64-D68, D83-D85, D96-D97, D101-D103, D116-D118, D214, D215, D241, D242, D244, D245, D283-D284, D287, D340-D353, D360, and D365. `OwnedBrowserAutomationBackend` now has an owned session-backed fallback extraction path for static and scripted cookie-backed pages, a minimal Chrome/CDP renderer for localStorage/sessionStorage-backed fallback extraction, explicit user-data-dir Chrome import, named Chrome profile resolution/copying into temporary user-data-dir imports, a first owned dedicated-profile login start/finish/cancel lifecycle, stale owned-login profile sweeping, PID-backed cleanup for detached owned login browsers including Windows pid termination hooks, profile-in-use Chrome startup classification as `requires_user_action`, Chrome stderr `DevTools listening on ...` URL startup fallback, sandbox/namespace startup hints, no-stderr Chrome startup hints, source-backed labeled generic Chrome stderr diagnostics including bounded five-line generic stderr tails, `/json/version`, `/json/list`, plus direct `/devtools/browser` CDP discovery fallbacks when attaching to an existing profile browser, three-attempt owned Chrome launch retries, stale `DevToolsActivePort` removal when existing-profile attach finds a dead endpoint, Windows detached process-group Chrome launch flags, agent-browser-style generic Chrome stability/noise-control launch flags, an agent-browser-style headed Chrome window-size boundary, stronger opt-in real-profile import smoke assertions for persisted scoped auth state, agent-browser-style networkidle reset/timeout coverage for the owned CDP navigation wait, agent-browser-style lifecycle/networkidle timeout messages for owned CDP navigation waits, agent-browser-style Chrome early-exit startup messages with explicit exit codes, agent-browser-style `Runtime.evaluate` exception reporting for owned rendered-page capture, readiness/preprocessing helpers, storage load/export including allow-domain-scoped frame-tree origins, stable blank-response storage navigation error reporting, Crawl4AI-compatible image readiness timeout boundary coverage, agent-browser-style binary and malformed CDP response frame coverage, agent-browser-style unlimited CDP WebSocket message and frame size configuration, agent-browser-style CDP WebSocket keepalive pings during command waits, agent-browser-style automatic `alert`/`beforeunload` dialog acceptance while leaving `confirm`/`prompt` explicit, and refreshed ignored local Chrome smoke assertions that match the owned newline-separated text block-boundary contract. I19e remains in progress because broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and still-fuller startup/error classification require deeper CDP/profile work.

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

### ✅ Task I19as: Expose consent-gated current-tab CLI/API through owned backends

Acceptance criteria:

- Inspect `agent-browser` source for external CDP connection and current-page capture behavior before exposing the public surface.
- Add a public `Aget` current-tab API and CLI command that use owned browser/CDP and owned HTML-to-content conversion.
- Require explicit local endpoint ownership and consent: callers must provide a CDP debugging port and acknowledge private current-tab access.
- Preserve generic behavior: no site-specific login/paywall advice, no ambient port/profile scanning, no navigation, no page creation, and no browser close.
- Shape current-tab output like `aget get`: markdown/html/text/json content formats, selectors/exclusions, waits, max chars, artifacts, warnings, timing, and JSON envelope behavior.
- Mark current-tab results sensitive by default so JSON `--inline-content auto` omits content unless explicitly requested.
- Add API/CLI tests that run against a mock CDP server without `agent-browser`.
- Verify with focused parser/API/CLI tests plus the standard check set.

Status note:

- Completed with D210 after inspecting `agent-browser` external CDP connection and current-page capture behavior in `references/repos/agent-browser/cli/src/native/browser.rs`. `aget current-tab` and `Aget::current_tab` now use owned CDP/browser and owned HTML-to-content conversion, require explicit `--cdp-port` plus `--allow-private-content`, avoid ambient scanning/navigation/page creation/browser close, return the standard `GetSuccess`/JSON envelope shape, and mark results sensitive so `--inline-content auto` omits content. README and the local aget skill now document the consent boundary. Validation passed with focused parser/API/CLI current-tab coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19at: Keep current-tab on owned backend under compatibility env

Acceptance criteria:

- Inspect `agent-browser` source for external-CDP/current-page behavior before changing dispatch.
- Preserve explicit compatibility command adapters for login/import/fallback behavior.
- Ensure `aget current-tab` and `Aget::current_tab` keep using the owned browser/CDP path even when `AGET_AGENT_BROWSER_COMMAND` is set for other compatibility surfaces.
- Add deterministic mock-CDP coverage proving current-tab still works under `AGET_AGENT_BROWSER_COMMAND` without invoking `agent-browser`.
- Verify with focused current-tab tests plus the standard check set.

Status note:

- Completed with D211 after re-checking `agent-browser` external-CDP/current-page behavior in `references/repos/agent-browser/cli/src/native/browser.rs`. `Aget::current_tab` now keeps using the owned browser/CDP engine when `AGET_AGENT_BROWSER_COMMAND` is set for login/import/fallback compatibility adapters, and deterministic CLI coverage proves a nonexistent compatibility command does not affect `aget current-tab`. Validation passed with focused current-tab coverage plus the standard check set.

### ✅ Task I19au: Split CLI parser unit tests by command surface

Acceptance criteria:

- Preserve current CLI parser behavior and test assertions.
- Split `src/cli/tests.rs` into smaller focused modules for shared parser/global behavior, `get`, `current-tab`, and `session` parsing.
- Keep the existing `src/cli.rs` test-module routing stable for `cargo test`.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting test boundaries in `knowledge.md`.
- Verify with focused CLI parser tests plus the standard check set.

Status note:

- Completed with D212. The CLI parser unit tests now route through focused modules for global/top-level behavior, `get`, `current-tab`, and `session` parsing while preserving existing assertions and `src/cli.rs` test-module routing. `workpads/research/tasks.md` was not compacted. Validation passed with focused CLI parser coverage plus the standard check set.

### ✅ Task I19av: Split CLI integration tests by behavior

Acceptance criteria:

- Preserve current `tests/cli.rs` integration-test behavior and assertions.
- Split help/error, top-level `get`/envelope, current-tab, and shared local-server/mock-tool helpers into focused modules under `tests/cli/`.
- Keep `tests/cli.rs` as the stable integration-test entrypoint for `cargo test --test cli`.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting test boundaries in `knowledge.md`.
- Verify with focused CLI integration tests plus the standard check set.

Status note:

- Completed with D213. The CLI integration tests now keep `tests/cli.rs` as the stable entrypoint and route help/error, top-level `get`/envelope, current-tab, and shared local-server/mock-tool helpers through focused modules under `tests/cli/`. `workpads/research/tasks.md` was not compacted. The first focused run caught integration-test module path resolution, fixed with explicit `#[path = "cli/..."]` module routes. Validation passed with focused CLI integration coverage plus the standard check set.

### ✅ Task I19aw: Port source-backed generic Chrome startup stderr diagnostics

Acceptance criteria:

- Inspect `agent-browser` Chrome launch-error handling before changing owned diagnostics.
- Preserve current owned Chrome startup classifications, including sandbox hints, silent-exit hints, and `requires_user_action` for profile-in-use cases.
- Improve generic Chrome startup stderr reporting so non-matching stderr lines are explicitly labeled as recent Chrome stderr rather than appended without context.
- Add deterministic unit coverage for generic stderr tail diagnostics and classified startup errors.
- Record the source-backed diagnostic boundary in `knowledge.md`.
- Verify with focused browser CDP discovery tests plus the standard check set.

Status note:

- Completed with D214 after inspecting `agent-browser` Chrome launch-error handling in `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`. Owned Chrome startup diagnostics now label generic non-matching stderr as recent Chrome stderr while preserving sandbox hints, silent-exit hints, and profile/user-action classification. Deterministic discovery tests cover generic stderr tails and classified startup errors. Validation passed with focused browser CDP discovery coverage plus the standard check set.

### ✅ Task I19ax: Strengthen real Chrome profile import smoke coverage

Acceptance criteria:

- Inspect `agent-browser` named Chrome profile copy/keychain behavior before changing smoke coverage.
- Preserve the opt-in/manual nature of the real Chrome profile smoke.
- Require explicit `AGET_REAL_BROWSER_PROFILE` and `AGET_REAL_BROWSER_DOMAIN`; do not default the domain to a public placeholder.
- After successful `session import browser --browser chrome`, assert the saved session exists and contains scoped cookies or storage for the requested domain.
- Keep raw/private cookie or storage values out of test output.
- Record the smoke boundary in `knowledge.md`.
- Verify with the focused ignored smoke build/run path plus the standard check set.

Status note:

- Completed with D215 after inspecting `agent-browser` named profile copy behavior in `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`. The ignored real Chrome import smoke now requires both approved profile and scoped domain env vars, parses the JSON envelope, loads the persisted session through `SessionStore`, verifies Chrome profile provenance, checks the allowlist, and asserts nonempty scoped auth state without printing cookie or storage values. Validation passed with the ignored smoke path returning early without env vars plus the standard check set.

### ✅ Task I19ay: Split AgetBrowser current-tab facade internals

Acceptance criteria:

- Preserve the public `AgetBrowser` wrapper and `AgetBrowserBackend` behavior.
- Split current-tab/CDP endpoint request/result types and methods out of `src/aget_browser.rs`.
- Split `AgetBrowser` internal tests out of the root facade file without weakening assertions.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting boundary in `knowledge.md`.
- Verify with focused `AgetBrowser` coverage plus the standard check set.

Status note:

- Completed with D216. `src/aget_browser.rs` now stays focused on the browser engine wrapper and session/fallback delegation, current-tab/CDP endpoint request/result types plus composition methods live in `src/aget_browser/current_tab.rs`, and internal engine tests live in `src/aget_browser/tests.rs`. Assertions were preserved and `workpads/research/tasks.md` was not compacted. Validation passed with focused AgetBrowser coverage plus the standard check set.

### ✅ Task I19az: Normalize Crawl4AI `css:` wait selectors in owned extraction

Acceptance criteria:

- Inspect Crawl4AI wait behavior before changing owned wait handling.
- Preserve the v1 safety boundary: JavaScript waits remain rejected.
- Strip an explicit `css:` wait prefix before owned static selector checks and CDP `document.querySelector` waits.
- Preserve plain CSS selector behavior and current output metadata.
- Add deterministic coverage for prefixed CSS waits without requiring Chrome, plus lower-level CDP expression coverage.
- Record the source-backed wait boundary in `knowledge.md`.
- Verify with focused extraction/browser script coverage plus the standard check set.

Status note:

- Completed with D217 after inspecting Crawl4AI `smart_wait` in `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`. Owned selector waits now strip the explicit `css:` prefix before CDP `document.querySelector` expressions, while existing static selector parsing already strips the same prefix and JavaScript waits remain rejected by validation. Deterministic coverage now exercises prefixed CSS waits in the static owned extractor and the lower-level CDP selector expression. Validation passed with focused browser script and extractor parity coverage plus the standard check set.

### ✅ Task I19ba: Port Crawl4AI `scan_full_page` rendered readiness into owned extraction

Acceptance criteria:

- Inspect Crawl4AI `scan_full_page`, `scroll_delay`, and `max_scroll_steps` behavior before changing owned rendering.
- Add owned backend options for `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, and `crawl4ai.max_scroll_steps` with source-backed defaults and validation.
- Propagate the option through owned URL extraction and current-tab rendering without changing the static HTTP path.
- Implement bounded CDP full-page scanning before rendered HTML capture, preserving generic local-only behavior and continuing extraction with a warning if scanning times out or fails.
- Add deterministic coverage for option parsing, CDP page-script generation, and the mock CDP render path.
- Update agent-facing option documentation and record the source-backed boundary in `knowledge.md`.
- Verify with focused extraction/browser CDP coverage plus the standard check set.

Status note:

- Completed with D218. Source inspection found Crawl4AI’s scan runs before pre-wait JS/interactions, scrolls by viewport height, defaults to `scroll_delay=0.2`, treats `max_scroll_steps=None` as a runtime cap of 10 in `_handle_full_page_scan`, and warns while continuing on timeout/failure. Owned extraction now validates `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, and `crawl4ai.max_scroll_steps`, propagates them through rendered URL and current-tab CDP capture, scrolls by viewport height before selector/image waits and HTML capture, and continues with a warning when scanning fails. Deterministic coverage exercises option validation, the generated CDP scan expression, and mock-CDP attached-page capture order.

### ✅ Task I19bb: Align Crawl4AI command compatibility options with owned scan readiness

Acceptance criteria:

- Keep the explicit Crawl4AI command helper option allowlist aligned with owned `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, and `crawl4ai.max_scroll_steps`.
- Preserve the compatibility helper's safety boundary: JavaScript waits remain rejected and unsupported options still fail explicitly.
- Add deterministic command-adapter coverage proving scan options pass through validation and are forwarded as backend options.
- Record the compatibility boundary in `knowledge.md`.
- Verify with focused command-adapter coverage plus the standard check set.

Status note:

- Completed with D219. `scripts/crawl4ai_extract.py` now accepts `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, and `crawl4ai.max_scroll_steps` as explicit compatibility options while preserving JavaScript-wait rejection and explicit unknown-option failures. Deterministic command-adapter coverage proves the scan options pass validation and are forwarded as backend options.

### ✅ Task I19bc: Preserve Crawl4AI fragment links in owned markdown

Acceptance criteria:

- Inspect Crawl4AI's active markdown generator before changing owned link rendering.
- Preserve fragment-only anchor links in owned markdown using the page/base URL, matching `CustomHTML2Text` rather than the base `html2text` default.
- Keep `mailto:` suppression unchanged.
- Add deterministic owned markdown coverage for fragment-only links.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused markdown coverage plus the standard check set.

Status note:

- Completed with D220. Source inspection found Crawl4AI's active `CustomHTML2Text` constructor sets `skip_internal_links = False`, so fragment-only links should be rendered rather than suppressed. Owned markdown now resolves `href="#..."` against the page/base URL while keeping `mailto:` suppression unchanged. Deterministic markdown coverage now proves fragment links survive as page-local absolute links.

### ✅ Task I19bd: Lock Crawl4AI ordered-list start cleanup boundary

Acceptance criteria:

- Inspect Crawl4AI/html2text ordered-list numbering and the active Crawl4AI markdown input path.
- Confirm whether `<ol start="N">` survives the default cleaned-HTML pipeline that `aget` uses.
- Preserve the owned extractor's default output boundary for ordered lists after Crawl4AI-style cleanup.
- Add deterministic owned markdown coverage for cleaned ordered-list numbering.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused markdown coverage plus the standard check set.

Status note:

- Completed with D221. Source inspection found Crawl4AI/html2text can read `ol[start]`, but the active default markdown path uses `cleaned_html` and Crawl4AI's attribute pruning does not keep `start`, so this slice locks the cleanup boundary instead of porting unreachable converter behavior. Deterministic owned markdown coverage now proves ordered lists with stripped or malformed starts still render from `1` after default cleanup.

### ✅ Task I19be: Broaden executable script detection for owned rendering

Acceptance criteria:

- Inspect Crawl4AI's normal HTTP crawl path before changing owned render escalation.
- Treat common executable JavaScript MIME types as script-bearing pages in the owned static-to-CDP heuristic.
- Keep non-executable data script types such as JSON-LD and import maps on the static path.
- Add deterministic unit coverage for executable and non-executable script-type detection.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused extraction coverage plus the standard check set.

Status note:

- Completed with D222. Source inspection confirmed Crawl4AI's normal HTTP path uses browser navigation by default. Owned extraction still keeps a fast static path, but now escalates to CDP rendering for common executable JavaScript MIME types including charset-qualified `text/javascript`, while keeping JSON-LD, JSON, import maps, and speculation rules static. Unit coverage locks both sides of the heuristic.

### ✅ Task I19bf: Support Crawl4AI `remove_forms` cleanup in owned extraction

Acceptance criteria:

- Inspect Crawl4AI's `remove_forms` configuration and content-scraping implementation before changing owned cleanup.
- Add owned backend option support for `crawl4ai.remove_forms` with the source-backed default of `false`.
- Remove `<form>` elements before owned content extraction only when the option is true.
- Keep unsupported option failures explicit and align the Crawl4AI command compatibility allowlist.
- Add deterministic mock-site coverage proving default form content remains and opted-in cleanup removes forms.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor and command-adapter coverage plus the standard check set.

Status note:

- Completed with D223. Source inspection found `CrawlerRunConfig.remove_forms` defaults to `false` and Crawl4AI removes `<form>` nodes before content processing when enabled. Owned extraction now supports `crawl4ai.remove_forms=true`, leaves forms in default output, removes forms during opted-in pre-content cleanup, and keeps the command compatibility allowlist aligned. Deterministic mock-site coverage locks the default and opted-in behavior.

### ✅ Task I19bg: Split browser CDP client test coverage by behavior

Acceptance criteria:

- Preserve current browser/CDP client behavior coverage while reducing `src/browser_cdp/tests/chrome/cdp_client.rs`.
- Split the large mock-CDP client test module into smaller behavior-focused modules.
- Keep shared mock server/request helpers local to the Chrome CDP test namespace.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D224. The 558-line mock CDP client test module now routes through smaller behavior-focused modules for setup/attach behavior, navigation behavior, and attached-page rendering behavior. Shared mock WebSocket request/reply helpers remain in the Chrome CDP test namespace, runtime behavior was unchanged, and `workpads/research/tasks.md` was not compacted.

### ✅ Task I19bh: Split session store internals

Acceptance criteria:

- Preserve current session-store layout, save/load/list/delete, private-permission, and orphan-sweep behavior.
- Split `src/session/store.rs` into smaller behavior-focused modules.
- Keep the existing `session::store` public interface stable for callers.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting module boundary in `knowledge.md`.
- Verify with focused session-store coverage plus the standard check set.

Status note:

- Completed with D225. `src/session/store.rs` is now split into `src/session/store/mod.rs` for the public `SessionStore` API, `permissions.rs` for private file/directory creation, `cleanup.rs` for orphaned temp/profile sweeping, and `tests.rs` for existing unit coverage. Session-store behavior and `session::store` exports stayed stable, and `workpads/research/tasks.md` was not compacted.

### ✅ Task I19bi: Support Crawl4AI `keep_data_attributes` cleanup option

Acceptance criteria:

- Inspect Crawl4AI's `keep_data_attributes` configuration and cleaned-HTML attribute pruning implementation before changing owned cleanup.
- Add owned backend option support for `crawl4ai.keep_data_attributes` with the source-backed default of `false`.
- Preserve `data-*` attributes in owned cleaned HTML only when the option is true.
- Keep selectors running against original attributes before pruning, and keep non-data unimportant attributes pruned.
- Align the Crawl4AI command compatibility allowlist and mock backend validation.
- Add deterministic mock-site coverage proving default data-attribute pruning and opted-in data-attribute preservation.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor and command-adapter coverage plus the standard check set.

Status note:

- Completed with D226. Source inspection found `CrawlerRunConfig.keep_data_attributes` defaults to `false`; when enabled, `remove_unwanted_attributes_fast` preserves attributes whose names start with `data-` while still pruning other unimportant attributes. Owned cleaned HTML now preserves `data-*` attributes only with `crawl4ai.keep_data_attributes=true`, keeps selectors running before pruning, and aligns command-helper plus mock-backend option validation.

### ✅ Task I19bj: Support Crawl4AI `exclude_all_images` cleanup option

Acceptance criteria:

- Inspect Crawl4AI's `exclude_all_images` configuration and cleanup implementation before changing owned cleanup.
- Add owned backend option support for `crawl4ai.exclude_all_images` with the source-backed default of `false`.
- Remove image elements before owned content extraction only when the option is true.
- Keep default image behavior unchanged.
- Align the Crawl4AI command compatibility allowlist and mock backend validation.
- Add deterministic mock-site coverage proving default image preservation and opted-in image removal.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor and command-adapter coverage plus the standard check set.

Status note:

- Completed with D227. Source inspection found `CrawlerRunConfig.exclude_all_images` defaults to `false`; when enabled, content scraping removes all `<img>` elements before content processing. Owned extraction now supports `crawl4ai.exclude_all_images=true`, leaves images in default output, removes images during opted-in pre-content cleanup, and keeps command-helper plus mock-backend option validation aligned.

### ✅ Task I19bk: Support Crawl4AI external link and image exclusion options

Acceptance criteria:

- Inspect Crawl4AI's `exclude_external_links` and `exclude_external_images` configuration and scraping implementation before changing owned cleanup.
- Add owned backend option support for `crawl4ai.exclude_external_links` and `crawl4ai.exclude_external_images` with source-backed defaults of `false`.
- Remove external `<a href>` and `<img src>` elements before owned content extraction only when the matching option is true.
- Keep same-origin relative and absolute URLs in default and opted-in external-exclusion output.
- Align the Crawl4AI command compatibility allowlist and mock backend validation.
- Add deterministic mock-site coverage proving default external link/image preservation and opted-in external link/image removal.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor and command-adapter coverage plus the standard check set.

Status note:

- Completed with D228. Source inspection found `CrawlerRunConfig.exclude_external_links` and `exclude_external_images` default to `false`; when enabled, content scraping removes external link or image elements before content extraction. Owned extraction now supports both options, keeps relative and same-base-domain URLs, removes special-scheme links/images consistently with Crawl4AI's external URL helper, and keeps command-helper plus mock-backend option validation aligned.

### ✅ Task I19bl: Support Crawl4AI domain exclusion cleanup option

Acceptance criteria:

- Inspect Crawl4AI's `exclude_domains` configuration and scraping implementation before changing owned cleanup.
- Add owned backend option support for `crawl4ai.exclude_domains` as a list option with source-backed default empty list.
- Remove `<a href>` and `<img src>` elements whose Crawl4AI-like base domain matches an excluded domain.
- Keep default link/image behavior unchanged and keep unrelated relative/same-domain/external URLs unless their base domain is explicitly excluded.
- Align the Crawl4AI command compatibility allowlist and mock backend validation.
- Add deterministic mock-site coverage proving default domain preservation and opted-in domain removal for both links and images.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor and command-adapter coverage plus the standard check set.

Status note:

- Completed with D229. Source inspection found `CrawlerRunConfig.exclude_domains` defaults to an empty list and content scraping removes matching-domain link and image elements while processing media/links. Owned extraction now supports `crawl4ai.exclude_domains`, normalizes configured domains with the Crawl4AI-like base-domain helper, removes matching anchors/images only when opted in, and keeps command-helper plus mock-backend option validation aligned.

### ✅ Task I19bm: Support Crawl4AI social media link exclusion option

Acceptance criteria:

- Inspect Crawl4AI's `exclude_social_media_links` and default social-domain configuration before changing owned cleanup.
- Add owned backend option support for `crawl4ai.exclude_social_media_links` with source-backed default `false`.
- Remove social-media `<a href>` elements before owned content extraction only when the option is true.
- Keep social-media images untouched unless another image/domain option removes them, matching Crawl4AI's link-specific option boundary.
- Keep default social link behavior unchanged.
- Align the Crawl4AI command compatibility allowlist and mock backend validation.
- Add deterministic mock-site coverage proving default social link preservation and opted-in social link removal.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor and command-adapter coverage plus the standard check set.

Status note:

- Completed with D230. Source inspection found `CrawlerRunConfig.exclude_social_media_links` defaults to `false` and, when enabled, Crawl4AI merges the default social-domain list into link-domain exclusions. Owned extraction now supports `crawl4ai.exclude_social_media_links`, removes matching social-media anchors only when opted in, leaves social images to image/domain cleanup options, and keeps command-helper plus mock-backend option validation aligned.

### ✅ Task I19bn: Support Crawl4AI custom social media domain option

Acceptance criteria:

- Inspect Crawl4AI's `exclude_social_media_domains` configuration and how it combines with `exclude_social_media_links` before changing owned cleanup.
- Add owned backend option support for `crawl4ai.exclude_social_media_domains` as a list option.
- Apply custom social domains only when `crawl4ai.exclude_social_media_links=true`, matching the Crawl4AI link-exclusion gate.
- Keep the built-in social domain list active when custom social domains are supplied.
- Keep social-media images untouched unless another image/domain option removes them.
- Align the Crawl4AI command compatibility allowlist and mock backend validation.
- Add deterministic mock-site coverage proving custom social link preservation by default and opted-in removal when the social-link gate is enabled.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor and command-adapter coverage plus the standard check set.

Status note:

- Completed with D231. Source inspection found `CrawlerRunConfig.exclude_social_media_domains` is a list crawler option and Crawl4AI applies it only through the `exclude_social_media_links` gate while merging it with the built-in social-domain list. Owned extraction now supports `crawl4ai.exclude_social_media_domains`, preserves custom social links by default, removes matching custom and built-in social anchors when `crawl4ai.exclude_social_media_links=true`, leaves images to image/domain cleanup options, and keeps command-helper plus mock-backend option validation aligned.

### ✅ Task I19bo: Split large archived workpad knowledge bundles

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Split the largest archived knowledge bundles into smaller referenced files so agents can load only the relevant decision range.
- Keep the original large archive paths as routing indexes rather than deleting them.
- Update `workpads/research/knowledge.md` and `archive/knowledge/current-decision-index.md` so current routing points at the split files.
- Record the compaction decision in `knowledge.md`.
- Verify the split with file-size/line-count checks and `git diff --check`.

Status note:

- Completed with D232. The largest historical knowledge bundles now keep their original paths as short routing indexes, while dense detail lives in smaller decision-range archive files. `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

### ✅ Task I19bp: Skip chrome-contained main-content candidates

Acceptance criteria:

- Inspect Crawl4AI's pruning filter excluded-tag behavior before changing owned main-content scoring.
- Keep owned default extraction from selecting nested `section`/`div` candidates inside generic page chrome such as headers, navs, footers, asides, forms, iframes, or noscript blocks.
- Preserve explicit selector and `crawl4ai.target_elements` behavior; this only affects default main-content candidate scoring.
- Add deterministic mock-site coverage where a content-labeled page-chrome candidate would otherwise beat the real article.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor coverage plus the standard check set.

Status note:

- Completed with D233. Source inspection found Crawl4AI's pruning content filter removes generic page-chrome tags before pruning content blocks. Owned default main-content scoring now skips candidates nested under those page-chrome ancestors, while explicit selectors and `crawl4ai.target_elements` remain unchanged. Deterministic mock-site coverage proves a content-labeled header block no longer beats the real article.

### ✅ Task I19bq: Support explicit Crawl4AI overlay cleanup option

Acceptance criteria:

- Inspect Crawl4AI `remove_overlay_elements` configuration and browser strategy behavior before changing owned cleanup.
- Preserve `aget`'s current default overlay cleanup behavior because the PoC Crawl4AI helper set `remove_overlay_elements=True`.
- Add owned backend option support for `crawl4ai.remove_overlay_elements` as a boolean override.
- Align the Crawl4AI command compatibility allowlist and mock backend validation.
- Add deterministic mock-site coverage proving default overlay cleanup remains on and explicit `false` preserves overlay content.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor and command-adapter coverage plus the standard check set.

Status note:

- Completed with D234. Source inspection found Crawl4AI exposes `remove_overlay_elements`, and the aget PoC compatibility helper already forced it on by default. Owned extraction now keeps overlay cleanup on by default but supports `crawl4ai.remove_overlay_elements=false` to preserve overlay content when requested. The command helper allowlist, mock backend validation, README, and OpenCode tool text are aligned, with deterministic coverage for default cleanup and explicit opt-out.

### ✅ Task I19br: Split owned HTML cleanup helpers by behavior

Acceptance criteria:

- Preserve current owned extractor HTML cleanup behavior while splitting `src/extraction/html_clean.rs` into smaller, behavior-owned modules.
- Keep public/caller-facing extraction module paths stable.
- Split at least attribute/empty-element cleanup from URL/domain/selector cleanup so future Crawl4AI option ports can load less code.
- Do not compact `workpads/research/tasks.md`.
- Record the mechanical split boundary in `knowledge.md`.
- Verify with focused cleanup/owned-extractor coverage plus the standard check set.

Status note:

- Completed with D235. The former `src/extraction/html_clean.rs` module is now split into a stable `src/extraction/html_clean/mod.rs` facade plus `attributes.rs` for attribute/comment/base64/empty-element cleanup and `urls.rs` for external/domain/social URL cleanup. Caller-facing extraction paths and behavior are unchanged, and `workpads/research/tasks.md` remains intentionally un-compacted.

### ✅ Task I19bs: Apply word-count threshold to owned main-content scoring

Acceptance criteria:

- Inspect Crawl4AI `PruningContentFilter` word-count threshold behavior before changing owned scoring.
- Preserve the default `crawl4ai.word_count_threshold=1` behavior for existing extraction.
- Treat explicit `crawl4ai.word_count_threshold` as a hard lower bound for default main-content candidates.
- Keep explicit selectors and `crawl4ai.target_elements` behavior unchanged.
- Add deterministic mock-site coverage proving a below-threshold article no longer wins default main-content selection.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor coverage plus the standard check set.

Status note:

- Completed with D236. Source inspection found Crawl4AI's pruning content filter returns a guaranteed removal score when a candidate's word count is below `min_word_threshold`. Owned default main-content scoring now applies explicit `crawl4ai.word_count_threshold` as a hard candidate lower bound while leaving explicit selectors and `crawl4ai.target_elements` unchanged. Deterministic mock-site coverage proves a below-threshold article no longer wins default main-content selection.

### ✅ Task I19bt: Apply Crawl4AI `only_text` cleanup to owned cleaned HTML

Acceptance criteria:

- Inspect Crawl4AI `only_text` scraping behavior before changing owned cleanup.
- Preserve current markdown `crawl4ai.only_text=true` behavior.
- Apply `crawl4ai.only_text=true` to cleaned HTML by replacing Crawl4AI's eligible inline formatting tags with plain text where doing so does not invalidate explicit selector roots or `crawl4ai.target_elements`.
- Keep explicit selectors and `crawl4ai.target_elements` stable when they select one of the eligible inline tags directly.
- Add deterministic mock-site coverage proving cleaned HTML no longer includes eligible inline formatting tags under `crawl4ai.only_text=true`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor coverage plus the standard check set.

Status note:

- Completed with D237. Source inspection found Crawl4AI's `content_scraping_strategy.py` replaces `ONLY_TEXT_ELIGIBLE_TAGS` with text nodes when `only_text` is set before base64 cleanup, empty-element pruning, and attribute pruning. Owned extraction now applies that replacement to cleaned HTML descendants while preserving selected roots and `crawl4ai.target_elements`, and deterministic mock-site coverage keeps existing markdown behavior plus the cleaned-HTML change locked.

### ✅ Task I19bu: Preserve Crawl4AI-style block boundaries in owned text output

Acceptance criteria:

- Inspect the Crawl4AI compatibility text-output path before changing owned text rendering.
- Replace owned text output's single-line descendant-text flattening with deterministic block-boundary normalization for text and JSON content payloads.
- Preserve existing markdown and cleaned-HTML behavior.
- Add or update deterministic mock-site coverage for headings, paragraphs, list items, selectors, target elements, and main-content text output.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor coverage plus the standard check set.

Status note:

- Completed with D238. Source inspection found the Crawl4AI compatibility helper's text path inserts block-boundary line breaks for common structural tags and drops blank lines after normalizing whitespace. Owned extraction now uses a dedicated text renderer for text output and owned JSON `"content"` payloads while preserving markdown and cleaned HTML behavior. Deterministic mock-site coverage now locks heading, paragraph, list-item, selector, target-element, and main-content text boundaries.

### ✅ Task I19bv: Keep word-count threshold out of cleaned-HTML empty pruning

Acceptance criteria:

- Inspect Crawl4AI cleaned-HTML empty-element pruning before changing owned cleanup.
- Keep `crawl4ai.word_count_threshold` as a default main-content candidate constraint.
- Do not apply `crawl4ai.word_count_threshold` to owned cleaned-HTML empty-leaf pruning.
- Add deterministic mock-site coverage proving short non-empty leaf text remains in cleaned HTML when `crawl4ai.word_count_threshold` is high.
- Preserve existing code-block whitespace empty-element behavior.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor coverage plus the standard check set.

Status note:

- Completed with D239. Source inspection found Crawl4AI calls `remove_empty_elements_fast(body, 1)` during cleaned-HTML generation, separately from `PruningContentFilter` word-threshold scoring. Owned cleanup now keeps `crawl4ai.word_count_threshold` limited to default main-content candidate selection, while cleaned-HTML empty-leaf pruning uses Crawl4AI's fixed threshold and preserves short non-empty headings/captions plus existing code-block whitespace spans.

### ✅ Task I19bw: Preserve Crawl4AI link-label code handling in owned markdown

Acceptance criteria:

- Inspect Crawl4AI markdown rendering behavior for `<code>` inside links before changing owned markdown.
- Render inline code/kbd/tt tags inside anchor labels as plain link-label text, matching Crawl4AI's `CustomHTML2Text` link boundary.
- Preserve standalone inline code/kbd/tt markdown behavior outside links.
- Add deterministic mock-site coverage for code inside a link label and standalone inline code on the same page.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor coverage plus the standard check set.

Status note:

- Completed with D240. Source inspection found Crawl4AI's `CustomHTML2Text` emits inline-code backticks only when not inside an anchor. Owned markdown now renders `code`/`kbd`/`tt` inside link labels as plain label text while preserving standalone inline-code marking. Deterministic mock-site coverage proves code inside a link label and standalone inline code on the same page.

### ✅ Task I19bx: Port agent-browser Windows Chrome process lifecycle hooks

Acceptance criteria:

- Inspect `agent-browser` Windows daemon/process launch and termination behavior before changing owned Chrome process helpers.
- Apply source-backed Windows process-group/detached launch flags to owned Chrome launches.
- Replace non-Unix no-op process termination paths with Windows pid termination for owned Chrome/login cleanup.
- Keep Unix behavior unchanged.
- Add deterministic coverage for the source-backed Windows flags/termination command construction without requiring a Windows runner.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser/CDP coverage plus the standard check set.

Status note:

- Completed with D241. Source inspection found agent-browser uses Windows detached process-group launch flags and pid-based termination for stale/unreachable processes. Owned Chrome process helpers now apply those Windows launch flags, use `taskkill /PID <pid> /F` for Windows cleanup before `Child::kill`, preserve Unix process-group behavior, and include deterministic helper coverage for the source-backed flag and command construction.

### ✅ Task I19by: Align generic Chrome startup stderr tail diagnostics

Acceptance criteria:

- Inspect `agent-browser` generic Chrome startup stderr handling before changing owned diagnostics.
- Preserve existing owned Chrome startup classifications for profile/user-action, sandbox/namespace hints, and silent startup failures.
- Align bounded generic stderr context with the source-backed last-five-lines behavior.
- Add deterministic classifier coverage for the generic stderr tail length.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser/CDP coverage plus the standard check set.

Status note:

- Completed with D242. Source inspection found agent-browser reports the last five generic Chrome stderr lines when no classified startup error keywords are present. Owned Chrome startup diagnostics now use the same five-line bounded fallback while preserving profile/user-action classification, sandbox/namespace hints, and silent-startup hints. Deterministic discovery coverage locks the generic tail length.

### ✅ Task I19bz: Split large archived session wrapper support spec

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Split the oversized historical `archive/support/session-wrapper-poc-spec.md` into smaller section files so agents can load only the relevant historical slice.
- Keep the original archive path as a routing index rather than deleting it.
- Update current support-file routing so the split is discoverable.
- Record the compaction decision in `knowledge.md`.
- Verify with file-size/line-count checks and `git diff --check`.

Status note:

- Completed with D243. The historical session-wrapper PoC spec archive now keeps its old path as a 17-line routing index and preserves the original text in four section files under `archive/support/session-wrapper-poc-spec/`. Concatenating the split files matches the original tracked archive content, `workpads/research/knowledge.md` routes to the index, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19ca: Align owned Chrome launch stability flags with agent-browser

Acceptance criteria:

- Inspect `agent-browser` Chrome launch argument construction before changing owned Chrome launch behavior.
- Add the source-backed generic Chrome launch flags that reduce background work and startup noise without broadening auth/session scope.
- Preserve existing owned profile, keychain, headless, Linux, Windows process-group, and startup-URL behavior.
- Add deterministic coverage for the source-backed launch arguments without requiring real Chrome.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser/CDP coverage plus the standard check set.

Status note:

- Completed with D244. Source inspection found agent-browser's Chrome launch arguments include additional generic stability/noise-control flags. Owned Chrome launches now include `--disable-hang-monitor`, `--disable-prompt-on-repost`, `--enable-features=NetworkService,NetworkServiceInProcess`, and `--metrics-recording-only` while preserving profile/keychain/headless/platform/startup URL behavior. Deterministic command-construction coverage locks the added arguments.

### ✅ Task I19cb: Align headed Chrome launch window-size behavior

Acceptance criteria:

- Inspect `agent-browser` Chrome launch argument construction before changing owned window-size behavior.
- Keep the default `--window-size=1280,720` for headless owned Chrome rendering.
- Omit the default window-size argument for headed visible login Chrome launches, matching agent-browser's headed launch behavior.
- Preserve existing profile, keychain, Linux, Windows process-group, startup URL, and headless SwiftShader behavior.
- Add deterministic command-construction coverage for headless and headed cases without requiring real Chrome.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser/CDP coverage plus the standard check set.

Status note:

- Completed with D245. Source inspection found agent-browser adds the default `--window-size=1280,720` only for headless Chrome launches without extensions. Owned Chrome launches now keep the default window size for headless extraction/rendering and omit it for headed visible login browsers while preserving startup URL, keychain, profile, platform, and SwiftShader behavior. Deterministic command-construction coverage locks both cases.

### ✅ Task I19cc: Split Chrome process launch tests from production module

Acceptance criteria:

- Preserve current Chrome launch command behavior and deterministic test coverage.
- Move launch-command unit tests out of `src/browser_cdp/chrome_process.rs` into a focused test module.
- Keep private production helper access scoped to the browser CDP module; do not introduce public API just for tests.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the mechanical split boundary in `knowledge.md`.
- Verify with focused Chrome process tests plus the standard check set.

Status note:

- Completed with D246. `src/browser_cdp/chrome_process.rs` now keeps production Chrome launch/process lifecycle behavior, while deterministic launch-argument unit coverage lives in `src/browser_cdp/chrome_process/tests.rs`. The split preserves private helper access through the nested test module and leaves `workpads/research/tasks.md` un-compacted.

### ✅ Task I19cd: Split current-tab facade orchestration

Acceptance criteria:

- Preserve the public `Aget`/`AgetWith` current-tab API and consent behavior.
- Move `CurrentTabOptions` and current-tab extraction orchestration out of `src/aget/mod.rs` into a focused facade submodule.
- Keep existing `aget::CurrentTabOptions` exports and backend trait bounds compatible for callers.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the module boundary in `knowledge.md`.
- Verify with focused current-tab/API coverage plus the standard check set.

Status note:

- Completed with D247. `CurrentTabOptions` and consent-gated current-tab extraction orchestration moved from `src/aget/mod.rs` into `src/aget/current_tab.rs`, while `aget::CurrentTabOptions` and `AgetWith::current_tab` remain caller-compatible. The split keeps facade internals private to the `aget` module tree and leaves `workpads/research/tasks.md` un-compacted.

### ✅ Task I19ce: Split AgetExtractor mock-site route fixture

Acceptance criteria:

- Preserve the public `aget_extractor_parity_site()` test helper and all existing mocked route behavior.
- Move dense route HTML out of `tests/mock_site_cli/aget_extractor_site.rs` into behavior-focused route modules.
- Keep the AgetExtractor parity test entrypoint and assertions unchanged.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the test fixture boundary in `knowledge.md`.
- Verify with focused mock-site AgetExtractor parity coverage plus the standard check set.

Status note:

- Completed with D248. `tests/mock_site_cli/aget_extractor_site.rs` now stays as a 21-line route composer, while dense route bodies live under `tests/mock_site_cli/aget_extractor_site/` by formats/options, main-content, selector, markdown, and cleanup behavior. The `aget_extractor_parity_site()` helper and parity assertions remain unchanged, route/asset strings match the previous fixture, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cf: Split session authorization CLI tests by behavior

Acceptance criteria:

- Preserve current `aget session authorize` CLI test behavior and assertions.
- Move individual authorization scenarios out of `tests/session_cli/authorize.rs` into behavior-focused modules.
- Keep the parent `tests/session_cli.rs` module route stable.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused session authorization coverage plus the standard check set.

Status note:

- Completed with D249. `tests/session_cli/authorize.rs` now routes focused authorization scenarios for verified import, verification failure, `requires_user_action`, unsupported browser rejection, and re-import-after-login behavior under `tests/session_cli/authorize/`. The parent `tests/session_cli.rs` route remains stable, assertions and command arguments are preserved, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cg: Split mock-site browser integration tests by behavior

Acceptance criteria:

- Preserve current mock-site browser fallback and Chrome-rendered extractor test behavior.
- Move browser fallback, storage-backed rendering, rendered extraction, and readiness/overlay/image/network tests out of `tests/mock_site_browser.rs` into behavior-focused modules.
- Keep the `tests/mock_site_browser.rs` integration target and ignored real-Chrome annotations stable.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused mock-site browser coverage plus the standard check set.

Status note:

- Completed with D250. `tests/mock_site_browser.rs` now routes behavior-focused modules for browser fallback replay/rendering, storage-backed rendering, rendered extraction, and readiness/overlay/image/network waits under `tests/mock_site_browser/`. The integration target, test names, assertions, and ignored real-Chrome annotations remain stable, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19ch: Split extraction pipeline finalization helpers

Acceptance criteria:

- Preserve current `get_url`/`get_url_with_*` behavior, artifacts, metadata, fallback, and direct extraction outputs.
- Move primary extractor execution, session fallback, selected-session loading, direct extraction finalization, and success/error finalization helpers out of `src/extraction/mod.rs` into a focused submodule.
- Keep public `crate::extraction::*` API paths and `finish_direct_extraction` internal access compatible for callers.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the module boundary in `knowledge.md`.
- Verify with focused extraction/current-tab coverage plus the standard check set.

Status note:

- Completed with D251. `src/extraction/mod.rs` now keeps public extraction API routing and `get_url_with_session_store` orchestration, while `src/extraction/pipeline.rs` owns selected-session loading, primary extractor execution, session fallback, direct extraction finalization, and success/error output finalization. Public `crate::extraction::*` paths and internal `finish_direct_extraction` access remain stable, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19ci: Split historical module-decomposition archive support file

Acceptance criteria:

- Preserve the full D123-D150 decision text under referenced archive files.
- Keep `workpads/research/archive/knowledge/d123-d150-module-decomposition.md` as the stable routing entrypoint.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Update current knowledge routing and decision index so the split is discoverable.
- Verify the split with reconstruction/content checks and `git diff --check`.

Status note:

- Completed with D252. The historical D123-D150 module-decomposition archive now keeps its old path as a 9-line routing index and preserves the original decision text in three section files under `archive/knowledge/d123-d150-module-decomposition/`. Concatenating the split files with the original title reconstructs the previous archive content exactly, current routing points at the section files, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cj: Split remaining session CLI root tests by behavior

Acceptance criteria:

- Preserve current `aget session` list/delete, compose, and inspect integration-test behavior.
- Move the remaining root tests out of `tests/session_cli.rs` into behavior-focused modules under `tests/session_cli/`.
- Keep the `tests/session_cli.rs` integration target and existing authorize/import/login module routing stable.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused `session_cli` coverage plus the standard check set.

Status note:

- Completed with D253. `tests/session_cli.rs` is now a small integration-target router, while remaining list/delete, compose, and inspect scenarios live in behavior-focused modules under `tests/session_cli/`. Test names, assertions, command arguments, existing authorize/import/login routing, and ignored real-smoke annotations remain stable, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19ck: Split shared mock-site support server helpers

Acceptance criteria:

- Preserve current `MockSite`, `MockSiteBuilder`, `MockResponse`, default routes, request recording, and header/cookie assertion behavior.
- Move request parsing/HTTP response helpers and default route dispatch out of `tests/support/mock_site.rs` into focused support submodules.
- Keep existing `support::mock_site::{MockResponse, MockSite, MockSiteBuilder}` imports compatible.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting support boundary in `knowledge.md`.
- Verify with focused mock-site targets plus the standard check set.

Status note:

- Completed with D254. `tests/support/mock_site.rs` now keeps the public mock-site support API and server lifecycle, while `tests/support/mock_site/protocol.rs` owns raw HTTP parsing/formatting and cookie matching, and `tests/support/mock_site/routes.rs` owns the default public/auth/storage/login/logout route dispatch. Existing support imports, custom route behavior, default route responses, request recording, and header/cookie assertions remain stable, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cl: Split owned extraction page pipeline helpers

Acceptance criteria:

- Preserve current owned extractor and browser fallback behavior, including static-vs-rendered routing, rendered waits, script detection, cleanup, selectors, and output formats.
- Move owned page extraction/routing helpers out of `src/extraction/owned/mod.rs` into a focused submodule.
- Keep `run_owned_extractor_backend`, `run_owned_browser_fallback`, `extract_owned_rendered_html`, `validate_owned_extraction_options`, and `OWNED_EXTRACTOR` internal access compatible for callers.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the module boundary in `knowledge.md`.
- Verify with focused owned extraction coverage plus the standard check set.

Status note:

- Completed with D255. `src/extraction/owned/mod.rs` now keeps backend adapter entrypoints, compatibility labels, artifact writes, and public owned extractor exports, while `src/extraction/owned/page.rs` owns static-versus-rendered routing, rendered waits, direct rendered-HTML extraction, script detection, HTML cleanup sequencing, selector/fallback selection, and output-format shaping. Internal access to `run_owned_extractor_backend`, `run_owned_browser_fallback`, `extract_owned_rendered_html`, `validate_owned_extraction_options`, and `OWNED_EXTRACTOR` remains stable, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cm: Split binary session import and login command handlers

Acceptance criteria:

- Preserve current `aget session import` and `aget session login` CLI behavior, JSON envelopes, plain output, warnings, and error classification.
- Move import and login command handler bodies out of `src/main_session/mod.rs` into focused submodules.
- Keep `main_session::run_session` as the binary-facing dispatcher and keep existing command-name/profile/envelope/inspect helper boundaries stable.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting command-handler boundary in `knowledge.md`.
- Verify with focused session import/login CLI coverage plus the standard check set.

Status note:

- Completed with D256. `src/main_session/mod.rs` remains the `run_session` dispatcher and keeps list/authorize/inspect/delete/compose routing, while `src/main_session/import_command.rs` owns cmux/browser/Chrome import output and dispatch, and `src/main_session/login_command.rs` owns login start/finish/cancel output, envelope shaping, and OAuth warning text. Command names, JSON fields, plain output, unsupported-browser usage errors, and error-response wrapping remain stable, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cn: Split browser CDP startup diagnostics helpers

Acceptance criteria:

- Preserve current Chrome startup stderr parsing, error classification, sandbox/no-stderr hints, and DevTools stderr URL fallback behavior.
- Move startup diagnostics helpers out of `src/browser_cdp/discovery.rs` into a focused submodule.
- Keep current `browser_cdp` internal function access compatible for Chrome process and discovery callers.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting browser CDP diagnostics boundary in `knowledge.md`.
- Verify with focused browser CDP discovery diagnostics coverage plus the standard check set.

Status note:

- Completed with D257. `src/browser_cdp/discovery.rs` now keeps DevToolsActivePort polling, existing-profile attach, `/json/version`, `/json/list`, direct `/devtools/browser` discovery, WebSocket host rewriting, and profile-browser shutdown polling, while `src/browser_cdp/discovery/diagnostics.rs` owns Chrome stderr DevTools URL fallback parsing, startup error classification, requires-user-action matching, relevant stderr filtering, sandbox/no-stderr hints, and generic stderr tails. Existing Chrome process and browser CDP test access stays routed through `discovery`, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19co: Split owned content main-content scoring helpers

Acceptance criteria:

- Preserve current owned content extraction behavior, including selector fallback, target elements, cleanup order, main-content selection, markdown/text/html output, and base URL handling.
- Move main-content candidate selection and scoring helpers out of `src/extraction/owned/content.rs` into a focused submodule.
- Keep `extract_owned_content` and `markdown_base_url` internal access compatible for owned extraction callers.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting owned content boundary in `knowledge.md`.
- Verify with focused AgetExtractor main-content coverage plus the standard check set.

Status note:

- Completed with D258. `src/extraction/owned/content.rs` now keeps owned content extraction composition, selector fallback, target-element collection, cleanup order, output assembly, selected-element lookup, and markdown base URL handling, while `src/extraction/owned/content/main_content.rs` owns default main-content candidate discovery and scoring, including Crawl4AI-like pruning density, label bonuses/penalties, class/id noise penalties, excluded ancestor checks, and word-threshold gating. `extract_owned_content` and `markdown_base_url` remain owned-extraction internal entrypoints, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cp: Split agent-browser fallback command adapter helpers

Acceptance criteria:

- Preserve current command-compat browser fallback behavior, including state load, open/get/close sequencing, timeout use, close-error handling, output formatting, text fallback, and failure classification.
- Move agent-browser command execution, temporary file/profile helpers, and fallback HTML-to-text conversion out of `src/extraction/fallback_command.rs` into focused submodules.
- Keep `CommandBrowserFallbackBackend` and `run_agent_browser_fallback` internal access compatible for extraction callers.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting compatibility fallback boundary in `knowledge.md`.
- Verify with focused session fallback CLI coverage plus the standard check set.

Status note:

- Completed with D259. `src/extraction/fallback_command.rs` now keeps command-compat browser fallback orchestration for state load, open, content extraction, close handling, warnings, and result shaping, while `src/extraction/fallback_command/command.rs` owns `AGET_AGENT_BROWSER_COMMAND` subprocess execution and failure classification, `src/extraction/fallback_command/temp.rs` owns private temporary fallback profile/output files and cleanup, and `src/extraction/fallback_command/text.rs` owns the compatibility HTML-to-text conversion used for fallback markdown/text/json content. `CommandBrowserFallbackBackend` and `run_agent_browser_fallback` remain extraction-internal entrypoints, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cq: Split browser CDP readiness/evaluation helpers

Acceptance criteria:

- Preserve current CDP navigation, network-idle, selector wait, image wait, full-page scan, overlay cleanup, and string evaluation behavior.
- Move runtime readiness/evaluation helpers out of `src/browser_cdp/client/navigation.rs` into a focused submodule.
- Keep current `browser_cdp` internal method access compatible for render/current-tab callers and tests.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting CDP client boundary in `knowledge.md`.
- Verify with focused browser CDP navigation/readiness coverage plus the standard check set.

Status note:

- Completed with D260. `src/browser_cdp/client/navigation.rs` now keeps page navigation, blank-response navigation, lifecycle-event waits, network-idle tracking, navigation response/error handling, session message matching, and network request id extraction, while `src/browser_cdp/client/navigation/readiness.rs` owns runtime readiness/evaluation helpers for selector waits, image completeness polling, full-page scan evaluation, rendered overlay cleanup evaluation, and string evaluation. Existing render/current-tab callers continue to use the same `CdpClient` methods, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cr: Split Chrome profile discovery helpers

Acceptance criteria:

- Preserve current owned Chrome import profile behavior, including explicit path handling, Chrome user-data-dir discovery, Local State parsing, directory/display-name matching, ambiguous-name errors, available-profile errors, and copied-profile import setup.
- Move Chrome profile discovery and name resolution helpers out of `src/session/chrome/profile.rs` into a focused submodule.
- Keep current `session::chrome::profile` internal function access compatible for owned Chrome import callers and tests.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting Chrome profile boundary in `knowledge.md`.
- Verify with focused session Chrome profile coverage plus the standard check set.

Status note:

- Completed with D261. `src/session/chrome/profile.rs` now keeps owned Chrome profile preparation, explicit profile path validation, copied-profile lifetime, profile snapshot copying, copy exclusions, and private-directory helpers, while `src/session/chrome/profile/discovery.rs` owns Chrome user-data-dir discovery, `Local State` profile parsing, directory/display-name/case-insensitive matching, ambiguous-name reporting, and available-profile error formatting. Existing owned Chrome import callers and `session::chrome` tests continue to route through `session::chrome::profile`, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19cs: Split Chrome process launch command helpers

Acceptance criteria:

- Preserve current owned Chrome launch behavior, including temp/profile/login launch entrypoints, launch retries, startup diagnostics, process cleanup, launch arguments, binary discovery, platform candidates, process-group configuration, and temp profile directory creation.
- Move Chrome launch command construction, Chrome binary discovery, platform candidate lookup, PATH lookup, and temp profile directory creation out of `src/browser_cdp/chrome_process.rs` into a focused submodule.
- Keep current `browser_cdp::chrome_process` internal function/test access compatible for render/import/login callers and launch-argument tests.
- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Record the resulting Chrome process launch boundary in `knowledge.md`.
- Verify with focused Chrome process coverage plus the standard check set.

Status note:

- Completed with D262. `src/browser_cdp/chrome_process.rs` now keeps `ChromeProcess` temp/profile/login launch entrypoints, launch retry orchestration, startup diagnostics classification, child process lifecycle, detach/wait-or-kill behavior, and owned temp profile cleanup, while `src/browser_cdp/chrome_process/launch.rs` owns Chrome launch command construction, launch flags, process-group configuration, Chrome binary discovery, platform candidate lookup, PATH lookup, and temp owned-profile directory creation. Existing render/import/login callers and Chrome process launch tests continue to route through `chrome_process`, and `workpads/research/tasks.md` remains un-compacted.

### ✅ Task I19ct: Split dense session-wrapper support archive slice

Acceptance criteria:

- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Preserve the historical V1 thin-wrapper implementation spec content under `workpads/research/archive/support/session-wrapper-poc-spec/`.
- Convert `v1-implementation-spec.md` into a compact routing index and move dense architecture/storage, command-surface, adapter/composition, and output metadata detail into smaller referenced files.
- Keep existing archive entrypoints usable for agents that need historical session-wrapper context.
- Record the support-file boundary in `knowledge.md`.
- Verify with markdown routing checks and a tracked workpad line-count scan.

Status note:

- Completed with D263. `workpads/research/archive/support/session-wrapper-poc-spec/v1-implementation-spec.md` is now a compact routing index, with historical V1 architecture/storage detail in `v1-architecture-storage.md`, command-surface detail in `v1-command-surface.md`, and adapter/composition/output metadata detail in `v1-adapters-composition-output.md`. `workpads/research/tasks.md` was not compacted; it only records this completed support-file split. Routing checks, line-count checks, and `git diff --check` passed.

### ✅ Task I19cu: Split dense session-wrapper implementation-plan archive

Acceptance criteria:

- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Preserve the historical session-wrapper implementation plan content under `workpads/research/archive/support/session-wrapper-implementation-plan/`.
- Convert `session-wrapper-implementation-plan.md` into a compact routing index and move dense objective/stack/repository, milestone, testing/first-slice, and pre-coding decision detail into smaller referenced files.
- Keep existing archive entrypoints usable for agents that need historical session-wrapper planning context.
- Record the support-file boundary in `knowledge.md`.
- Verify with markdown routing checks, line-count checks, and `git diff --check`.

Status note:

- Completed with D264. `workpads/research/archive/support/session-wrapper-implementation-plan.md` is now a compact routing index, with historical objective/stack/repository detail in `objective-stack-repo.md`, milestones in `milestones.md`, testing/first-slice detail in `testing-and-first-slice.md`, and pre-coding decisions in `decisions-before-coding.md`. `workpads/research/tasks.md` was not compacted; it only records this completed support-file split. Routing checks, line-count checks, and `git diff --check` passed.

### ✅ Task I19cv: Split dense session-wrapper product-spec archive

Acceptance criteria:

- Do not compact or remove planned tasks from `workpads/research/tasks.md`.
- Preserve the historical session-wrapper product spec content under `workpads/research/archive/support/session-wrapper-poc-spec/`.
- Convert `product-spec.md` into a compact routing index and move dense overview, core-concept, journey, UX, and security detail into smaller referenced files.
- Keep existing archive entrypoints usable for agents that need historical session-wrapper product context.
- Record the support-file boundary in `knowledge.md`.
- Verify with markdown routing checks, line-count checks, and `git diff --check`.

Status note:

- Completed with D265. `workpads/research/archive/support/session-wrapper-poc-spec/product-spec.md` is now a compact routing index, with historical overview/non-goal detail in `product-spec/overview.md`, core concepts in `product-spec/core-concepts.md`, user journeys in `product-spec/user-journeys.md`, and UX/security requirements in `product-spec/ux-security.md`. `workpads/research/tasks.md` was not compacted; it only records this completed support-file split. Routing checks, line-count checks, and `git diff --check` passed.

### ✅ Task I19cw: Support Crawl4AI internal link exclusion cleanup option

Acceptance criteria:

- Inspect Crawl4AI source for `exclude_internal_links`, base-domain classification, and special-scheme link handling before changing owned extraction.
- Add owned backend option support for `crawl4ai.exclude_internal_links` with source-backed default `false`.
- When enabled, remove same-base-domain and relative `<a href>` elements from owned cleaned HTML before markdown/text/json extraction while keeping external and special-scheme links.
- Keep Crawl4AI command compatibility helper and mock-backend option validation aligned with the new option.
- Record the source-backed cleanup boundary in `knowledge.md`.
- Verify with focused owned cleanup/option coverage plus the standard check set.

Status note:

- Completed with D266. Source inspection found Crawl4AI exposes `CrawlerRunConfig.exclude_internal_links`, and Crawl4AI's URL helpers classify relative and same-base-domain URLs as internal while treating special schemes as external. Owned extraction now supports `crawl4ai.exclude_internal_links=false` by default and removes relative/same-base-domain anchors before owned content extraction when enabled, while preserving external and special-scheme links. The Crawl4AI command helper, mock-backend validation, README option list, and unsupported-option error text are aligned. Focused cleanup, mock-site extractor, command-option validation, and the standard `cargo fmt --check && cargo test && git diff --check` gate passed.

### ✅ Task I19cx: Preserve Crawl4AI linked-heading markdown behavior

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` handling for headings nested inside anchors before changing owned markdown rendering.
- Preserve current normal anchor, linked image, and standalone heading rendering.
- Render anchors whose only non-whitespace child is an `h1`-`h6` as linked headings, e.g. `<a href="/guide"><h2>Guide</h2></a>` becomes `## [Guide](...)`.
- Keep link title, base URL resolution, and link-label inline-code behavior compatible with the existing owned renderer.
- Record the source-backed markdown boundary in `knowledge.md`.
- Verify with focused owned markdown coverage plus the standard check set.

Status note:

- Completed with D267. Source inspection found Crawl4AI's active `CustomHTML2Text` path delegates linked headings to the base heading-in-anchor handling, which renders the heading marker outside the link label. Owned markdown now renders anchors whose only non-whitespace child is an `h1`-`h6` as linked headings while preserving normal anchors, linked images, standalone headings, titles, base URL resolution, and linked inline-code label behavior. Focused mock-site markdown coverage and the standard `cargo fmt --check && cargo test && git diff --check` gate passed.

### ✅ Task I19cy: Split R12 architecture proposal archive support file

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Convert `workpads/research/archive/knowledge/r12-mvp-architecture-proposal.md` into a compact routing index.
- Move the dense historical R12 architecture proposal content into smaller referenced files.
- Update current knowledge routing and decision index so the split is discoverable.
- Verify the split with line-count checks and `git diff --check`.

Status note:

- Completed with D268. `workpads/research/archive/knowledge/r12-mvp-architecture-proposal.md` is now a 9-line routing index, with historical architecture/boundary/CLI detail, config/session/output detail, and privacy/integration/implementation detail moved into three smaller section files under `archive/knowledge/r12-mvp-architecture-proposal/`. `workpads/research/tasks.md` was not compacted. Line-count checks and `git diff --check` passed.

### ✅ Task I19cz: Split D92-D108 markdown/browser archive support file

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Convert `workpads/research/archive/knowledge/d092-d108-markdown-browser-slices.md` into a compact routing index.
- Move the dense historical D92-D108 markdown/browser decision content into smaller referenced files.
- Update current knowledge routing and decision index so the split is discoverable.
- Verify the split with line-count checks and `git diff --check`.

Status note:

- Completed with D269. `workpads/research/archive/knowledge/d092-d108-markdown-browser-slices.md` is now a 9-line routing index, with historical D92-D97 markdown/startup detail, D98-D103 inline-markdown/CDP-discovery detail, and D104-D108 list/blockquote/link-escaping detail moved into three smaller section files under `archive/knowledge/d092-d108-markdown-browser-slices/`. `workpads/research/tasks.md` was not compacted. Line-count checks and `git diff --check` passed.

### ✅ Task I19da: Split D64-D76 browser/default-switch archive support file

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Convert `workpads/research/archive/knowledge/d064-d076-browser-default-switch.md` into a compact routing index.
- Move the dense historical D64-D76 browser/default-switch decision content into smaller referenced files.
- Update current knowledge routing and decision index so the split is discoverable.
- Verify the split with line-count checks and `git diff --check`.

Status note:

- Completed with D270. `workpads/research/archive/knowledge/d064-d076-browser-default-switch.md` is now a 9-line routing index, with historical D64-D68 Chrome import/login detail, D69-D73 extraction/rendering option detail, and D74-D76 owned-default/audit detail moved into three smaller section files under `archive/knowledge/d064-d076-browser-default-switch/`. `workpads/research/tasks.md` was not compacted. Line-count checks and `git diff --check` passed.

### ✅ Task I19db: Split Aget engine refactor plan support file

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Convert `workpads/research/aget-engine-refactor-plan.md` into a compact routing index.
- Move the dense historical engine-refactor plan content into smaller referenced files.
- Update current knowledge routing and decision index so the split is discoverable.
- Verify the split with line-count checks and `git diff --check`.

Status note:

- Completed with D271. `workpads/research/aget-engine-refactor-plan.md` is now an 8-line routing index, with historical engine goal/boundary/naming detail and migration/testing/guardrail detail moved into two smaller section files under `workpads/research/aget-engine-refactor-plan/`. `workpads/research/tasks.md` was not compacted. Line-count checks and `git diff --check` passed.

### ✅ Task I19dc: Split D77-D91 owned-extractor archive support file

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Convert `workpads/research/archive/knowledge/d077-d091-owned-extractor-options-cleanup.md` into a compact routing index.
- Move the dense historical D77-D91 owned extractor/browser decision content into smaller referenced files.
- Update current knowledge routing and decision index so the split is discoverable.
- Verify the split with line-count checks and `git diff --check`.

Status note:

- Completed with D272. `workpads/research/archive/knowledge/d077-d091-owned-extractor-options-cleanup.md` is now a 9-line routing index, with historical D77-D82 option/readiness detail, D83-D85 browser state/fallback detail, and D86-D91 cleanup/markdown detail moved into three smaller section files under `archive/knowledge/d077-d091-owned-extractor-options-cleanup/`. `workpads/research/tasks.md` was not compacted. Line-count checks and `git diff --check` passed.

### ✅ Task I19dd: Split D109-D122 selector/overlay archive support file

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Convert `workpads/research/archive/knowledge/d109-d122-selector-overlay-shadow.md` into a compact routing index.
- Move the dense historical D109-D122 selector/overlay/shadow decision content into smaller referenced files.
- Update current knowledge routing and decision index so the split is discoverable.
- Verify the split with line-count checks and `git diff --check`.

Status note:

- Completed with D273. `workpads/research/archive/knowledge/d109-d122-selector-overlay-shadow.md` is now a 9-line routing index, with historical D109-D115 markdown/selector/escaping detail, D116-D120 Chrome/overlay cleanup detail, and D121-D122 linked-image/shadow-DOM detail moved into three smaller section files under `archive/knowledge/d109-d122-selector-overlay-shadow/`. `workpads/research/tasks.md` was not compacted. Line-count checks and `git diff --check` passed.

### ✅ Task I19de: Split OAuth-safe browser login design support file

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Convert `workpads/research/oauth-safe-browser-login-design.md` into a compact routing index.
- Move the dense OAuth-safe browser login design content into smaller referenced files.
- Update current knowledge routing and decision index so the split is discoverable.
- Verify the split with line-count checks and `git diff --check`.

Status note:

- Completed with D274. `workpads/research/oauth-safe-browser-login-design.md` is now an 8-line routing index, with historical vocabulary/decision-tree detail and browser/errors/tests/manual-smoke detail moved into two smaller section files under `workpads/research/oauth-safe-browser-login-design/`. `workpads/research/tasks.md` was not compacted. Line-count checks and `git diff --check` passed.

### ✅ Task I19df: Support Crawl4AI iframe processing option

Acceptance criteria:

- Inspect Crawl4AI source for `process_iframes` before changing owned CDP rendering.
- Add owned backend option support for `crawl4ai.process_iframes` with source-backed default `false`.
- When enabled, force the owned CDP rendering path and replace accessible iframe elements with extracted iframe body content before HTML capture.
- Treat inaccessible/cross-origin iframe processing as a warning rather than a hard extraction failure.
- Keep the compatibility command helper and unsupported-option error text aligned.
- Record the source-backed iframe-processing boundary in `knowledge.md`.
- Verify with deterministic script/option coverage plus an ignored local-Chrome iframe smoke and the standard check set.

Status note:

- Completed with D275. Source inspection found Crawl4AI `AsyncCrawlerStrategy.process_iframes` assigns iframe IDs, waits for content frames, extracts `document.body.innerHTML`, and replaces accessible iframes with `div.extracted-iframe-content-*` before capture while continuing past inaccessible frames. Owned extraction now supports `crawl4ai.process_iframes=false` by default, forces the CDP-rendered path when enabled, warns instead of failing on inaccessible iframe processing, and keeps the Crawl4AI command helper, mock backend validation, README option list, and unsupported-option text aligned. Focused script/option coverage, the ignored local-Chrome iframe smoke, and the standard `cargo fmt --check && cargo test && git diff --check` gate passed.

### ✅ Task I19dg: Split browser CDP page scripts by behavior

Acceptance criteria:

- Move the mixed CDP page-script helpers out of `src/browser_cdp/page_scripts.rs` into behavior-focused modules.
- Preserve all existing helper names and visibility through the `page_scripts` module route.
- Keep storage, readiness/scroll, overlay cleanup, iframe processing, and shadow DOM scripts separated for future agent work.
- Do not compact `workpads/research/tasks.md`.
- Record the mechanical split in `knowledge.md`.
- Verify with focused page-script/CDP tests plus the standard check set.

Status note:

- Completed with D276. `src/browser_cdp/page_scripts.rs` was replaced by `src/browser_cdp/page_scripts/` modules for storage, readiness/scroll, overlay cleanup, iframe processing, and shadow DOM scripts, with `mod.rs` preserving the existing helper route and visibility for browser/CDP callers and tests. `workpads/research/tasks.md` was not compacted. Focused page-script/CDP tests passed.

### ✅ Task I19dh: Support Crawl4AI local-content URLs in owned static extraction

Acceptance criteria:

- Inspect Crawl4AI source for `raw:`, `raw://`, and `file://` URL handling before changing owned extraction.
- Add owned static extraction support for explicit `raw:`, `raw://`, and `file://` inputs without applying cookie/session state to local content.
- Preserve existing HTTP(S) fetch, cookie matching, and rendered-page behavior.
- Record the local-content compatibility and privacy boundary in `knowledge.md`.
- Verify with focused owned-extractor local-content coverage plus the standard check set.

Status note:

- Completed with D277. Source inspection found Crawl4AI accepts explicit `raw:`, `raw://`, and `file://` inputs, strips raw prefixes directly to preserve characters like `#`, reads local files for `file://`, and only routes local content through the browser path when browser-only options require it. Owned static extraction now accepts the same local-content prefixes before HTTP URL parsing, keeps local responses on the static non-network path, documents the CLI input shape, and preserves the privacy boundary by composing cookies only for HTTP(S) and rejecting named-session replay for local-content inputs. Focused owned-fetch, CLI parser, replay-scope, and mock-site extractor coverage passed.

### ✅ Task I19di: Support Crawl4AI `base_url` for raw/local HTML link resolution

Acceptance criteria:

- Inspect Crawl4AI source for `CrawlerRunConfig.base_url` and markdown base URL selection before changing owned extraction.
- Add owned backend option support for `crawl4ai.base_url` so raw/local HTML can resolve relative links without a `<base>` tag.
- Preserve the existing precedence where an HTML `<base href>` overrides the configured base URL for markdown/link cleanup.
- Keep the compatibility command helper, mock-backend validation, README option list, and unsupported-option error text aligned.
- Record the source-backed base-URL boundary in `knowledge.md`.
- Verify with focused raw/local markdown coverage and the standard check set.

Status note:

- Completed with D278. Source inspection found Crawl4AI stores `CrawlerRunConfig.base_url` for markdown link resolution, preserves it for raw/local content instead of falling back to the raw HTML string, and then lets an HTML `<base href>` override the configured/effective URL before markdown generation. Owned extraction now supports `crawl4ai.base_url` as a non-empty absolute URL, feeds it into the existing markdown/link-cleanup base calculation, preserves `<base href>` precedence, and keeps final URL/session scope unchanged. The Crawl4AI helper, mock-backend validation, README option list, and unsupported-option text are aligned. Focused raw/local markdown, command-option validation, and helper syntax checks passed.

### ✅ Task I19dj: Exclude Crawl4AI-negative main-content candidates

Acceptance criteria:

- Inspect Crawl4AI source for negative class/id handling before changing owned main-content selection.
- Make owned default main-content selection reject candidates marked by generic navigation, advertising, comments, promo, or social class/id labels instead of only down-ranking them.
- Preserve the existing page-chrome tag exclusion and word-threshold behavior.
- Add focused mock-site coverage where a long noisy candidate would otherwise beat the real article.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused main-content coverage and the standard check set.

Status note:

- Completed with D279. Source inspection found Crawl4AI's relevant-content candidate path excludes elements whose class/id text matches generic navigation, footer/header/sidebar, ads, comments, promo, advert, social, or share labels, while its pruning filter also uses the same labels as scoring input. Owned default main-content selection now rejects candidates whose own class/id contains those labels, preserving existing page-chrome ancestor tag exclusion and explicit word-threshold behavior. Focused mock-site coverage verifies that a long noisy comments-labeled article no longer beats the real content candidate.

### ✅ Task I19dk: Split D55-D63 owned-extractor foundation archive

Acceptance criteria:

- Keep `workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation.md` as the stable routing entrypoint.
- Move dense D55-D63 decision detail into smaller referenced files.
- Update current knowledge routing and decision index so the split is discoverable.
- Do not compact `workpads/research/tasks.md`.
- Validate that concatenating the split files reconstructs the prior archive content and `git diff --check` passes.

Status note:

- Completed with D280. `workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation.md` is now a compact routing index, with historical D55-D57 static extractor/markdown detail, D58-D60 fallback/CDP detail, and D61-D63 render-retry/table detail moved into three smaller section files under `archive/knowledge/d055-d063-owned-extractor-foundation/`. `workpads/research/tasks.md` was not compacted. Concatenating the split files reconstructs the previous archive content, line-count checks show the routed entrypoint is 9 lines, and `git diff --check` passed.

### ✅ Task I19dl: Propagate Crawl4AI-style page metadata from owned extraction

Acceptance criteria:

- Inspect Crawl4AI source for page metadata extraction before changing owned extraction.
- Extract generic `<head>` metadata in owned static/rendered extraction before cleanup removes `title`, `meta`, or `link` elements.
- Propagate metadata through `ExtractorBackendResult`, `GetSuccess`, JSON envelopes, and run metadata artifacts without changing content extraction behavior.
- Keep command-backed Crawl4AI compatibility aligned by forwarding `result.metadata` through the helper response when available.
- Add focused mock-site coverage for title, description, keywords, author, Open Graph, Twitter, and article metadata.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned-extractor coverage and the standard check set.

Status note:

- Completed with D281. Source inspection found Crawl4AI extracts generic page metadata before content filtering and cleanup, then exposes it separately as `CrawlResult.metadata`. Owned extraction now captures `<title>`, description, keywords, author, Open Graph, Twitter, and article-prefixed metadata before cleanup removes head elements, propagates it as `page_metadata` through backend results, `GetSuccess`, JSON envelopes, and run metadata artifacts, and keeps the compatibility Crawl4AI helper aligned by forwarding `result.metadata`. README output documentation is aligned. Focused mock-site coverage verifies the public field, envelope output, and metadata artifact.

### ✅ Task I19dm: Preserve page metadata across owned browser fallback extraction

Acceptance criteria:

- Re-check the source-backed metadata boundary from D281 before changing fallback propagation.
- Propagate owned fallback extraction metadata through `BrowserFallbackResult` and `GetSuccess` instead of dropping it after primary extractor failure.
- Keep command-backed `agent-browser` fallback compatibility stable with an empty metadata map because it only requests body HTML/text.
- Add focused session-backed fallback coverage proving page metadata appears in the public success result without changing extracted content.
- Record the fallback boundary in `knowledge.md`.
- Verify with focused fallback coverage and the standard check set.

Status note:

- Completed with D282. `BrowserFallbackResult` now carries `page_metadata`, owned browser fallback forwards the metadata produced by the owned extraction pipeline, session-backed fallback finalization places it in `GetSuccess`, and the command-backed `agent-browser` fallback compatibility path intentionally returns an empty map because it only requests body HTML/text. Focused coverage verifies protected-page fallback metadata, unchanged content, cookie replay, and empty command-fallback metadata.

### ✅ Task I19dn: Strengthen agent-browser-style networkidle CDP parity coverage

Acceptance criteria:

- Re-inspect `agent-browser` source for `networkidle` wait behavior before changing owned CDP coverage.
- Add deterministic mock-CDP navigation coverage proving owned `PageWaitUntil::NetworkIdle` resets its quiet window when a new request arrives during the idle period.
- Add deterministic mock-CDP navigation coverage proving owned `PageWaitUntil::NetworkIdle` times out when in-flight requests never settle.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser CDP navigation tests and the standard check set.

Status note:

- Completed with D283. Source inspection confirmed `agent-browser` resets its 500 ms `networkidle` quiet window when new requests arrive, starts a new quiet window when in-flight requests empty, and returns an overall timeout when idle is never reached. Owned CDP navigation already matched that bounded behavior, so this slice added deterministic mock-CDP coverage for reset and timeout behavior without changing runtime code. Validation passed with focused `browser_cdp_networkidle` tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19do: Port agent-browser-style CDP navigation wait timeout messages

Acceptance criteria:

- Re-inspect `agent-browser` source for lifecycle and `networkidle` wait timeout messages before changing owned CDP errors.
- Return specific owned CDP timeout messages for lifecycle waits and `networkidle` instead of the generic Chrome-CDP polling timeout.
- Add deterministic mock-CDP coverage for lifecycle timeout and `networkidle` timeout classification.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser CDP navigation tests and the standard check set.

Status note:

- Completed with D284. Source inspection confirmed `agent-browser` reports lifecycle wait timeouts as `Timeout waiting for {event_name}` and `networkidle` timeouts as `Timeout waiting for networkidle`. Owned CDP navigation now preserves the existing `timeout` error code while reporting `load`, `domcontentloaded`, or `networkidle` as the timed-out wait condition. Deterministic mock-CDP coverage verifies lifecycle and `networkidle` timeout classification. Validation passed with focused browser CDP navigation tests, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19dp: Align agent-facing Crawl4AI option documentation

Acceptance criteria:

- Inspect the owned extractor option parser and current README option list before updating agent-facing tool text.
- Update the OpenCode tool `backend_options` description to include the currently owned Crawl4AI compatibility options.
- Keep README and project skill behavior unchanged if they already route agents to the current option list.
- Record the source-backed boundary in `knowledge.md`.
- Verify with option-list checks and the standard check set.

Status note:

- Completed with D285. Inspection found `.opencode/tools/aget.ts` missing `crawl4ai.base_url`, `crawl4ai.exclude_internal_links`, and `crawl4ai.process_iframes` from the agent-facing `backend_options` description even though the owned parser, README, and mock backend validation allowlist already include those options. The OpenCode tool schema description now matches the current owned option surface; README and the project skill were left unchanged because they were already aligned. Validation passed with option-list `rg` checks, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19dq: Preserve Crawl4AI code-block whitespace in owned markdown

Acceptance criteria:

- Inspect Crawl4AI's active markdown and cleaned-HTML code-block handling before changing owned markdown normalization.
- Preserve raw lines inside owned fenced code blocks while keeping existing normal markdown whitespace cleanup outside code blocks.
- Add deterministic owned markdown coverage for blank lines and trailing spaces inside `<pre><code>`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused markdown coverage plus the standard check set.

Status note:

- Completed with D286. Source inspection found Crawl4AI's active `CustomHTML2Text` emits raw data while inside `<pre>` and Crawl4AI cleaned-HTML pruning skips descendants of `<pre>`/`<code>` so whitespace-only code descendants survive. Owned markdown already rendered fenced code blocks from raw text; the global markdown normalizer now preserves raw fenced-code lines while keeping ordinary whitespace cleanup outside code blocks. Deterministic mock-site coverage verifies blank lines and trailing spaces inside `<pre><code>`. Validation passed with focused markdown coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19dr: Port agent-browser-style Chrome early-exit startup message

Acceptance criteria:

- Inspect `agent-browser` Chrome startup handling before changing owned CDP startup errors.
- Report owned Chrome startup exits before `DevToolsActivePort` with the explicit exit code and missing-port condition.
- Add deterministic coverage for the early-exit message.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser CDP discovery coverage plus the standard check set.

Status note:

- Completed with D287. Source inspection found `agent-browser` reports early Chrome startup exits as `Chrome exited early (exit code: X) without writing DevToolsActivePort`. Owned startup now reports the explicit exit code and missing-`DevToolsActivePort` condition while preserving the existing `backend_unavailable` code, retry behavior, stderr classification, and stderr/port-file discovery fallbacks. Validation passed with focused browser CDP discovery coverage, `cargo fmt --check`, `cargo test`, and `git diff --check`.

### ✅ Task I19ds: Split Browser CDP discovery tests by behavior

Acceptance criteria:

- Split `src/browser_cdp/tests/discovery.rs` into smaller behavior-focused modules without changing production code.
- Preserve all existing discovery, diagnostics, active-port, stale-port, and direct-WebSocket fallback assertions.
- Keep test names and behavior discoverable by the existing `browser_cdp::tests::discovery` module path.
- Record the decomposition boundary in `knowledge.md`.
- Verify with focused browser CDP discovery coverage plus the standard check set.

Status note:

- Completed with D288. `src/browser_cdp/tests/discovery.rs` is now a small module index, with Chrome stderr/startup diagnostics coverage in `discovery/diagnostics.rs`, `DevToolsActivePort` and stale-port cleanup coverage in `discovery/port_file.rs`, and `/json/version`/`json/list`/direct-WebSocket endpoint fallback coverage in `discovery/endpoint.rs`. Production code and assertions are unchanged, and `workpads/research/tasks.md` was not compacted. Validation passed with focused browser CDP discovery coverage plus the standard check set.

### ✅ Task I19dt: Split mock-site docs contract tests by behavior

Acceptance criteria:

- Split `tests/mock_site_docs_contract.rs` into smaller behavior-focused modules without changing production code.
- Preserve the public get, output/artifact, custom-route, session replay/scope, and session lifecycle contract assertions.
- Keep the mock-site docs contract test target discoverable as `mock_site_docs_contract`.
- Record the decomposition boundary in `knowledge.md`.
- Verify with focused docs-contract coverage plus the standard check set.

Status note:

- Completed with D289. `tests/mock_site_docs_contract.rs` is now a small module index, with public get contract coverage in `mock_site_docs_contract/public_get.rs`, output/warning/limit coverage in `output.rs`, custom fixture coverage in `custom_site.rs`, session compose/replay/scope coverage in `session_replay.rs`, and session lifecycle coverage in `session_lifecycle.rs`. Production code and assertions are unchanged, and `workpads/research/tasks.md` was not compacted. Validation passed with focused docs-contract coverage plus the standard check set.

### ✅ Task I19du: Split owned extractor option parsing helpers

Acceptance criteria:

- Split private parser/helper functions out of `src/extraction/owned/options.rs` without changing owned extractor behavior.
- Preserve `validate_owned_extraction_options`, `OwnedExtractorOptions`, supported option names, default values, and error text.
- Keep the JavaScript-wait safety boundary unchanged.
- Record the mechanical split boundary in `knowledge.md`.
- Verify with focused owned extractor option coverage plus the standard check set.

Status note:

- Completed with D290. `src/extraction/owned/options.rs` still owns `OwnedExtractorOptions`, defaults, supported option dispatch, and `validate_owned_extraction_options`, while private URL/list/bool/duration/wait/selector parsing plus CSS-only wait validation now lives in `src/extraction/owned/options/parse.rs`. Supported `crawl4ai.*` option names, defaults, and error text are unchanged; JavaScript waits remain rejected before backend execution. `workpads/research/tasks.md` was not compacted. Validation passed with focused owned extractor option coverage plus the standard check set.

### ✅ Task I19dv: Split owned extraction page rendering helpers

Acceptance criteria:

- Split rendered-page request composition and script/readiness detection helpers out of `src/extraction/owned/page.rs` without changing owned extraction behavior.
- Preserve static-versus-rendered routing, browser render request fields, rendered wait retry behavior, and script type detection assertions.
- Keep `extract_owned_static_or_rendered` and `extract_owned_rendered_html` as the owned page extraction entrypoints.
- Do not compact `workpads/research/tasks.md`.
- Record the mechanical split boundary in `knowledge.md`.
- Verify with focused owned extraction/rendering coverage plus the standard check set.

Status note:

- Completed with D291. `src/extraction/owned/page.rs` still owns the owned page extraction entrypoints, HTTP/static handoff, HTML cleanup sequencing, selector/fallback selection, base-URL handling, output-format shaping, and metadata propagation, while `src/extraction/owned/page/rendered.rs` owns CDP render request composition and rendered-wait retry matching, and `src/extraction/owned/page/readiness.rs` owns script/readiness detection and its assertions. Behavior is unchanged, the JavaScript-wait safety boundary is unchanged, and `workpads/research/tasks.md` was not compacted. Validation passed with focused owned page/readiness and mock-site rendering coverage plus the standard check set.

### ✅ Task I19dw: Split Aget facade session orchestration helpers

Acceptance criteria:

- Split session list/load/delete, import, authorization, compose, and login orchestration methods out of `src/aget/mod.rs` without changing the public `AgetWith` API.
- Keep `src/aget/mod.rs` focused on type wiring, constructors, backend replacement helpers, timeout/home access, get-request creation, and shared facade helpers.
- Preserve default owned backend wiring and all session/import/login behavior.
- Do not compact `workpads/research/tasks.md`.
- Record the mechanical split boundary in `knowledge.md`.
- Verify with focused Aget facade/API and session CLI coverage plus the standard check set.

Status note:

- Completed with D292. `src/aget/mod.rs` remains focused on facade module routing, public re-exports, default owned backend constructors, backend replacement helpers, timeout/home access, get-request creation, and shared facade helpers, while `src/aget/sessions.rs` owns session list/load/delete, cmux/Chrome import, authorization verification, session composition, and login start/finish/cancel orchestration methods on `AgetWith`. The public API and behavior are unchanged, default owned backend wiring is unchanged, and `workpads/research/tasks.md` was not compacted. Validation passed with focused Aget API and session CLI coverage plus the standard check set.

### ✅ Task I19dx: Support Crawl4AI `skip_internal_links` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown behavior.
- Add owned backend option support for `crawl4ai.skip_internal_links` with the source-backed default of `false`.
- Preserve the existing default fragment-link behavior while suppressing fragment-only link targets, not visible text, when opted in.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D293. Source inspection found Crawl4AI's active `CustomHTML2Text` defaults `skip_internal_links` to `false` and suppresses only raw fragment-only `href` targets when enabled. Owned extraction now supports `crawl4ai.skip_internal_links=false` by default, preserves existing fragment-link markdown by default, and renders fragment-only anchors as visible text without markdown link targets when opted in. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19dy: Support Crawl4AI `include_sup_sub` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown behavior.
- Add owned backend option support for `crawl4ai.include_sup_sub` with the source-backed default of `false`.
- Preserve the default plain-text rendering of `sup`/`sub` content while emitting literal `<sup>` and `<sub>` wrappers when opted in.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D294. Source inspection found Crawl4AI's active `CustomHTML2Text` defaults `include_sup_sub` to `false` and emits literal `<sup>`/`<sub>` wrappers only when the option is enabled. Owned extraction now supports `crawl4ai.include_sup_sub=false` by default, preserves existing plain-text `sup`/`sub` output by default, and emits literal wrappers when opted in. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19dz: Support Crawl4AI `ignore_links` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown behavior.
- Add owned backend option support for `crawl4ai.ignore_links` with the source-backed default of `false`.
- Preserve the existing default link markdown behavior while rendering anchor children as plain visible content when opted in.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D295. Source inspection found Crawl4AI's active `CustomHTML2Text` defaults `ignore_links` to `false`, and anchor markdown handling runs only when `ignore_links` is false. Owned extraction now supports `crawl4ai.ignore_links=false` by default, preserves existing link markdown by default, and renders anchor children as visible content without link targets when opted in; child images inside ignored anchors still render as images. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ea: Support Crawl4AI `ignore_images` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown behavior.
- Add owned backend option support for `crawl4ai.ignore_images` with the source-backed default of `false`.
- Preserve the existing default image markdown behavior while suppressing image markdown when opted in.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D296. Source inspection found Crawl4AI's markdown generator defaults `ignore_images` to `false`, and html2text image handling runs only when `ignore_images` is false. Owned extraction now supports `crawl4ai.ignore_images=false` by default, preserves existing image markdown by default, and suppresses image markdown when opted in while preserving enclosing anchor behavior for linked images. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19eb: Support Crawl4AI `ignore_emphasis` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown behavior.
- Add owned backend option support for `crawl4ai.ignore_emphasis` with the source-backed default of `false`.
- Preserve the existing default emphasis/strong markdown behavior while rendering emphasized children as plain visible content when opted in.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D297. Source inspection found Crawl4AI's markdown generator defaults `ignore_emphasis` to `false`, and `CustomHTML2Text` gates `em`/`i`/`u` plus `strong`/`b` markdown markers behind the option while keeping strikethrough handling separate. Owned extraction now supports `crawl4ai.ignore_emphasis=false` by default, preserves existing emphasis and strong markdown by default, and renders those children as visible content without emphasis markers when opted in while leaving strikethrough unchanged. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ec: Support Crawl4AI `protect_links` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown link behavior.
- Add owned backend option support for `crawl4ai.protect_links` with the source-backed default of `false`.
- Preserve the existing default markdown link target behavior while wrapping link targets in angle brackets when opted in.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D298. Source inspection found Crawl4AI's markdown generator defaults `protect_links` to `false`, and `CustomHTML2Text` wraps anchor `href` targets in angle brackets when the option is enabled while leaving automatic absolute links on their existing `<url>` path. Owned extraction now supports `crawl4ai.protect_links=false` by default, preserves existing escaped link targets by default, and wraps anchor targets in `<...>` when opted in, including linked-heading and linked-image anchor targets while leaving image `src` targets unchanged. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ed: Support Crawl4AI `escape_snob` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown text escaping.
- Add owned backend option support for `crawl4ai.escape_snob` with the source-backed default of `false`.
- Preserve existing default text escaping while escaping Crawl4AI's broader normal-text markdown character set when opted in.
- Keep code/pre text on the existing raw/code-specific path rather than broad escaping it.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D299. Source inspection found Crawl4AI's markdown generator defaults `escape_snob` to `false`, and `CustomHTML2Text` delegates normal text through the broader markdown-character escaping path while keeping inline code and preformatted code on code-specific paths. Owned extraction now supports `crawl4ai.escape_snob=false` by default, preserves existing default text escaping, and escapes backtick, star, underscore, braces, brackets, parens, hash, and bang in normal text when opted in while leaving generated markdown syntax and code/pre text unchanged. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ee: Support Crawl4AI `ignore_mailto_links` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown link behavior.
- Add owned backend option support for `crawl4ai.ignore_mailto_links` with the source-backed active markdown default of `true`.
- Preserve existing default mailto suppression while rendering mailto anchors as markdown links when opted out.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D300. Source inspection found Crawl4AI's active `CustomHTML2Text` defaults `ignore_mailto_links` to `true` and skips `mailto:` href targets only while that option is true. Owned extraction now supports `crawl4ai.ignore_mailto_links=true` by default, preserves existing mailto suppression by default, and renders mailto anchors as markdown links when opted out. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ef: Support Crawl4AI `ignore_tables` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown table behavior.
- Add owned backend option support for `crawl4ai.ignore_tables` with the source-backed markdown default of `false`.
- Preserve existing default table markdown while rendering table content without markdown table syntax when opted in.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D301. Source inspection found Crawl4AI's active markdown path defaults `ignore_tables` to `false`; when enabled, `CustomHTML2Text` ignores table-related tags while keeping row content and row breaks. Owned extraction now supports `crawl4ai.ignore_tables=false` by default, preserves existing markdown table output by default, and renders caption plus row/cell content as plain markdown lines when opted in. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19eg: Support Crawl4AI `bypass_tables` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown table behavior.
- Add owned backend option support for `crawl4ai.bypass_tables` with the source-backed markdown default of `false`.
- Preserve existing default markdown table rendering while rendering table structure as HTML-style tags when opted in.
- Keep inline markdown behavior inside table cells compatible with existing owned link, image, emphasis, and code options.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D302. Source inspection found Crawl4AI's active markdown path defaults `bypass_tables` to `false`, checks `ignore_tables` first, and emits HTML-style table-related tags when bypass mode is enabled instead of markdown table syntax. Owned extraction now supports `crawl4ai.bypass_tables=false` by default, preserves existing markdown table output by default, keeps `ignore_tables` stronger when both options are set, and renders compact HTML-style table/row/cell tags with inline markdown content when opted in. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. Source inspection also found `mark_code` is not useful for the active `CustomHTML2Text` path because pre/code markdown is hardcoded there. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19eh: Support Crawl4AI `use_automatic_links` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown anchor behavior.
- Add owned backend option support for `crawl4ai.use_automatic_links` with the source-backed markdown default of `true`.
- Preserve existing default automatic absolute-link rendering while rendering same-text absolute anchors as explicit markdown links when opted out.
- Keep link titles, protected links, skip-internal links, ignored links, and mailto handling compatible with existing owned link options.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D303. Source inspection found Crawl4AI defaults `use_automatic_links` to true, records candidate anchor hrefs, and emits `<url>` only when the anchor text exactly matches an absolute URL and the option remains enabled; otherwise it falls back to normal inline markdown link handling. Owned extraction now supports `crawl4ai.use_automatic_links=true` by default, preserves existing automatic absolute-link output by default, and renders same-text absolute anchors as explicit markdown links with titles when opted out. The existing ignore-links, skip-internal, mailto, and protect-links gates remain aligned. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ei: Support Crawl4AI `images_to_alt` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown image behavior.
- Add owned backend option support for `crawl4ai.images_to_alt` with the source-backed markdown default of `false`.
- Preserve existing default image markdown rendering while rendering image alt text only when opted in.
- Keep linked images, default automatic link behavior, ignored images, ignored links, escaped markdown targets, and base-URL handling compatible with existing owned image/link options.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D304. Source inspection found Crawl4AI's active markdown path defaults `images_to_alt` to `false`, emits normal image markdown by default, and when enabled discards image `src` output while keeping escaped alt text; linked images still feed the alt text into anchor rendering, and same-text absolute URL alt values can use the automatic `<url>` path. Owned extraction now supports `crawl4ai.images_to_alt=false` by default, preserves existing `![alt](src)` output by default, renders standalone image alt text only when opted in, and turns linked images into explicit links whose labels are the escaped image alt text. Existing ignore-images, ignore-links, automatic-link, escaped-target, and base-URL behavior remains aligned. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ej: Support Crawl4AI `images_as_html` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown image behavior.
- Add owned backend option support for `crawl4ai.images_as_html` with the source-backed markdown default of `false`.
- Preserve existing default image markdown rendering while rendering raw HTML `<img>` tags with `src`, `width`, `height`, and `alt` attributes when opted in.
- Keep ignored images, image alt-only mode, linked images, escaped markdown targets, and base-URL handling compatible with existing owned image/link options.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D305. Source inspection found Crawl4AI's active markdown path defaults `images_as_html` to `false`, emits raw `<img ... />` HTML before normal image markdown and `images_to_alt` branches when opted in, and preserves `src`, `width`, `height`, and `alt` attributes when present. Owned extraction now supports `crawl4ai.images_as_html=false` by default, preserves existing `![alt](src)` output by default, renders raw image HTML when opted in, keeps `images_as_html` stronger than `images_to_alt` when both are set, and leaves `ignore_images` as the stronger image gate. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ek: Support Crawl4AI `images_with_size` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown image behavior.
- Add owned backend option support for `crawl4ai.images_with_size` with the source-backed markdown default of `false`.
- Preserve existing default image markdown rendering while rendering raw HTML `<img>` tags only for images with `width` or `height` attributes when opted in.
- Keep ignored images, image alt-only mode, raw-image HTML mode, linked images, escaped markdown targets, and base-URL handling compatible with existing owned image/link options.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D306. Source inspection found Crawl4AI's active markdown path defaults `images_with_size` to `false` and emits raw `<img ... />` HTML when `images_with_size` is enabled and the image has a `width` or `height` attribute, before normal image markdown and `images_to_alt` handling. Owned extraction now supports `crawl4ai.images_with_size=false` by default, preserves existing `![alt](src)` output by default, renders sized images as raw image HTML when opted in, leaves unsized images on normal markdown output unless another option changes them, and keeps `images_with_size` stronger than `images_to_alt` for sized images while leaving `ignore_images` as the stronger image gate. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19el: Support Crawl4AI `default_image_alt` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned markdown image behavior.
- Add owned backend option support for `crawl4ai.default_image_alt` with the source-backed markdown default of an empty string.
- Preserve existing default image markdown rendering while using the configured default alt text for images with missing or empty `alt` attributes.
- Keep normal image markdown, image alt-only mode, raw-image HTML modes, linked images, ignored images, escaped markdown targets, and base-URL handling compatible with existing owned image/link options.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D307. Source inspection found Crawl4AI defaults `default_image_alt` to an empty string and computes `alt = attrs.get("alt") or self.default_image_alt`, so the effective alt is reused by normal image markdown, `images_to_alt`, and raw image HTML modes. Owned extraction now supports `crawl4ai.default_image_alt` with the empty default, preserves existing empty alt output by default, applies the configured fallback to missing or empty image `alt` attributes, and preserves non-empty source alt text as stronger than the configured default. The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`, and mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19em: Compact current workpad support routing without compacting tasks

Acceptance criteria:

- Keep `workpads/research/tasks.md` as the full executable backlog; do not compact existing task content.
- Move dense current support-file routing and completed-history detail from top-level support files into referenced archive files.
- Keep top-level `knowledge.md` and `references.md` focused on current direction, open work, routing, and verification expectations.
- Record the compaction decision in `archive/knowledge/` and update the current decision index.
- Verify archive links and diff hygiene.

Status note:

- Completed with D308. `tasks.md` was not compacted. Dense current migration/support routing moved from top-level `knowledge.md` into `archive/knowledge/d308-workpad-support-routing.md`; the current decision index routes through D308. Support-file line-length checks, route checks, and diff hygiene passed.

### ✅ Task I19en: Support Crawl4AI `open_quote` and `close_quote` markdown options

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned quote behavior.
- Add owned backend option support for `crawl4ai.open_quote` and `crawl4ai.close_quote` with source-backed defaults of `"`.
- Preserve existing default `<q>` markdown rendering while using configured opening and closing quote strings when supplied.
- Keep quote output compatible with emphasis, inline code, `only_text`, and existing inline normalization.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D309. Source inspection found Crawl4AI defaults both quote markers to `"` and emits `open_quote` on `<q>` entry plus `close_quote` on exit, with `DefaultMarkdownGenerator(options=...)` forwarding custom values. Owned extraction now supports `crawl4ai.open_quote` and `crawl4ai.close_quote` with the double-quote defaults, preserves existing `<q>` output by default, applies configured quote markers around inline `<q>` content, and keeps the `only_text` path text-only. The Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19eo: Support Crawl4AI `ul_item_mark` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned unordered-list behavior.
- Add owned backend option support for `crawl4ai.ul_item_mark` with the source-backed default of `*`.
- Preserve existing default unordered-list markdown while using the configured marker string for unordered list items.
- Keep nested lists, ordered lists, link/image rendering inside list items, and existing markdown normalization compatible.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D310. Source inspection found Crawl4AI defaults `ul_item_mark` to `*`, emits `self.ul_item_mark + " "` for unordered list items, and forwards custom values through `DefaultMarkdownGenerator(options=...)`. Owned extraction now supports `crawl4ai.ul_item_mark`, preserves existing `*` unordered-list output by default, applies configured markers to unordered items including nested lists, and leaves ordered numbering unchanged. The Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19ep: Support Crawl4AI emphasis marker markdown options

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned emphasis behavior.
- Add owned backend option support for `crawl4ai.emphasis_mark` and `crawl4ai.strong_mark` with source-backed defaults of `_` and `**`.
- Preserve existing default `em`/`i`/`u` and `strong`/`b` markdown output.
- Use configured markers for emphasis and strong output when supplied.
- Keep `ignore_emphasis`, inline code, quote output, link labels, abbreviation definitions, and normalization compatible.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D311. Source inspection found Crawl4AI defaults `emphasis_mark` to `_` and `strong_mark` to `**`, uses those strings for `em`/`i`/`u` and `strong`/`b` output when emphasis is not ignored, and maps CLI `--asterisk-emphasis` to `emphasis_mark="*"` plus `strong_mark="__"`. Owned extraction now supports `crawl4ai.emphasis_mark` and `crawl4ai.strong_mark`, preserves existing `_` and `**` output by default, applies configured marker strings when supplied, and keeps `ignore_emphasis=true` stronger than both marker options. The Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19eq: Split mock-site extractor markdown assertions

Acceptance criteria:

- Preserve current mock-site owned extractor markdown coverage and assertions.
- Keep `tests/mock_site_cli/aget_extractor/markdown.rs` as the module route for the existing parent integration test.
- Move cohesive markdown assertion groups into smaller child modules by behavior.
- Keep the split mechanical with no intentional extraction behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundaries in `knowledge.md`.
- Verify with focused mock-site extractor coverage plus the standard check set.

Status note:

- Completed with D312. `tests/mock_site_cli/aget_extractor/markdown.rs` remains the module route for the existing parent integration test and now delegates to behavior-owned child modules for table/base-link coverage, inline/block constructs, lists/code blocks, and link/image options. The split is mechanical, extraction runtime behavior is unchanged, and `workpads/research/tasks.md` was not compacted. The markdown test route dropped from 511 lines to a 38-line parent plus smaller child modules, with the largest child at 230 lines. Focused mock-site extractor coverage and the standard gate passed.

### ✅ Task I19er: Split markdown block rendering helpers

Acceptance criteria:

- Preserve current owned markdown output behavior.
- Keep `src/extraction/markdown/mod.rs` as the caller-facing markdown route.
- Move cohesive block-level helpers out of the markdown route into a smaller child module.
- Keep the split mechanical with no intentional extraction behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting markdown module boundary in `knowledge.md`.
- Verify with focused owned markdown coverage plus the standard check set.

Status note:

- Completed with D313. `src/extraction/markdown/mod.rs` remains the caller-facing markdown route and keeps `element_to_markdown`, node dispatch, inline-tag decisions, and the `only_text` eligibility table, while `src/extraction/markdown/block.rs` now owns heading, block, horizontal-rule, list, definition-list, code-block, blockquote, and structural-block helpers. The split is mechanical, extraction runtime behavior is unchanged, and `workpads/research/tasks.md` was not compacted. The markdown route dropped from 345 lines to 188 lines, with a 168-line block helper. Focused mock-site extractor coverage and the standard gate passed.

### ✅ Task I19es: Split owned extractor option application helpers

Acceptance criteria:

- Preserve current owned extractor backend-option validation behavior and error messages.
- Keep `src/extraction/owned/options.rs` as the caller-facing options route.
- Move the long backend-option application match into a smaller child module.
- Keep the split mechanical with no intentional extraction behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting options module boundary in `knowledge.md`.
- Verify with focused backend-option validation coverage plus the standard check set.

Status note:

- Completed with D314. `src/extraction/owned/options.rs` remains the caller-facing options route and owns `OwnedExtractorOptions`, defaults, CSS-wait precheck, Crawl4AI namespace checks, and the validator entrypoint, while `src/extraction/owned/options/apply.rs` now owns per-key option application, typed parsing calls, field assignment, and supported-option error text. The split is mechanical, extraction runtime behavior and unsupported-option messages are unchanged, and `workpads/research/tasks.md` was not compacted. The options route dropped from 325 lines to 135 lines, with a 189-line apply helper. Focused backend-option validation coverage and the standard gate passed.

### ✅ Task I19et: Split Playwright session composition tests

Acceptance criteria:

- Preserve current Playwright session composition and temp-state test coverage.
- Keep `src/session/playwright/tests.rs` as the test module route.
- Move cohesive Playwright test groups into smaller child modules by behavior.
- Keep the split mechanical with no intentional session behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused `session::playwright` tests plus the standard check set.

Status note:

- Completed with D315. `src/session/playwright/tests.rs` remains the route and delegates to child modules for Playwright state composition, composed-session behavior, temp state files, and shared fixtures. The split is mechanical; session runtime behavior is unchanged, and `workpads/research/tasks.md` was not compacted. The parent route is 4 lines and the largest child is 143 lines. Focused `session::playwright` coverage and the standard gate passed.

### ✅ Task I19eu: Split mock-site session integration tests

Acceptance criteria:

- Preserve current mock-site session replay, auth-state, Chrome import, and login bootstrap coverage.
- Keep `tests/mock_site_sessions.rs` as the integration-test route.
- Move cohesive mock-site session scenarios into smaller child modules by behavior.
- Keep the split mechanical with no intentional CLI, session, or mock-site behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with the focused mock-site session integration test plus the standard check set.

Status note:

- Completed with D316. `tests/mock_site_sessions.rs` remains the integration-test route and delegates to child modules for replay, unauthenticated/expired/logout states, Chrome import, and login bootstrap scenarios. The split is mechanical; CLI, session, mock-site, and backend behavior are unchanged, and `workpads/research/tasks.md` was not compacted. The parent route is 10 lines and all child modules are 104 lines or less. Focused `mock_site_sessions` coverage and the standard gate passed.

### ✅ Task I19ev: Split Crawl4AI compatibility adapter script

Acceptance criteria:

- Preserve the current `scripts/crawl4ai_extract.py` executable entrypoint and command-line contract.
- Move cohesive helper groups into smaller Python modules by behavior.
- Keep the split mechanical with no intentional Crawl4AI adapter behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting adapter boundary in `knowledge.md`.
- Verify Python syntax/imports, pre-import validation paths, and the standard Rust check set.

Status note:

- Completed with D317. `scripts/crawl4ai_extract.py` remains the executable compatibility adapter route and delegates option parsing/signature checks, private file writes, and content selection/text extraction to `scripts/aget_crawl4ai_compat/` helper modules. The split is mechanical; command-line arguments, JSON response shape, metadata writes, validation messages, and Crawl4AI invocation behavior are intended unchanged, and `workpads/research/tasks.md` was not compacted. The executable route is 132 lines and all helper modules are 122 lines or less. Python syntax, pre-Crawl4AI-import validation paths, focused get CLI validation coverage, and the standard gate passed.

### ✅ Task I19ew: Split CLI session parser unit tests

Acceptance criteria:

- Preserve current `aget session` parser unit-test coverage.
- Keep `src/cli/tests/session.rs` as the session parser test route.
- Move cohesive session parser test groups into smaller child modules by command surface.
- Keep the split mechanical with no intentional CLI parser behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused CLI session parser tests plus the standard check set.

Status note:

- Completed with D318. `src/cli/tests/session.rs` remains the CLI session parser test route and delegates inspect/compose, authorize, import, and login parser assertions to command-surface child modules. The split is mechanical; CLI parser behavior and public command names are unchanged, and `workpads/research/tasks.md` was not compacted. The parent route is 4 lines and all child modules are 137 lines or less. Focused `cli::tests::session` coverage and the standard gate passed.

### ✅ Task I19ex: Split attached-page CDP client tests

Acceptance criteria:

- Preserve current attached-page CDP client test coverage.
- Keep `src/browser_cdp/tests/chrome/cdp_client/attached_page.rs` as the attached-page test route.
- Move cohesive attached-page CDP scenarios into smaller child modules by behavior.
- Keep the split mechanical with no intentional browser/CDP behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused attached-page CDP tests plus the standard check set.

Status note:

- Completed with D319. `src/browser_cdp/tests/chrome/cdp_client/attached_page.rs` remains the attached-page CDP test route and delegates current-page capture, full-page scan, and no-page-target error coverage to behavior-owned child modules. The split is mechanical; browser/CDP runtime behavior is unchanged, and `workpads/research/tasks.md` was not compacted. The parent route is 3 lines and all child modules are 136 lines or less. Focused attached-page CDP coverage and the standard gate passed.

### ✅ Task I19ey: Split login-finish session CLI tests

Acceptance criteria:

- Preserve current session login-finish integration test coverage.
- Keep `tests/session_cli/login/finish.rs` as the login-finish test route.
- Move cohesive login-finish scenarios into smaller child modules by behavior.
- Keep the split mechanical with no intentional login/session CLI behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused login-finish session CLI tests plus the standard check set.

Status note:

- Completed with D320. `tests/session_cli/login/finish.rs` remains the login-finish integration-test route and delegates successful scoped-save cleanup, existing-bucket merge, and finish failure scenarios to scenario-owned child modules. The split is mechanical; session login CLI behavior is unchanged, and `workpads/research/tasks.md` was not compacted. The parent route is 6 lines and all child modules are 116 lines or less. Focused login-finish coverage and the standard gate passed.

### ✅ Task I19ez: Split browser CDP navigation client tests

Acceptance criteria:

- Preserve current browser CDP navigation/wait test coverage.
- Keep `src/browser_cdp/tests/chrome/cdp_client/navigation.rs` as the navigation test route.
- Move cohesive navigation CDP scenarios into smaller child modules by behavior.
- Keep the split mechanical with no intentional browser/CDP behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused navigation CDP tests plus the standard check set.

Status note:

- Completed with D321. `src/browser_cdp/tests/chrome/cdp_client/navigation.rs` remains the navigation client test route and delegates same-document navigation, `Page.navigate` error text, lifecycle timeout, and network-idle readiness scenarios to behavior-owned child modules. The split is mechanical; browser/CDP runtime behavior is unchanged, and `workpads/research/tasks.md` was not compacted. The parent route is 4 lines and all child modules are 86 lines or less. Focused navigation CDP coverage and the standard gate passed.

### ✅ Task I19fa: Split AgetBrowser engine tests by behavior

Acceptance criteria:

- Preserve current `AgetBrowser` engine test coverage and assertions.
- Keep `src/aget_browser/tests.rs` as the engine test route.
- Move shared mock-CDP helpers and behavior scenarios into smaller child modules.
- Keep the split mechanical with no intentional `AgetBrowser` behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused `AgetBrowser` tests plus the standard check set.

Status note:

- Completed with D322. `src/aget_browser/tests.rs` remains the AgetBrowser engine test route and delegates pending-login cancellation, explicit-port CDP discovery, current-tab rendering, attached-page rendering, and shared mock-CDP helpers to behavior-owned child modules. The split is mechanical; AgetBrowser runtime behavior is unchanged, and `workpads/research/tasks.md` was not compacted. The parent route is 5 lines and all child modules are 140 lines or less. Focused AgetBrowser coverage and the standard gate passed.

### ✅ Task I19fb: Split Playwright session composition implementation

Acceptance criteria:

- Preserve current Playwright state composition and composed-session behavior.
- Keep `session::playwright::{compose_playwright_state, compose_session}` public exports stable.
- Move cookie normalization/conflict helpers, storage merge helpers, Playwright-state composition, and composed-session construction into smaller modules.
- Keep the split mechanical with no intentional session behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting module boundary in `knowledge.md`.
- Verify with focused `session::playwright` tests plus the standard check set.

Status note:

- Completed with D323. `src/session/playwright/compose.rs` remains the composition route and re-exports the stable `compose_playwright_state` and `compose_session` functions while cookie normalization/conflict helpers, storage merge helpers, Playwright-state composition, and persisted composed-session construction live in focused child modules. The split is mechanical; session composition behavior is unchanged, and `workpads/research/tasks.md` was not compacted. Focused `session::playwright` coverage and the standard gate passed.

### ✅ Task I19fc: Split Chrome command import CLI tests

Acceptance criteria:

- Preserve current command-backed Chrome import CLI test coverage and assertions.
- Keep `tests/session_cli/imports/chrome_command.rs` as the Chrome command import test route.
- Move successful import, missing backend, profile-lock, and malformed-state scenarios into smaller child modules.
- Keep the split mechanical with no intentional session import behavior changes.
- Do not compact `workpads/research/tasks.md`.
- Record the resulting test boundary in `knowledge.md`.
- Verify with focused Chrome command import tests plus the standard check set.

Status note:

- Completed with D324. `tests/session_cli/imports/chrome_command.rs` remains the command-backed Chrome import test route and delegates successful import, success assertions, missing-backend classification, profile-lock user action classification, and malformed-state cleanup scenarios to child modules. The split is mechanical; session import behavior is unchanged, and `workpads/research/tasks.md` was not compacted. The parent route is 10 lines and all child modules are 133 lines or less. Focused Chrome command import coverage and the standard gate passed.

### ✅ Task I19fd: Support Crawl4AI `mark_code` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned code markdown behavior.
- Add owned backend option support for `crawl4ai.mark_code` with the source-backed markdown default of `true`.
- Preserve existing owned inline-code and fenced-code markdown output when `mark_code` is omitted, true, or false if source inspection shows no active output difference.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D327. Source inspection found Crawl4AI `DefaultMarkdownGenerator` defaults `mark_code` to `true`, while the active `CustomHTML2Text` path produces the same inline backticks and fenced-code output for `mark_code=true` and `mark_code=false`. Owned extraction now validates and accepts `crawl4ai.mark_code` as a compatibility no-op, preserving existing code markdown output while keeping the Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed.

### ✅ Task I19fe: Support Crawl4AI `single_line_break` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned paragraph-spacing behavior.
- Add owned backend option support for `crawl4ai.single_line_break`.
- Preserve existing owned default markdown spacing unless the option is supplied explicitly.
- Collapse non-code blank lines when `crawl4ai.single_line_break=true`.
- Preserve fenced code block contents while applying compact spacing outside code fences.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary and any default-output parity caveat in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D328. Source inspection found Crawl4AI `DefaultMarkdownGenerator` defaults `single_line_break` to `true`, and direct `CustomHTML2Text` source-snapshot execution showed that true collapses paragraph gaps while false preserves blank paragraph gaps. Owned extraction now validates and accepts `crawl4ai.single_line_break`; when true, it collapses non-code blank lines after markdown normalization while preserving fenced-code internals. Existing owned default markdown spacing is preserved unless the option is supplied explicitly, leaving the default-output parity question recorded for final I19d/I19h review instead of silently changing broad markdown output in this small slice. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed.

### ✅ Task I19ff: Broaden Chrome profile-lock startup classification

Acceptance criteria:

- Re-inspect agent-browser profile/startup docs before changing owned startup diagnostics.
- Classify common Chrome profile/user-data-dir lock variants as `requires_user_action`.
- Preserve relevant Chrome stderr details in the surfaced error message.
- Keep sandbox and backend-unavailable startup classifications unchanged.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser-CDP diagnostics tests plus the standard check set.

Status note:

- Completed with D329. Source inspection confirmed agent-browser's Chrome profile reuse boundary is copied local profile state, with documented locked-file/profile-close handling and direct startup error reporting. Owned Chrome/CDP startup now maps common local Chrome profile-lock variants, including `Process Singleton`, `Singleton Lock`, another Chrome process, and `user data directory is already in use`, to `requires_user_action` while preserving the relevant stderr detail. Sandbox hints, generic backend-unavailable startup errors, silent-exit hints, launch retries, and CDP discovery behavior are unchanged. `workpads/research/tasks.md` was not compacted. Focused browser-CDP diagnostics and the standard gate passed.

### ✅ Task I19fg: Support Crawl4AI `unicode_snob` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` before changing owned entity/Unicode handling.
- Add owned backend option support for `crawl4ai.unicode_snob`.
- Preserve existing owned default Unicode markdown unless the option is supplied explicitly.
- When `crawl4ai.unicode_snob=false`, apply Crawl4AI-style ASCII-ish replacements for common named/numeric entity characters.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary and parser-distinction caveat in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D330. Source inspection found Crawl4AI stores `unicode_snob` on `CustomHTML2Text`, defaults it to `false`, and uses it when named/numeric entity references map through the html2text `UNIFIABLE` replacement table. Owned extraction now validates and accepts `crawl4ai.unicode_snob`; explicit `false` applies Crawl4AI-style ASCII-ish replacements for common decoded entity characters, while `true` and the current owned default preserve Unicode output. Because the owned HTML parser decodes entity references before markdown rendering, D330 records the parser-distinction caveat that explicit `false` normalizes matching literal Unicode characters too. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed.

### ✅ Task I19fh: Support Crawl4AI `wrap_links` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` wrapping behavior before changing owned body-width wrapping.
- Add owned backend option support for `crawl4ai.wrap_links`.
- Preserve existing owned/default body-width wrapping when the option is omitted or true.
- When `crawl4ai.wrap_links=false`, preserve markdown lines that contain inline markdown links during body-width wrapping.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D331. Source inspection found Crawl4AI defaults `wrap_links` to true and uses `skipwrap` to preserve link-containing paragraphs only when `wrap_links=false` during body-width wrapping. Owned extraction now validates and accepts `crawl4ai.wrap_links`; default/true behavior preserves existing owned wrapping, while explicit `false` preserves markdown lines containing inline links when `crawl4ai.body_width` is positive. The slice is scoped to owned inline-link markdown output and does not add Crawl4AI reference-link output. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed.

### ✅ Task I19fi: Support Crawl4AI `wrap_list_items` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` wrapping behavior before changing owned body-width wrapping.
- Add owned backend option support for `crawl4ai.wrap_list_items`.
- Preserve existing/default body-width list-item preservation when the option is omitted or false.
- When `crawl4ai.wrap_list_items=true`, allow body-width wrapping for markdown list item lines.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D332. Source inspection found Crawl4AI defaults `WRAP_LIST_ITEMS` to false and uses `skipwrap` to preserve list-like paragraphs unless `wrap_list_items=true`. Owned extraction now validates and accepts `crawl4ai.wrap_list_items`; default/false behavior preserves existing body-width list-item preservation, while explicit true allows top-level markdown list item lines to wrap. Indented markdown lines remain preserved by the owned renderer's broader indentation-preservation rule. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed.

### ✅ Task I19fj: Support Crawl4AI `wrap_tables` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` wrapping behavior before changing owned body-width table wrapping.
- Add owned backend option support for `crawl4ai.wrap_tables`.
- Preserve existing/default body-width table-line preservation when the option is omitted or false.
- When `crawl4ai.wrap_tables=true`, allow body-width wrapping for markdown table lines.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D333. Source inspection found Crawl4AI defaults `WRAP_TABLES` to false and uses `skipwrap` to preserve table-like paragraphs unless `wrap_tables=true`. Owned extraction now validates and accepts `crawl4ai.wrap_tables`; default/false behavior preserves existing body-width table-line preservation, while explicit true allows markdown table lines to wrap. Because Crawl4AI treats this as a wrapping control rather than a table reflow feature, explicit true can intentionally produce broken markdown table syntax when a table row wraps. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed.

### ✅ Task I19fk: Support Crawl4AI `pad_tables` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` table-padding behavior before changing owned table output.
- Add owned backend option support for `crawl4ai.pad_tables`.
- Preserve existing/default markdown table output when the option is omitted or false.
- When `crawl4ai.pad_tables=true`, pad owned markdown table columns to a shared width.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D334. Source inspection found Crawl4AI applies `pad_tables_in_text` after wrapping when `pad_tables=true`, using marker-buffered table lines and max column widths with a one-character right margin. Owned extraction now validates and accepts `crawl4ai.pad_tables`; default/false behavior preserves existing GFM table output, while explicit true pads owned GFM table blocks to shared column widths. This applies the source-backed padding concept to the owned renderer's current table representation rather than adding Crawl4AI's internal table markers. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed.

### ✅ Task I19fl: Support Crawl4AI `hide_strikethrough` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` strikethrough handling before changing owned markdown output.
- Add owned backend option support for `crawl4ai.hide_strikethrough`.
- Preserve existing/default semantic `<del>`/`<s>`/`<strike>` markdown output when the option is omitted or false.
- When `crawl4ai.hide_strikethrough=true`, suppress CSS `line-through` text while keeping semantic deletion tags source-compatible.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D335. Source inspection found Crawl4AI exposes `hide_strikethrough` as a default-false option relevant to the Google-doc style-emphasis path, suppresses output only when CSS emphasis includes `line-through`, and keeps semantic `<del>`/`<strike>`/`<s>` tag handling separate with `~~` output. Owned extraction now validates and accepts `crawl4ai.hide_strikethrough`; default/false behavior preserves existing semantic strikethrough markdown, while explicit true suppresses inline-style `line-through` elements without changing semantic deletion tags. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19fm: Support Crawl4AI `inline_links` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` reference-link behavior before changing owned link output.
- Add owned backend option support for `crawl4ai.inline_links`.
- Preserve existing/default inline markdown link output when the option is omitted or true.
- When `crawl4ai.inline_links=false`, emit reference-style markdown links and append deterministic link definitions.
- Keep automatic absolute links and ignored/skipped link options compatible with the existing owned behavior.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D336. Source inspection found Crawl4AI defaults `INLINE_LINKS` to true, maps CLI `--reference-links` to `inline_links=false`, emits inline links by default, and when false writes `[label][n]` plus end-of-document `   [n]: url` definitions while reusing numbers for matching `href` and optional `title`. Owned extraction now validates and accepts `crawl4ai.inline_links`; default/true behavior preserves existing inline markdown links, while explicit false emits deterministic reference-style link and image definitions. Existing owned automatic absolute-link, ignored-link, skipped-internal-link, and mailto suppression behavior remains stronger than reference-link conversion. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19fn: Support Crawl4AI `links_each_paragraph` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` paragraph-scoped reference-link flushing before changing owned reference-link output.
- Add owned backend option support for `crawl4ai.links_each_paragraph`.
- Preserve existing/default end-of-document reference definitions when the option is omitted or false.
- When `crawl4ai.inline_links=false` and `crawl4ai.links_each_paragraph=true`, emit link definitions after the paragraph that introduced them.
- Keep automatic absolute links and ignored/skipped link options compatible with the existing owned behavior.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D337. Source inspection found Crawl4AI defaults `LINKS_EACH_PARAGRAPH` to false, maps CLI `--links-after-para` to `links_each_paragraph=true`, and flushes pending reference definitions at paragraph breaks instead of only end-of-document when the option is true. Owned extraction now validates and accepts `crawl4ai.links_each_paragraph`; default/false behavior preserves end-of-document reference definitions, while explicit true with `crawl4ai.inline_links=false` emits paragraph-scoped definitions and assigns repeated later-paragraph links fresh reference numbers. Existing owned automatic absolute-link, ignored-link, skipped-internal-link, and mailto suppression behavior remains stronger than reference-link conversion. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19fo: Support Crawl4AI line-start escape markdown options

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` line-start escaping behavior before changing owned markdown output.
- Add owned backend option support for `crawl4ai.escape_dot`, `crawl4ai.escape_plus`, and `crawl4ai.escape_dash`.
- Preserve existing owned/default protection against accidental ordered, plus, and dash list markers unless the option is supplied explicitly.
- When the corresponding option is false, do not escape that source-backed line-start marker in normal text.
- Keep broad `escape_snob`, backslash escaping, and existing structural list rendering compatible with the line-start controls.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary and default-output caveat in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D338. Source inspection found Crawl4AI's `escape_md_section` applies separate `escape_dot`, `escape_plus`, and `escape_dash` flags to normal text outside code/pre blocks, and `DefaultMarkdownGenerator(options=...)` forwards those values into `CustomHTML2Text`. Owned extraction now validates and accepts `crawl4ai.escape_dot`, `crawl4ai.escape_plus`, and `crawl4ai.escape_dash`; current owned defaults continue escaping accidental line-start ordered-list, plus-bullet, and dash-bullet markers, while explicit false values disable the corresponding escape. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19fp: Support Crawl4AI `escape_backslash` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` backslash escaping behavior before changing owned markdown output.
- Add owned backend option support for `crawl4ai.escape_backslash`.
- Preserve existing owned/default literal-backslash protection unless the option is supplied explicitly.
- When `crawl4ai.escape_backslash=false`, do not add extra escaping before literal backslashes that precede Markdown-sensitive characters in normal text.
- Keep broad `escape_snob`, line-start escaping, code/pre text, and link/image target escaping compatible with the backslash control.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary and default-output caveat in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D339. Source inspection found Crawl4AI's `escape_md_section` applies `escape_backslash` to normal text outside code/pre blocks before broad/snob and line-start escaping, and `DefaultMarkdownGenerator(options=...)` forwards the value into `CustomHTML2Text`. Owned extraction now validates and accepts `crawl4ai.escape_backslash`; current owned defaults continue preserving literal source backslashes before Markdown-sensitive characters, while explicit false disables the extra owned backslash protection for normal text. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19fq: Surface CDP `Runtime.evaluate` exception details

Acceptance criteria:

- Inspect `agent-browser` source for `Runtime.evaluate` exception handling before changing owned CDP evaluation.
- Preserve current owned rendered-page and current-tab public surfaces while improving failure reporting.
- When a CDP `Runtime.evaluate` response contains `exceptionDetails`, return a stable extraction failure that includes the exception description or text instead of silently treating the value as empty.
- Keep direct-page and browser-session CDP paths compatible.
- Add deterministic mock CDP coverage for an evaluation exception during rendered-page capture.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D340. Source inspection found agent-browser returns an evaluation error when `Runtime.evaluate` includes `exception_details`, preferring `exception.description` over the fallback text. Owned `evaluate_string` now preserves existing empty-string behavior for non-exception missing values while returning a stable `ExtractionFailed` with the CDP exception message when `exceptionDetails` is present. The helper is shared by launched-page rendering and current-tab attachment, and deterministic mock CDP coverage exercises the attached rendered-page capture path. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage and the standard gate passed.

### ✅ Task I19fr: Surface CDP readiness evaluation exceptions

Acceptance criteria:

- Reuse the D340 agent-browser source inspection for `Runtime.evaluate` exception handling.
- Preserve current owned rendered-page public behavior while improving failure reporting for readiness and preprocessing evaluations.
- When selector waits, image waits, scan-full-page, rendered overlay cleanup, or iframe processing receive CDP `exceptionDetails`, return the same stable owned evaluation failure instead of treating the result as false, empty, or successful.
- Keep warning-based continuation behavior for optional preprocessing steps that already intentionally continue after helper errors.
- Add deterministic mock CDP coverage for a readiness `Runtime.evaluate` exception.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D341. D340's source inspection boundary now applies to selector waits, image waits, full-page scanning, rendered overlay cleanup, and iframe processing. Owned CDP helpers fail immediately on `exceptionDetails` with the same stable `Runtime.evaluate` extraction failure, while optional scan/overlay/iframe call sites keep their existing warning-and-continue behavior. Deterministic mock CDP coverage confirms selector readiness reports an evaluation exception instead of timing out or treating the result as false. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage and the standard gate passed.

### ✅ Task I19fs: Surface CDP storage evaluation exceptions

Acceptance criteria:

- Inspect agent-browser source for storage `Runtime.evaluate` exception handling before changing owned storage load/export.
- Preserve current owned session load/export surfaces while improving failure reporting for localStorage/sessionStorage evaluation.
- When owned storage load or export receives CDP `exceptionDetails`, return a stable extraction failure that includes browser exception text instead of treating storage load as successful or exported storage as absent.
- Keep direct-page and browser-session CDP paths compatible.
- Add deterministic mock CDP coverage for a storage `Runtime.evaluate` exception.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D342. Source inspection found agent-browser's storage helper returns a storage error when `Runtime.evaluate` includes `exception_details`; owned CDP storage load/export now uses the shared Runtime.evaluate exception helper instead of treating storage writes as successful or exported origins as absent. Deterministic mock CDP coverage confirms localStorage load reports the browser-provided storage exception. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage and the standard gate passed.

### ✅ Task I19ft: Surface storage-export blank navigation failures

Acceptance criteria:

- Inspect agent-browser source for temp-target storage export navigation before changing owned storage export navigation.
- Preserve current owned storage export behavior while improving failure reporting for the blank-response navigation used to read origin storage.
- When `Page.navigate` for blank-response storage export returns CDP `errorText`, return a stable extraction failure that includes the browser error text instead of waiting for a load event that will not arrive.
- Keep normal `Fetch.requestPaused` blank fulfillment and load-event success behavior unchanged.
- Add deterministic mock CDP coverage for a blank-response navigation `errorText`.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D343. Source inspection found agent-browser's storage export path uses temporary-target blank-response navigation before collecting origin storage. Owned `navigate_with_blank_response` now reuses the same `Page.navigate` response parser as normal navigation, so CDP command errors and `result.errorText` become stable extraction failures while `Fetch.requestPaused` blank fulfillment and load-event success remain unchanged. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage and the standard gate passed.

### ✅ Task I19fu: Lock Crawl4AI image readiness timeout boundary

Acceptance criteria:

- Inspect Crawl4AI source for `wait_for_images` and `wait_for_timeout` interaction before changing owned readiness behavior.
- Preserve Crawl4AI's boundary that `wait_for_timeout` applies to explicit `wait_for` conditions, while image readiness uses its own short bounded wait.
- Add deterministic mock CDP coverage that an explicit short `crawl4ai.wait_for_timeout` does not truncate the image-readiness wait.
- Keep current rendered-page warning behavior when images do not complete within the image-readiness timeout.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D344. Source inspection found Crawl4AI documents `wait_for_timeout` as specific to explicit `wait_for` conditions and applies it only around `smart_wait`; `wait_for_images` is handled separately through a fixed 1000 ms image-completion wait with warning-only continuation on timeout. Owned rendering already matched that boundary: selector waits use `wait_for_timeout.unwrap_or(page_timeout)`, while image readiness keeps its independent one-second deadline. Deterministic mock CDP coverage now proves an explicit one-millisecond `crawl4ai.wait_for_timeout` does not truncate image readiness; the renderer waits for a later successful image-completion response and emits no warning. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage and the standard gate passed.

### ✅ Task I19fv: Support Crawl4AI `google_doc` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` and `DefaultMarkdownGenerator` handling for `google_doc` before changing owned markdown output.
- Add owned backend option support for `crawl4ai.google_doc`.
- Preserve existing/default owned markdown output unless the option is supplied explicitly.
- When `crawl4ai.google_doc=true`, render Google Docs-style inline CSS emphasis for bold, italic, and fixed-width spans in a source-compatible way.
- Keep styled line-through behavior compatible with the existing `crawl4ai.hide_strikethrough` boundary.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D345. Source inspection found `DefaultMarkdownGenerator(options=...)` forwards `google_doc` into `CustomHTML2Text.update_params`; `CustomHTML2Text` defaults `google_doc` to false, and when enabled maps Google Docs-style inline CSS emphasis to strong, emphasis, and inline-code markers while `hide_strikethrough` suppresses line-through styled text. Owned extraction now validates and accepts `crawl4ai.google_doc`; defaults remain unchanged because style attributes are still pruned unless the option is supplied, while `google_doc=true` preserves style through cleanup so markdown rendering can apply bold, italic, and fixed-width span markers. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown, command-option validation, helper syntax checks, and the standard gate passed.

### ✅ Task I19fw: Lock agent-browser-style binary CDP response handling

Acceptance criteria:

- Inspect agent-browser CDP client WebSocket response handling before changing owned browser/CDP behavior.
- Preserve the owned CDP transport surface while proving UTF-8 binary CDP response frames are accepted.
- Add deterministic mock WebSocket coverage for a binary CDP command response.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D346. Source inspection found agent-browser accepts both text and UTF-8 binary WebSocket CDP response frames before parsing JSON, skipping invalid binary frames. Owned CDP transport already accepted UTF-8 binary frames, and deterministic mock WebSocket coverage now proves the normal `CdpClient::send` path accepts a binary `Browser.getVersion` response. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage passed; the standard gate passed before commit.

### ✅ Task I19fx: Split dense current decision index routing

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the planned-task source of truth; do not compact or summarize completed tasks out of it.
- Keep `workpads/research/archive/knowledge/current-decision-index.md` as a compact router for active decision lookup.
- Move dense historical per-decision routing rows into smaller referenced index files.
- Preserve links to recent decisions and current open migration gaps.
- Validate Markdown/link readability and record the support-file split in `knowledge.md`.

Status note:

- Completed with D347. `current-decision-index.md` now stays as a compact router, dense D168-D346 per-decision routing moved into `archive/knowledge/decision-index/`, and current open migration gaps remain visible in the top-level router. `workpads/research/tasks.md` was not compacted.

### ✅ Task I19fy: Support Crawl4AI `google_list_indent` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` handling for `google_list_indent` and Google Docs list style before changing owned markdown output.
- Add owned backend option support for `crawl4ai.google_list_indent`.
- Preserve existing/default owned list markdown unless `crawl4ai.google_doc=true` is supplied.
- When `crawl4ai.google_doc=true`, infer list nesting from `li` `margin-left` divided by `crawl4ai.google_list_indent`, and infer ordered/unordered list markers from list `list-style-type`.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D348. Source inspection found Crawl4AI defaults `GOOGLE_LIST_INDENT` to 36, uses `google_list_style` to infer ordered versus unordered Google Docs lists from `list-style-type`, and indents each `li` by `margin-left // google_list_indent`. Owned extraction now validates and accepts `crawl4ai.google_list_indent`; default list rendering is unchanged unless `crawl4ai.google_doc=true`, and Google Docs list rendering now uses preserved style attributes to infer ordered markers and margin-derived nesting. The Crawl4AI command helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed; the standard gate passed before commit.

### ✅ Task I19fz: Skip invalid binary CDP response frames

Acceptance criteria:

- Inspect agent-browser CDP client WebSocket response handling before changing owned CDP transport behavior.
- Preserve owned handling for valid text and UTF-8 binary CDP JSON frames.
- Skip invalid UTF-8 binary WebSocket frames instead of failing the active CDP command, matching agent-browser's reader behavior.
- Add deterministic mock WebSocket coverage where an invalid binary frame arrives before the valid command response.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D349. Source inspection found agent-browser accepts text and UTF-8 binary CDP WebSocket messages, but skips invalid UTF-8 binary frames and continues waiting for the command response. Owned CDP transport now preserves valid text and UTF-8 binary handling while ignoring invalid binary frames. Deterministic mock WebSocket coverage proves an invalid binary frame before a valid `Browser.getVersion` response does not fail the active command. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage passed; the standard gate passed before commit.

### ✅ Task I19ga: Skip malformed non-CDP WebSocket frames

Acceptance criteria:

- Inspect agent-browser CDP client WebSocket response parsing before changing owned CDP malformed-frame behavior.
- Preserve owned handling for valid text and UTF-8 binary CDP JSON frames.
- Skip malformed text and UTF-8 binary JSON frames instead of failing the active CDP command, matching agent-browser's reader behavior.
- Keep CDP command error objects as stable extraction failures.
- Add deterministic mock WebSocket coverage where malformed frames arrive before the valid command response.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D350. Source inspection found agent-browser's CDP reader skips invalid UTF-8 binary frames, malformed JSON text/binary frames, and typed parse failures, then continues waiting for pending command responses. Owned CDP transport now skips malformed text and UTF-8 binary JSON frames while preserving valid text/binary response handling and stable failures for valid CDP command error objects. Deterministic mock WebSocket coverage proves malformed text and binary frames before a valid `Browser.getVersion` response do not fail the active command. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage passed; the standard gate passed before commit.

### ✅ Task I19gb: Remove owned CDP WebSocket size limits

Acceptance criteria:

- Inspect agent-browser CDP WebSocket connection config before changing owned CDP transport.
- Configure owned CDP WebSocket connections with unlimited incoming message and frame size, matching agent-browser.
- Preserve current CDP connect error wording and post-connect socket timeout behavior.
- Add deterministic coverage for the owned CDP WebSocket config boundary.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D351. Source inspection found agent-browser builds a `WebSocketConfig` with `max_message_size: None` and `max_frame_size: None` before `connect_async_with_config`, while local tungstenite defaults bound incoming messages at 64 MiB and frames at 16 MiB. Owned CDP transport now uses `connect_with_config` with the same unlimited incoming message/frame size boundary while preserving existing connect error wording and socket timeouts. Deterministic unit coverage proves the owned config disables both limits. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage passed; the standard gate passed before commit.

### ✅ Task I19gc: Send CDP WebSocket keepalive pings

Acceptance criteria:

- Inspect agent-browser CDP keepalive behavior before changing owned CDP transport.
- Send periodic WebSocket Ping frames while owned CDP commands are waiting for responses.
- Keep incoming Ping/Pong handling unchanged and preserve current CDP timeout/error behavior.
- Add deterministic mock WebSocket coverage that a delayed CDP response can be gated on the owned client sending a keepalive Ping.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D352. Source inspection found agent-browser sends empty WebSocket Ping frames every 30 seconds to keep CDP connections alive through intermediate proxies, and comments that WebSocket Ping keepalive is the primary liveness mechanism. Owned CDP transport now sends empty Ping frames after the same interval while waiting for command responses, preserving incoming Ping/Pong handling and timeout/error behavior. Deterministic mock WebSocket coverage withholds a valid `Browser.getVersion` response until the client keepalive Ping arrives. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage passed; the standard gate passed before commit.

### ✅ Task I19gd: Auto-handle blocking alert dialogs in owned CDP

Acceptance criteria:

- Inspect agent-browser dialog auto-handling before changing owned CDP event handling.
- Auto-accept `alert` and `beforeunload` JavaScript dialogs during owned CDP command waits, matching agent-browser's non-blocking default.
- Leave `confirm` and `prompt` dialogs unhandled by the automatic path.
- Preserve existing CDP command response, timeout, and malformed-frame behavior.
- Add deterministic mock WebSocket coverage proving an alert event triggers `Page.handleJavaScriptDialog` and does not block the active command response.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D353. Source inspection found agent-browser's default dialog handler auto-accepts `alert` and `beforeunload` dialogs so they do not block the agent, while `confirm` and `prompt` remain explicit-agent decisions. Owned CDP transport now consumes `Page.javascriptDialogOpening` events for `alert` and `beforeunload`, sends `Page.handleJavaScriptDialog` with `accept: true` and the event session when present, and keeps active command waits running until their real response arrives. Deterministic mock WebSocket coverage verifies alert auto-acceptance and prompt non-acceptance. `workpads/research/tasks.md` was not compacted. Focused browser CDP coverage passed; the standard gate passed before commit.

### ✅ Task I19ge: Split CDP transport tests out of setup tests

Acceptance criteria:

- Preserve `workpads/research/tasks.md` as the full task backlog; do not compact it.
- Move transport-oriented CDP client tests out of `src/browser_cdp/tests/chrome/cdp_client/setup.rs` into a smaller behavior-owned module.
- Keep setup/attach/domain-enabling tests in `setup.rs`.
- Make no intentional behavior changes.
- Record the split boundary in `knowledge.md`.
- Verify focused CDP tests plus the standard check set.

Status note:

- Completed with D354. Transport-oriented CDP client tests now live in `src/browser_cdp/tests/chrome/cdp_client/transport.rs`, covering WebSocket config, keepalive, dialog auto-handling, binary frames, invalid binary frames, and malformed frames. `setup.rs` now stays focused on direct page connections, existing-page attachment setup, and page-domain enablement, dropping from 400 lines to 144 lines. No production behavior changed. `workpads/research/tasks.md` was not compacted. Focused CDP coverage passed; the standard gate passed before commit.

### ✅ Task I19gf: Let dense unlabeled containers win main-content extraction

Acceptance criteria:

- Re-inspect Crawl4AI pruning/content-filter source before changing owned main-content scoring.
- Let generic unlabeled `div`/`section` candidates win when source-backed density, link-density, tag-weight, and text-length signals are strong enough.
- Keep negative class/id labels and page-chrome ancestor exclusions stronger than density scoring.
- Do not introduce site-specific labels or built-in site behavior.
- Add deterministic mock-site coverage for an unlabeled dense article container beating page chrome.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused main-content coverage plus the standard check set.

Status note:

- Completed with D355. Source inspection found Crawl4AI's `PruningContentFilter` scores generic structural tags with text density, non-link density, tag weight, class/id weight, and text length rather than requiring positive content labels for every `div`/`section` candidate. Owned main-content scoring now accepts unlabeled `div`/`section` candidates with strong Crawl4AI-like pruning scores, still rejects candidates with negative class/id labels or page-chrome ancestors, and avoids promoting broad unlabeled layout wrappers when they contain a more specific positive content descendant. Deterministic mock-site coverage proves an unlabeled dense prose container beats nav/footer fallback. `workpads/research/tasks.md` was not compacted. Focused main-content coverage passed; the standard gate passed before commit.

### ✅ Task I19gg: Remove page chrome when automatic main-content falls back to body

Acceptance criteria:

- Re-inspect Crawl4AI pruning cleanup source before changing owned body-fallback behavior.
- When automatic main-content selection falls back to `body` or document root, remove generic page-chrome tags before markdown/text extraction.
- Do not change explicit selector extraction, explicit HTML output, or already-selected `main`/`article`/container candidates.
- Keep the cleanup generic and source-backed; do not introduce site-specific rules.
- Add deterministic mock-site coverage for a bare body page where nav/footer are removed but the title and paragraph remain.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify focused main-content coverage plus the standard check set.

Status note:

- Completed with D356. Source inspection found Crawl4AI's `PruningContentFilter` removes generic unwanted tags, including `nav`, `footer`, `header`, `aside`, `form`, `iframe`, and `noscript`, before pruning remaining content blocks. Owned extraction now removes those page-chrome tags only when automatic main-content selection falls back to `body` or document root, leaving explicit selectors, explicit HTML output, and already-selected `main`/`article`/container candidates unchanged. Deterministic mock-site coverage proves a bare body page keeps the title and useful paragraph while dropping header/nav/footer. `workpads/research/tasks.md` was not compacted. Focused main-content coverage passed; the standard gate passed before commit.

### ✅ Task I19gh: Split oversized archived knowledge bundle without compacting tasks

Acceptance criteria:

- Keep `workpads/research/tasks.md` as the full executable backlog; do not compact existing task content.
- Split the oversized archived D21-D34 knowledge bundle into smaller referenced section files.
- Keep the original archive path as a compact router so existing links continue to work.
- Update current support-file routing so the split is discoverable.
- Record the support-file boundary in `knowledge.md`.
- Verify route targets, size reduction, and diff hygiene.

Status note:

- Completed with D357. `workpads/research/tasks.md` was not compacted. The oversized historical D21-D34 knowledge bundle is now a compact router at its original path, with detailed content split into three referenced section files for D21-D24, D25-D28, and D29-D34. Concatenating the three new section files matches the previous archive content byte-for-byte, the original route remains discoverable through the current decision index, and size checks plus diff hygiene passed.

### ✅ Task I19gi: Support Crawl4AI `preserve_tags` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` preserve-tag handling before changing owned markdown output.
- Add owned backend option support for `crawl4ai.preserve_tags` as a comma-separated list of plain tag names.
- Preserve configured tags as raw HTML blocks in markdown output instead of recursively converting their children.
- Keep default markdown behavior unchanged when the option is absent.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D358. Source inspection found Crawl4AI's `CustomHTML2Text.update_params` accepts `preserve_tags` and that `handle_tag` collects configured tag subtrees, including nested tags and text, before emitting the preserved HTML block instead of normal markdown conversion. Owned extraction now validates and accepts `crawl4ai.preserve_tags` as a comma-separated tag list, preserves configured tag subtrees as raw HTML blocks in markdown output, and leaves default markdown behavior unchanged. The Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown and command-option validation passed.

### ✅ Task I19gj: Support Crawl4AI `handle_code_in_pre` markdown option

Acceptance criteria:

- Inspect Crawl4AI `CustomHTML2Text` handling for `handle_code_in_pre` before changing owned markdown output.
- Add owned backend option support for `crawl4ai.handle_code_in_pre`.
- Preserve default fenced-code markdown behavior when the option is absent or false.
- When true, preserve Crawl4AI-style backtick markers around `<code>` descendants inside `<pre>` blocks.
- Keep the Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text aligned.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned markdown and command-option coverage plus the standard check set.

Status note:

- Completed with D359. Source inspection found Crawl4AI's `CustomHTML2Text.__init__` defaults `handle_code_in_pre` to false, `update_params` accepts the option, and `handle_tag` ignores `<code>` tags inside `<pre>` by default while emitting backticks around those tags when enabled. Owned extraction now validates and accepts `crawl4ai.handle_code_in_pre`, leaves default fenced-code output unchanged, and wraps `<code>` descendants inside `<pre>` with Crawl4AI-style backticks when enabled. The Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown/command-option validation and the standard check set passed.

### ✅ Task I19gk: Refresh ignored local Chrome smoke expectations

Acceptance criteria:

- Run the non-visible owned Chrome import smoke when local Chrome is available.
- Run ignored local Chrome mock-site browser smokes and identify any stale expectations.
- Preserve the owned text block-boundary contract that deterministic tests already cover.
- Update ignored smoke assertions only where they are stale relative to the current owned extractor contract.
- Do not compact `workpads/research/tasks.md`.
- Record the local Chrome evidence in `knowledge.md`.
- Verify the updated ignored smokes plus the standard check set.

Status note:

- Completed with D360. The non-visible owned Chrome import smoke passed locally. An ignored local Chrome mock-site run initially found four stale expectations for rendered text block boundaries; the owned extractor already returns newline-separated heading/paragraph blocks, so only those smoke assertions were updated. No extractor/browser behavior changed. After refresh, all ignored mock-site browser smokes passed. `workpads/research/tasks.md` was not compacted.

### ✅ Task I19gl: Cover Crawl4AI metadata title fallback parity

Acceptance criteria:

- Inspect Crawl4AI's active metadata extraction path before adding coverage.
- Add deterministic owned extractor coverage for the missing/empty `<title>` fallback to Open Graph and Twitter title metadata.
- Preserve existing metadata behavior and page content output.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned extractor coverage plus the standard check set.

Status note:

- Completed with D361. Source inspection found Crawl4AI's active lxml scraping path calls `extract_metadata_using_lxml`, which tries `<title>`, falls back to `og:title`, then falls back to `twitter:title` for page title metadata. Owned extractor coverage now locks the missing-title fallback to Open Graph title while preserving prefixed Open Graph/Twitter metadata and unchanged page content output. No runtime behavior changed. `workpads/research/tasks.md` was not compacted.

### ✅ Task I19gm: Cover Crawl4AI Twitter title fallback parity

Acceptance criteria:

- Inspect Crawl4AI's active metadata title fallback order before adding coverage.
- Add deterministic owned extractor coverage for the missing `<title>` and missing `og:title` fallback to `twitter:title`.
- Preserve existing metadata behavior and page content output.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned extractor coverage plus the standard check set.

Status note:

- Completed with D362. Source inspection confirmed Crawl4AI's active lxml metadata path falls back from `<title>` to `og:title`, then to `twitter:title`. Owned extractor coverage now locks the Twitter-only fallback path, preserves prefixed Twitter metadata, verifies no Open Graph title key is invented, and leaves page content output unchanged. No runtime behavior changed. `workpads/research/tasks.md` was not compacted.

### ✅ Task I19gn: Accept Crawl4AI `ignore_anchors` markdown alias

Acceptance criteria:

- Inspect Crawl4AI/html2text source for anchor suppression terminology before changing owned option parsing.
- Accept `crawl4ai.ignore_anchors` as a source-backed compatibility alias for the owned `crawl4ai.ignore_links` behavior.
- Preserve existing `crawl4ai.ignore_links` behavior, output, docs, and unsupported-option diagnostics.
- Verify with focused owned markdown/option validation coverage plus the standard check set.

Status note:

- Completed with D363. Source inspection found Crawl4AI/html2text names the default anchor-suppression config `IGNORE_ANCHORS`, while runtime markdown rendering consumes `ignore_links` and the CLI exposes `--ignore-links` from that config default. Owned extraction now accepts `crawl4ai.ignore_anchors` as a compatibility alias for existing `crawl4ai.ignore_links` behavior, and the Crawl4AI compatibility helper forwards the alias as runtime `ignore_links`. Mock backend validation, unsupported-option diagnostics, README, and OpenCode option text are aligned. `workpads/research/tasks.md` was not compacted. Focused owned markdown/option validation and Python helper syntax checks passed.

### ✅ Task I19go: Audit current large-file ergonomics without compacting tasks

Acceptance criteria:

- Audit the current tracked file size profile using git-tracked files rather than broad filesystem scans.
- Confirm whether any source, test, or support workpad files still need immediate splitting after the recent decomposition work.
- Preserve `workpads/research/tasks.md` as the full executable backlog; do not compact it.
- Record the result in compact support routing/decision files so future agents do not re-open task compaction.
- Verify with line-count checks and diff hygiene.

Status note:

- Completed with D364. Git-tracked line-count audit shows no source, test, or support-workpad file currently needs another split: aside from intentional exceptions `workpads/research/tasks.md` and `Cargo.lock`, the largest tracked source/test file is 382 lines, while top-level support routers remain compact (`knowledge.md` 66 lines, `references.md` 75 lines). `workpads/research/tasks.md` remains the full executable backlog and was not compacted. Validation passed with tracked line-count checks and `git diff --check`.

### ✅ Task I19gp: Export allowed frame-tree storage origins

Acceptance criteria:

- Inspect agent-browser state-save source before changing owned browser state export.
- Preserve explicit allow-domain scope; frame-tree origins must not broaden saved auth state beyond user-authorized domains.
- Add owned CDP state export support for storage origins discovered through `Page.getFrameTree`.
- Keep existing explicit origin candidate probing, cookie export, storage load, and blank-response navigation behavior unchanged.
- Add deterministic mock CDP coverage for an allowed child-frame storage origin and a disallowed frame origin.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused browser CDP coverage plus the standard check set.

Status note:

- Completed with D365. Source inspection found agent-browser merges current frame-tree origins into storage-state saving, then collects storage for remaining origins via a temporary target. Owned CDP state export now asks `Page.getFrameTree`, adds only frame origins whose hosts remain within the explicit allow-domain scope, and keeps disallowed frame origins and non-origin URLs out of saved state. Existing cookie export, explicit origin candidates, storage load, and blank-response navigation behavior remain unchanged. `workpads/research/tasks.md` was not compacted. Focused CDP tests, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gq: Support Crawl4AI `excluded_selector` cleanup option

Acceptance criteria:

- Inspect Crawl4AI source before changing owned extraction cleanup.
- Accept `crawl4ai.excluded_selector` as a backend option that removes matching CSS-selected elements before owned content extraction.
- Preserve the existing top-level `--exclude-selector` behavior and invalid-selector tolerance.
- Keep cleanup ordering aligned with Crawl4AI's form/tag/selector removal boundary where practical in the owned pipeline.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned extractor option/cleanup coverage plus the standard check set.

Status note:

- Completed with D366. Source inspection found Crawl4AI's `CrawlerRunConfig.excluded_selector` stores a CSS selector string, removes matching elements after form/tag cleanup in the scraping strategy, and tolerates selector errors. Owned extraction now accepts repeated non-empty `crawl4ai.excluded_selector` backend options, removes matching elements through the existing invalid-selector-tolerant cleanup helper, preserves top-level `--exclude-selector`, and keeps command/mock validation, README, and OpenCode tool text aligned. `workpads/research/tasks.md` was not compacted. Focused owned selector coverage, command-backend option validation, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gr: Support Crawl4AI `remove_consent_popups` cleanup option

Acceptance criteria:

- Inspect Crawl4AI source before changing owned consent-popup cleanup.
- Accept `crawl4ai.remove_consent_popups` as a backend option that removes generic cookie/GDPR consent elements before owned content extraction.
- Preserve `aget`'s generic fetcher boundary: do not add site-specific paywall/login handling and do not click consent buttons or set cookies.
- Keep `crawl4ai.remove_overlay_elements=false` separate from consent-popup cleanup so callers can remove cookie consent noise without removing every modal/dialog.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned extractor option/cleanup coverage plus the standard check set.

Status note:

- Completed with D367. Source inspection found Crawl4AI's `CrawlerRunConfig.remove_consent_popups` is default false, triggers browser-backed cleanup, and runs consent-popup cleanup before generic overlay cleanup. Owned extraction now accepts `crawl4ai.remove_consent_popups`, removes generic cookie/GDPR consent elements before overlay cleanup, keeps `crawl4ai.remove_overlay_elements=false` separate, and does not click consent controls, set cookies, or add site-specific paywall/login handling. Command/mock validation, README, OpenCode tool text, and compact knowledge routing were updated. `workpads/research/tasks.md` was not compacted. Focused owned extractor cleanup coverage, command-backend option validation, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gs: Support Crawl4AI `css_selector` extraction option

Acceptance criteria:

- Inspect Crawl4AI source before changing owned selector extraction.
- Accept `crawl4ai.css_selector` as a backend option that selects matching content before owned output rendering.
- Preserve the existing top-level `--selector`/API selector behavior and precedence.
- Preserve Crawl4AI/static-scraper-style fallback behavior for invalid selectors and selector misses.
- Keep `crawl4ai.target_elements` scoped under the selected content when both options are supplied.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned selector coverage, command-backend option validation, and the standard check set.

Status note:

- Completed with D368. Source inspection found Crawl4AI's `CrawlerRunConfig.css_selector` extracts a specific page portion, with the static scraper selecting matches into a wrapper and falling back to body on selector misses or selector errors, while the rendered path serializes `querySelectorAll` matches before scraping. Owned extraction now accepts `crawl4ai.css_selector`, keeps top-level `--selector`/API selector precedence, falls back to the document root for misses/invalid selectors, and scopes `crawl4ai.target_elements` under selected content. The command mock, README, OpenCode tool text, and Crawl4AI compatibility helper were aligned; the helper allowlist also now includes the recently added `excluded_selector` and `remove_consent_popups` options. `workpads/research/tasks.md` was not compacted. Focused owned selector coverage, command-backend option validation, Python helper syntax checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gt: Align Crawl4AI cache-mode compatibility options

Acceptance criteria:

- Inspect Crawl4AI source before changing cache-option validation.
- Accept bypass/no-cache cache options that preserve the owned extractor's current uncached behavior.
- Reject cache read/write modes in the owned extractor until `aget` has an owned cache store.
- Keep the optional Crawl4AI command compatibility helper able to pass valid `CacheMode` values to real Crawl4AI.
- Preserve output metadata recording of backend options.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused option validation, command-backend forwarding, helper syntax checks, and the standard check set.

Status note:

- Completed with D369. Source inspection found Crawl4AI's `CrawlerRunConfig.cache_mode` defaults to `CacheMode.BYPASS`, `CacheMode` supports enabled/disabled/read_only/write_only/bypass, and cache read/write behavior is routed through `CacheContext` only when a cache store exists. Owned extraction now accepts `crawl4ai.cache` and `crawl4ai.cache_mode` only for bypass/disabled no-cache modes, rejects enabled/read_only/write_only until `aget` has an owned extraction cache store, and keeps backend-option output metadata recording unchanged. The optional Crawl4AI compatibility helper validates cache modes, maps the existing `crawl4ai.cache` alias to `cache_mode`, and converts strings into the installed `CacheMode` enum before building `CrawlerRunConfig`. Command mock validation, README, OpenCode tool text, and compact knowledge routing were updated. `workpads/research/tasks.md` was not compacted. Focused owned option validation, command-backend option validation, Python helper syntax and conversion checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gu: Support Crawl4AI `keep_attrs` cleanup option

Acceptance criteria:

- Inspect Crawl4AI source before changing owned attribute cleanup.
- Accept `crawl4ai.keep_attrs` as a backend option that preserves the named attributes during owned cleaned-HTML serialization.
- Preserve existing selector behavior: selectors still run before attribute pruning.
- Keep `crawl4ai.keep_data_attributes` behavior unchanged.
- Keep the optional Crawl4AI command compatibility helper able to pass `keep_attrs` to real Crawl4AI when the installed version accepts it.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned cleanup coverage, command-backend option validation, helper syntax checks, and the standard check set.

Status note:

- Completed with D370. Source inspection found `CrawlerRunConfig.keep_attrs` is documented and serialized as a list of attributes to keep, while the current static scraper snapshot only threads the fixed important-attribute allowlist plus `keep_data_attributes` into `remove_unwanted_attributes_fast`. Owned extraction now accepts `crawl4ai.keep_attrs`, preserves explicitly named attributes during owned cleaned-HTML serialization, keeps selectors running before attribute pruning, and leaves `crawl4ai.keep_data_attributes` unchanged. The optional Crawl4AI compatibility helper, command mock validation, README, OpenCode tool text, and compact knowledge routing were updated. `workpads/research/tasks.md` was not compacted. Focused owned cleanup coverage, command-backend option validation, Python helper syntax and parsing checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gv: Support Crawl4AI `prettiify` cleaned-HTML option

Acceptance criteria:

- Inspect Crawl4AI source before changing owned HTML output formatting.
- Accept `crawl4ai.prettiify` as a backend option for owned extraction.
- Apply formatting only to owned HTML output, matching Crawl4AI's `cleaned_html` post-processing boundary.
- Keep markdown, text, and JSON output unchanged.
- Keep the optional Crawl4AI command compatibility helper able to pass `prettiify` to real Crawl4AI.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused owned HTML coverage, command-backend option validation, helper syntax checks, and the standard check set.

Status note:

- Completed with D371. Source inspection found `CrawlerRunConfig.prettiify` defaults to `false`, and `AsyncWebCrawler` applies `fast_format_html(cleaned_html)` after scraping/extraction but before returning `CrawlResult.cleaned_html`. Owned extraction now accepts `crawl4ai.prettiify` and applies Crawl4AI-style two-space fast formatting only to owned HTML output; markdown, text, and JSON output stay unchanged. The optional Crawl4AI compatibility helper, command mock validation, README, OpenCode tool text, and compact knowledge routing were updated. `workpads/research/tasks.md` was not compacted. Focused owned HTML coverage, command-backend option validation, Python helper syntax and parsing checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gw: Support Crawl4AI `process_in_browser` local-content routing

Acceptance criteria:

- Inspect Crawl4AI source before changing owned raw/file input routing.
- Accept `crawl4ai.process_in_browser` as a backend option for owned extraction.
- Route raw/file inputs through owned CDP rendering when `process_in_browser` or safe browser-only local options require the browser pipeline.
- Preserve current fast static raw/file behavior when browser routing is not requested.
- Preserve the authenticated safety rule: do not enable arbitrary user-supplied JavaScript execution.
- Keep the optional Crawl4AI command compatibility helper able to pass `process_in_browser` to real Crawl4AI.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused local-routing/option coverage, helper syntax checks, and the standard check set.

Status note:

- Completed with D372. Source inspection found Crawl4AI's `CrawlerRunConfig.process_in_browser` defaults to `false`, and `AsyncCrawlerStrategy.crawl` routes `raw:`, `raw://`, and `file://` inputs through the browser pipeline when `process_in_browser` or other browser-only options are present; otherwise local content returns through the fast HTML path. Owned extraction now accepts `crawl4ai.process_in_browser`, materializes raw HTML into a temporary run-local file URL for CDP rendering when browser routing is requested, preserves the original raw/file final URL in extraction output, and also routes safe browser-only local options such as CSS waits, image readiness, iframe processing, and full-page scanning through the same path. Arbitrary user-supplied JavaScript remains unsupported. The optional Crawl4AI compatibility helper, command mock validation, README, OpenCode tool text, compact knowledge routing, and ignored local Chrome raw-browser smoke coverage were updated. `workpads/research/tasks.md` was not compacted. Focused local-routing unit coverage, command-backend option validation, Python helper syntax and parsing checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed; the ignored local Chrome smoke was added but not run in the deterministic gate.

### ✅ Task I19gx: Support Crawl4AI explicit `user_agent` option

Acceptance criteria:

- Inspect Crawl4AI source before changing owned request identity behavior.
- Accept `crawl4ai.user_agent` as an owned backend option for explicit user-agent strings.
- Apply the configured user agent to owned static HTTP requests.
- Apply the configured user agent to owned CDP-rendered page requests before navigation.
- Do not implement random user-agent generation, `user_agent_mode`, or arbitrary header injection in this slice.
- Keep the optional Crawl4AI command compatibility helper able to pass `user_agent` to real Crawl4AI.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused static/CDP option coverage, helper syntax checks, and the standard check set.

Status note:

- Completed with D373. Source inspection found Crawl4AI `CrawlerRunConfig.user_agent` defaults to `None`, is applied before crawl execution, and is pushed into Playwright context/request user-agent behavior for non-persistent browser contexts. Owned extraction now accepts `crawl4ai.user_agent`, applies it to owned static HTTP requests, and sends `Network.setUserAgentOverride` before owned CDP state-loading or target navigation. This slice intentionally does not implement random user-agent generation, `user_agent_mode`, client hints synthesis, or arbitrary custom headers. The optional Crawl4AI compatibility helper, command mock validation, README, OpenCode tool text, compact knowledge routing, static request assertions, and CDP protocol coverage were updated. `workpads/research/tasks.md` was not compacted. Focused static/CDP/command-option coverage, Python helper syntax and parsing checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gy: Split AgetExtractor options/waits parity helper

Acceptance criteria:

- Split oversized `tests/mock_site_cli/aget_extractor/options_waits.rs` into behavior-focused child modules.
- Preserve the public integration test route and all existing assertions.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused AgetExtractor parity coverage, line-count checks, and the standard check set.

Status note:

- Completed with D374. `tests/mock_site_cli/aget_extractor/options_waits.rs` is now a 16-line router, with positive option behavior in `options_waits/content.rs`, backend option validation and unsupported-option diagnostics in `options_waits/validation.rs`, and redirect/wait behavior in `options_waits/waits.rs`. Existing integration test routing and assertions were preserved; no production code or fixture behavior changed. `workpads/research/tasks.md` was not compacted. Focused AgetExtractor parity coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gz: Support Crawl4AI browser locale and timezone options

Acceptance criteria:

- Inspect Crawl4AI source before changing owned browser context identity behavior.
- Accept `crawl4ai.locale` and `crawl4ai.timezone_id` as owned backend options.
- Apply configured locale/timezone to owned CDP-rendered page requests before navigation.
- Route otherwise-static requests through the owned browser path when these browser-context options are supplied.
- Do not implement geolocation, proxy, arbitrary headers, or random user-agent behavior in this slice.
- Keep the optional Crawl4AI command compatibility helper able to pass `locale` and `timezone_id` to real Crawl4AI.
- Do not compact `workpads/research/tasks.md`.
- Record the source-backed boundary in `knowledge.md`.
- Verify with focused CDP/option coverage, helper syntax checks, and the standard check set.

Status note:

- Completed with D375. Crawl4AI source inspection showed `CrawlerRunConfig.locale` and `timezone_id` are explicit browser-context settings, and owned `AgetExtractor` now accepts `crawl4ai.locale` plus `crawl4ai.timezone_id`, routes requests requiring those context settings through owned CDP rendering, applies CDP locale/timezone emulation before page capture, and forwards both options through the compatibility Crawl4AI helper. This slice did not add geolocation, proxy, arbitrary-header, or random-user-agent behavior. `workpads/research/tasks.md` was not compacted. Focused CDP/context-routing/command-option/helper parse checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gza: Split owned page extraction module

Acceptance criteria:

- Preserve owned extraction behavior while reducing `src/extraction/owned/page.rs` into smaller behavior-owned modules.
- Keep static/rendered orchestration, cleaned HTML extraction, and local-input browser routing as separate readable boundaries.
- Preserve current public/internal module paths used by callers.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused owned page coverage, line-count checks, and the standard check set.

Status note:

- Completed with D376. `src/extraction/owned/page.rs` became `src/extraction/owned/page/mod.rs`, with cleaned HTML extraction and output shaping in `page/html.rs`, local raw/file browser-routing helpers plus their tests in `page/local_input.rs`, and existing readiness/rendered helpers kept as sibling modules. The orchestration route is now 123 lines, with child modules at 176, 157, 85, and 78 lines. Behavior and internal caller paths are preserved, and `workpads/research/tasks.md` was not compacted. Focused local-input tests, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzb: Split attached-page capture CDP tests

Acceptance criteria:

- Preserve current attached-page capture test behavior and assertions.
- Split `src/browser_cdp/tests/chrome/cdp_client/attached_page/capture.rs` into smaller behavior-focused child modules.
- Keep the existing parent module path available for test routing.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused attached-page CDP coverage, line-count checks, and the standard check set.

Status note:

- Completed with D377. `src/browser_cdp/tests/chrome/cdp_client/attached_page/capture.rs` is now a 3-line router at `capture/mod.rs`, with current URL/HTML capture coverage in `capture/basic.rs`, image-readiness timeout-boundary coverage in `capture/images.rs`, and runtime-evaluation exception coverage in `capture/runtime_error.rs`. Test names, mock CDP sequencing, and assertions are preserved. `workpads/research/tasks.md` was not compacted. Focused attached-page CDP coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzc: Split markdown normalization helpers

Acceptance criteria:

- Preserve current markdown rendering behavior while reducing `src/extraction/markdown/normalize.rs`.
- Keep document wrapping/single-line-break behavior, markdown escaping behavior, and URL/link target helpers in separate readable boundaries.
- Preserve existing `super::normalize::*` call paths used by markdown renderer modules.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused markdown coverage, line-count checks, and the standard check set.

Status note:

- Completed with D378. `src/extraction/markdown/normalize.rs` is now a module directory: `normalize/mod.rs` keeps document/inline normalization, Unicode-snob replacements, punctuation spacing, and trailing whitespace helpers; `normalize/wrap.rs` owns body-width wrapping and single-line-break application; `normalize/escape.rs` owns Markdown/link/table escaping; and `normalize/url.rs` owns absolute URL checks plus base-URL target resolution. Existing `super::normalize::*` imports are preserved, and `workpads/research/tasks.md` was not compacted. Focused markdown parity coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzd: Split markdown table renderer helpers

Acceptance criteria:

- Preserve current markdown table, ignored-table, bypass-table, and padded-table behavior.
- Split `src/extraction/markdown/table.rs` into smaller behavior-focused modules.
- Preserve existing `table::render_table` and `table::pad_markdown_tables` call paths.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused table/markdown coverage, line-count checks, and the standard check set.

Status note:

- Completed with D379. `src/extraction/markdown/table.rs` is now a module directory: `table/mod.rs` keeps table rendering orchestration and the existing `render_table`/`pad_markdown_tables` route; `table/rows.rs` owns caption/row/cell extraction plus GFM row formatting; `table/ignored.rs` owns ignored-table plain text extraction; `table/bypass.rs` owns bypass-table HTML-like rendering; and `table/pad.rs` owns padded-table post-processing. The split keeps all child files at 87 lines or less and preserves existing call paths. `workpads/research/tasks.md` was not compacted. Focused table/markdown coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gze: Split markdown writer helper state

Acceptance criteria:

- Preserve current markdown writer behavior, including text normalization, reference links, paragraph-scoped reference flushing, and abbreviation definitions.
- Split `src/extraction/markdown/writer.rs` into smaller behavior-focused modules.
- Preserve existing `writer::MarkdownWriter` call paths used by markdown rendering modules.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused markdown coverage, line-count checks, and the standard check set.

Status note:

- Completed with D380. `src/extraction/markdown/writer.rs` is now a module directory: `writer/mod.rs` keeps `MarkdownWriter` state, constructor/child cloning, URL resolution, tag-preservation checks, and image-alt fallback; `writer/text.rs` owns normalized text pushes, spacing-sensitive inline output, and blank-line normalization; and `writer/references.rs` owns abbreviation collection, reference-link numbering, paragraph-scoped reference flushing, final reference definitions, and abbreviation definition output. Existing `writer::MarkdownWriter` call paths are preserved. The split keeps the child helper modules at 103 and 54 lines and reduces the writer route to 207 lines. `workpads/research/tasks.md` was not compacted. Focused extractor/markdown coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzf: Split markdown render dispatch helpers

Acceptance criteria:

- Preserve current markdown rendering behavior, including node traversal, tag dispatch, preserved raw HTML, Google Docs inline/list style handling, and `only_text` inline-tag handling.
- Split `src/extraction/markdown/mod.rs` into smaller behavior-focused modules.
- Preserve the existing `element_to_markdown` entrypoint and internal renderer call paths.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused markdown coverage, line-count checks, and the standard check set.

Status note:

- Completed with D381. `src/extraction/markdown/mod.rs` now keeps the `element_to_markdown` entrypoint, writer setup, final reference/abbreviation flushing, normalization, wrapping, and table padding. `src/extraction/markdown/dispatch.rs` owns DOM node traversal, tag dispatch, preserved raw HTML output, Google Docs inline/list style handling, line-through hiding, and `only_text` inline-tag handling. Existing `element_to_markdown`, `render_node`, and `render_children` routes are preserved for markdown submodules. The split reduces the renderer entrypoint route to 114 lines, with dispatch at 239 lines. `workpads/research/tasks.md` was not compacted. Focused extractor/markdown coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzg: Split owned extractor option application helpers

Acceptance criteria:

- Preserve current owned backend option parsing and unsupported-option diagnostics.
- Split `src/extraction/owned/options/apply.rs` into smaller behavior-focused modules.
- Preserve the existing `apply_owned_extractor_option` call path used by owned option parsing.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused owned option validation coverage, line-count checks, and the standard check set.

Status note:

- Completed with D382. `src/extraction/owned/options/apply.rs` is now a module directory: `apply/mod.rs` keeps the `apply_owned_extractor_option` route and canonical supported-option list; `apply/document.rs` owns document selection, cleanup, metadata/request-identity, and cleaned-HTML options; `apply/markdown.rs` owns markdown, link, image, table, typography, wrapping, and social-link options; and `apply/browser.rs` owns browser/readiness, timeout, scrolling, iframe, shadow-DOM, and word-threshold options. Existing owned option parsing call paths and unsupported-option diagnostics are preserved. The split reduces the route to 34 lines, with child modules at 183, 89, and 61 lines. `workpads/research/tasks.md` was not compacted. Focused owned option validation coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzh: Split owned content extraction helpers

Acceptance criteria:

- Preserve current owned content extraction behavior, including selector fallback, target-element selection, main-content selection, line-through cleanup, single/multi-target output, and markdown base-URL handling.
- Split `src/extraction/owned/content.rs` into smaller behavior-focused modules.
- Preserve the existing `content::extract_owned_content` and `content::markdown_base_url` call paths.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused owned extractor/content coverage, line-count checks, and the standard check set.

Status note:

- Completed with D383. `src/extraction/owned/content.rs` moved to `src/extraction/owned/content/mod.rs`, which keeps `extract_owned_content`, cleanup ordering, and `markdown_base_url`. `content/selection.rs` owns root/target selection and selected-element lookup, `content/cleanup.rs` owns line-through removal, and `content/render.rs` owns single/multi-target HTML/markdown/text output construction. Existing owned content extraction and markdown base-URL call paths are preserved. The split reduces the route to 111 lines, with child helper modules at 36, 41, and 110 lines; `main_content.rs` remains 261 lines. `workpads/research/tasks.md` was not compacted. Focused owned extractor/content coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzi: Split inline markdown assertion helpers

Acceptance criteria:

- Preserve current inline markdown parity assertions and test behavior.
- Split `tests/mock_site_cli/aget_extractor/markdown/inline_blocks.rs` into smaller behavior-focused assertion helpers.
- Preserve the existing `inline_blocks::assert_inline_blocks` call path used by mock-site extractor coverage.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused markdown coverage, line-count checks, and the standard check set.

Status note:

- Completed with D384. `tests/mock_site_cli/aget_extractor/markdown/inline_blocks.rs` now keeps only the `assert_inline_blocks` route and dispatches to focused assertion helpers: `inline_blocks/semantics.rs` for semantic inline/block assertions, `inline_blocks/escape_unicode.rs` for escaping and Unicode options, `inline_blocks/google_preserve.rs` for Google Docs and preserved-tag behavior, and `inline_blocks/wrapping.rs` for wrapping and single-line-break behavior. Existing `inline_blocks::assert_inline_blocks` coverage is preserved. The route is now 19 lines, with helper files at 72, 69, 60, and 157 lines. `workpads/research/tasks.md` was not compacted. Focused markdown coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzj: Split markdown mock-site route fixtures

Acceptance criteria:

- Preserve current markdown mock-site routes and fixture HTML.
- Split `tests/mock_site_cli/aget_extractor_site/markdown.rs` into smaller behavior-focused route helpers.
- Preserve the existing `markdown::routes` call path used by the mock-site extractor parity site.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused markdown/mock-site coverage, line-count checks, and the standard check set.

Status note:

- Completed with D385. `tests/mock_site_cli/aget_extractor_site/markdown.rs` now keeps only the `routes` entrypoint and dispatches to focused fixture route helpers: `markdown/basic.rs` for basic markdown/base-link fixtures, `markdown/inline.rs` for inline/escaping/Unicode/Google Docs fixtures, `markdown/wrapping_preserve.rs` for wrapping/reference/body-width/single-line-break/preserved-tag fixtures, and `markdown/links_code_lists.rs` for nested-list/link/image/code-whitespace/ordered-start fixtures. Existing mock-site route paths and fixture HTML are preserved. The route is now 17 lines, with helper files at 47, 108, 106, and 78 lines. `workpads/research/tasks.md` was not compacted. Focused markdown/mock-site coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

### ✅ Task I19gzk: Split get CLI validation tests

Acceptance criteria:

- Preserve current get CLI validation coverage and assertion behavior.
- Split `tests/get_cli/validation.rs` into smaller behavior-focused test helpers.
- Preserve the existing `validation` module entrypoint used by `tests/get_cli.rs`.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused get CLI validation coverage, line-count checks, and the standard check set.

Status note:

- Completed with D386. `tests/get_cli/validation.rs` now keeps only the `validation` module route. `validation/options.rs` owns the Crawl4AI backend-option forwarding matrix, and `validation/rejections.rs` owns unsupported extractor-option and JavaScript wait rejection coverage plus shared JSON/metadata error assertions. Existing test names and command assertions are preserved. The route is now 4 lines, with helper files at 106 and 104 lines. `workpads/research/tasks.md` was not compacted. Focused get CLI validation coverage, line-count checks, `cargo fmt --check`, and `git diff --check` passed. A first full `cargo test` run hit a transient `session_authorize_reimport_after_user_login_replaces_failed_verification_session` failure outside the touched get validation area; the failing test passed on direct rerun, and a full `cargo test` rerun passed.

### ✅ Task I19gzl: Split AgetExtractor option validation helper

Acceptance criteria:

- Preserve current AgetExtractor backend-option validation coverage and assertion behavior.
- Split `tests/mock_site_cli/aget_extractor/options_waits/validation.rs` into smaller behavior-focused helpers.
- Preserve the existing `validation::assert_option_validation` call path used by mock-site extractor coverage.
- Do not compact `workpads/research/tasks.md`.
- Record the split in compact knowledge routing.
- Verify with focused owned extractor option validation coverage, line-count checks, and the standard check set.

Status note:

- Completed with D387. `tests/mock_site_cli/aget_extractor/options_waits/validation.rs` now keeps only the `assert_option_validation` route. `validation/document.rs` owns document/cache/base-URL and unsupported-option diagnostics, `validation/markdown.rs` owns markdown/link/table/typography/wrapping/Google Docs diagnostics, `validation/browser.rs` owns browser/readiness/timeout/iframe/scrolling diagnostics, and `validation/support.rs` owns the shared invalid-option assertion helper. Existing validation coverage and assertion strings are preserved. The route is now 18 lines, with helper files at 45, 87, 42, and 25 lines. `workpads/research/tasks.md` was not compacted. Focused owned extractor option validation coverage, line-count checks, `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.

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

### ✅ Task I23: Support provider-session injection for login flows

Priority: high.

Acceptance criteria:

- Add first-class support for launching a user-driven login flow with one or more existing local sessions available in the login browser profile, e.g. `aget session login start target --url <url> --session oauth`.
- Preserve the credential boundary: the user still completes provider prompts, passwords, passkeys, and one-time-code steps; the agent only supplies named local session state that the user already authorized.
- Support the intended OAuth-provider model:
  - Users can keep a reusable provider bucket such as `oauth`, or provider-specific buckets such as `google`, `github`, or `okta`.
  - Login attempts for relying-party sites can reuse provider cookies/storage so the user does not need to perform duplicate provider login when local provider state is already available.
  - The target relying-party session saved by `login finish` remains a separate explicit session unless the user asks to compose sessions.
- Define and implement conflict behavior for injected sessions before the login profile is created:
  - Reuse the existing session composition conflict rules where possible.
  - Reject ambiguous cookie/localStorage conflicts instead of silently choosing one session's secrets.
  - Report actionable `requires_user_action` or usage errors when injection cannot be performed.
- Keep session scope enforcement intact:
  - Only inject explicitly named sessions.
  - Preserve replay-time origin/domain checks for later fetches.
  - Do not broaden allowed domains implicitly because a provider session was supplied.
- Update CLI/API/help text and the root `skills/aget/SKILL.md` guidance once the behavior exists.
- Add deterministic tests covering:
  - `login start --session oauth` seeds the pending login profile with provider cookies/storage.
  - multiple injected sessions compose deterministically when disjoint.
  - conflicts fail before opening or saving a login profile.
  - `login finish` saves only the intended target login bucket and does not merge provider state unless explicitly requested.
  - JSON/plain output makes the injected-session behavior and privacy boundary clear.
- Add or update a manual OAuth smoke recipe that validates the intended no-double-login flow without recording credentials or private content.

Status note:

- Completed with D325. `aget session login start` now accepts repeated `--session` flags, loads explicitly named local sessions through the facade, reuses the existing Playwright composition rules to reject conflicts before browser startup, seeds the owned login Chrome profile with the composed state, records injected session names in pending/JSON/plain output, and keeps `login finish` scoped to the target login bucket unless the user later composes sessions. The command-backed compatibility login path reports a usage error for injection because the behavior is owned-backend only. Validation passed with focused parser/API/session-login tests, full `cargo test`, `cargo fmt --check`, and `git diff --check`.

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
