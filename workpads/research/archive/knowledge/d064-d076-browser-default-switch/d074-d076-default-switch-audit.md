# D74-D76: Default Switch And Local Audit

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
