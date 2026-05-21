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
