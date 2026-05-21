# Research Knowledge

This compact file records the current decisions and routing context needed for active work. Detailed append-only decision history has been moved to `workpads/research/archive/knowledge/`; use `rg "D164" workpads/research/archive/knowledge` or open the referenced archive file when exact evidence, validation commands, or source paths are needed.

## Current Project Direction

`aget` is a local-first, auth-aware URL-to-agent-context tool. The current implementation direction is a Rust CLI/library with pluggable capability boundaries:

- Facade/orchestration under `src/aget/` wires extractor, browser automation/fallback, and session-store backends.
- AgetExtractor-backed extraction lives under `src/extraction/` and now covers static HTTP fetch, selector/exclusion handling, cleaned HTML, markdown/text/html/json output, artifacts, and selected Crawl4AI-compatible options.
- AgetBrowser-backed CDP behavior lives under `src/browser_cdp/` and now covers temporary Chrome launch, CDP rendering, cookie/storage replay/export, Chrome profile import, login lifecycle, discovery diagnostics, and process cleanup.
- Session model/store/import/login composition is split under `src/session/`.

Safety boundary remains unchanged: `aget` is a generic fetcher for content the user is authorized to access. Authenticated state and content stay local by default. Site-specific paywall/login reasoning belongs to the caller or an agent-facing skill, not the binary.

## Active Work State

- Active workpad: `workpads/research/tasks.md`.
- Main migration umbrella: I19, branch `dep-migration-homegrown-backends`.
- Completed migration phases include source clones/inventory (I19a), backend abstraction audit (I19b), parity tests (I19c), default owned runtime switch (I19f), PoC command-default demotion (I19g), and original oversized extraction/CDP module split (I19i).
- Still open: I19d, I19e, and I19h.
- I19d remains open for full Crawl4AI-quality readability/markdown and richer rendered-page readiness, even though many owned extraction slices are implemented.
- I19e remains open for current-tab attach, broader rendered JavaScript parity, real logged-in profile/keychain smoke coverage, cross-platform close/process lifecycle parity, and fuller startup/error classification.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.

## Recent Decisions To Keep In Working Context

- D158-D160: owned main-content selection and word-count pruning improved with Crawl4AI source-backed heuristics; these are deterministic local improvements, not a full readability port.
- D161-D162: CLI command parsing and session command execution were split without changing public command behavior.
- D163-D164: owned markdown now preserves semantic figure/details/address block boundaries and keeps automatic absolute links title-insensitive like Crawl4AI/html2text.
- D165-D167: follow-up test decomposition split owned mock-site fixtures, cmux session-import tests, and CDP discovery tests. This was done for agent ergonomics; behavior should remain unchanged.
- D168: workpad knowledge history was compacted into archive files so active agents can load current routing context without reading the full append-only record.
- D169: I19j now plans a mechanical rename/boundary refactor from migration-era `owned` backend names toward product engine names: `AgetExtractor`, `AgetExtractorBackend`, `AgetBrowser`, and `AgetBrowserBackend`.
- D170: the first I19j code slice introduced `AgetBrowser` and `AgetBrowserBackend`; the default local browser automation path now routes through this engine wrapper.
- D171: the second I19j code slice introduced `AgetExtractor` and `AgetExtractorBackend`; default extraction and standalone `get_url` helpers now route through the extractor/browser engine wrappers.
- D172: `AgetExtractor` now has direct engine-level coverage that bypasses the `Aget` facade, and active mock-site extractor tests/docs use `AgetExtractor` naming instead of the migration-era `owned` label.
- D173: `AgetBrowser` now has direct engine-level cancellation coverage without the `Aget` facade or command backend, and active browser fallback tests/docs use `AgetBrowser` naming where behavior is current rather than historical.
- D174: support-file compaction now keeps `tasks.md` intact while moving dense `references.md` detail into `archive/references/` and leaving the top-level references file as a current routing index.
- D175: the active `OwnedExtractorBackend` and `OwnedBrowserAutomationBackend` shim types were removed, and default backend enum variants now use `Aget` to describe the local engine path.
- D176: direct `AgetExtractor` coverage now includes selector extraction, exclusion, and `crawl4ai.target_elements` option handling without using the `Aget` facade.
- D177: the markdown renderer was split into `src/extraction/markdown/mod.rs`, `normalize.rs`, and `table.rs` without changing the extraction module interface.
- D178: owned extraction orchestration was split into `src/extraction/owned/mod.rs`, `options.rs`, and `content.rs` without changing the extraction module interface.
- D179: CDP client internals were split into `src/browser_cdp/client/mod.rs`, `transport.rs`, `navigation.rs`, and `state.rs` without changing the browser CDP module interface.
- D180: extraction orchestration helpers were split into `src/extraction/output.rs` and `replay_scope.rs` without changing the extraction module interface.
- D181: Playwright session composition was split into `src/session/playwright/mod.rs`, `compose.rs`, `state_file.rs`, and `tests.rs` without changing the session module interface.
- D182: the `Aget` facade helper surfaces were split into `src/aget/mod.rs`, `authorize.rs`, `backends.rs`, `get_request.rs`, and `session_store.rs` without changing public API paths.
- D183: agent-browser compatibility session helpers were split into `src/session/agent_browser/mod.rs`, `command.rs`, `raw_state.rs`, `state_filter.rs`, and `tests.rs` without changing internal caller paths.
- D184: Chrome session import internals were split into `src/session/chrome/mod.rs`, `compat.rs`, `owned.rs`, `profile.rs`, and `tests.rs` without changing public session import paths.
- D185: login session lifecycle internals were split into `src/session/login/mod.rs`, `types.rs`, `pending.rs`, `compat.rs`, `owned.rs`, `session_merge.rs`, and `tests.rs` without changing session login exports.
- D186: binary session command helpers were split into `src/main_session/mod.rs`, `command_name.rs`, `profile.rs`, `envelope.rs`, and `inspect.rs` without changing `aget session` CLI behavior.
- D187: the mock extractor backend support binary was split into `tests/support/bin/aget_mock_backend.rs` plus `args.rs`, `behavior.rs`, `config.rs`, `http.rs`, and `output.rs` helper modules without changing mock-tool behavior.
- D188: session login CLI tests were split into `tests/session_cli/login.rs` plus start, finish, cancel, direct lifecycle, and ignored real-smoke submodules without changing session CLI behavior.
- D189: non-session get CLI tests were split into `tests/get_cli.rs` plus success, output, validation, and failure submodules while keeping existing session get tests and support routing stable.
- D190: session-backed get CLI tests were split into `tests/get_cli/session.rs` plus replay, fallback/redaction, and ignored real-smoke submodules without changing authenticated get behavior.
- D191: Aget facade/backend API tests were split into `tests/aget_api.rs` plus support, extraction, authorization, session-backend, and default browser-backend submodules without changing backend contract coverage.
- D192: session import CLI tests were split into `tests/session_cli/imports.rs` plus browser, command-backed Chrome, owned Chrome, and real-smoke submodules without changing import behavior.

## Archive Index

| Range | Archive | Contents |
| --- | --- | --- |
| D1-D48 | `archive/knowledge/d001-d048-foundation-and-poc.md` | product premise, prior art, benchmarks, R12 MVP architecture, early implementation and backend-pluggability decisions |
| D49-D76 | `archive/knowledge/d049-d076-migration-setup-and-defaults.md` | OAuth workflow assessment, I19 source inventory, backend abstractions, parity matrix, owned default switch, command-adapter demotion, first final-audit evidence |
| D77-D122 | `archive/knowledge/d077-d122-owned-backend-slices.md` | Crawl4AI and agent-browser feature slices for owned options, markdown behavior, CDP rendering/readiness, Chrome startup/discovery, overlays, linked images, shadow DOM |
| D123-D150 | `archive/knowledge/d123-d150-module-decomposition.md` | original extraction/CDP/test module decomposition through I19i completion |
| D151-D157 | `archive/knowledge/d151-d157-auth-and-browser-import.md` | OAuth-safe authorization API/CLI and conservative browser-neutral Chrome import surface |
| D158-D167 | `archive/knowledge/d158-d167-recent-migration-followups.md` | recent source-backed extraction improvements and follow-up decomposition slices |
| D168 | `archive/knowledge/d168-workpad-knowledge-compaction.md` | workpad knowledge compaction decision and validation |
| D169 | `archive/knowledge/d169-engine-refactor-plan.md` | planned `AgetExtractor`/`AgetBrowser` engine boundaries and I19j guardrails |
| D170 | `archive/knowledge/d170-aget-browser-wrapper.md` | first `AgetBrowser` wrapper and default browser backend wiring slice |
| D171 | `archive/knowledge/d171-aget-extractor-wrapper.md` | first `AgetExtractor` wrapper and default extractor backend wiring slice |
| D172 | `archive/knowledge/d172-aget-extractor-direct-test.md` | direct `AgetExtractor` unit test plus active extractor test/doc naming cleanup |
| D173 | `archive/knowledge/d173-aget-browser-direct-test.md` | direct `AgetBrowser` cancellation test plus active browser test/doc naming cleanup |
| D174 | `archive/knowledge/d174-reference-compaction.md` | support-file reference compaction with `tasks.md` preserved as the active plan |
| D175 | `archive/knowledge/d175-remove-owned-backend-shims.md` | removal of active `Owned*Backend` compatibility shim types and `Default*Backend::Owned` variants |
| D176 | `archive/knowledge/d176-direct-extractor-option-test.md` | direct `AgetExtractor` selector/exclusion/target-elements coverage |
| D177 | `archive/knowledge/d177-markdown-module-split.md` | mechanical markdown renderer split into core, normalization, and table modules |
| D178 | `archive/knowledge/d178-owned-extraction-module-split.md` | mechanical owned extraction split into orchestration, option parsing, and content selection modules |
| D179 | `archive/knowledge/d179-cdp-client-module-split.md` | mechanical CDP client split into core, transport, navigation, and state modules |
| D180 | `archive/knowledge/d180-extraction-helper-module-split.md` | mechanical extraction orchestration helper split into output and replay-scope modules |
| D181 | `archive/knowledge/d181-playwright-session-module-split.md` | mechanical Playwright session split into public types, composition, temp state files, and tests |
| D182 | `archive/knowledge/d182-aget-facade-module-split.md` | mechanical facade split into orchestration, authorization DTOs, backend adapters, get builder, and session-store adapter |
| D183 | `archive/knowledge/d183-agent-browser-compat-module-split.md` | mechanical compatibility split into command execution, state filtering, raw state files, and tests |
| D184 | `archive/knowledge/d184-chrome-session-module-split.md` | mechanical Chrome session split into compatibility import, owned import, profile helpers, and tests |
| D185 | `archive/knowledge/d185-login-session-module-split.md` | mechanical login lifecycle split into types, pending metadata, compatibility flow, owned flow, merge logic, and tests |
| D186 | `archive/knowledge/d186-main-session-module-split.md` | mechanical binary session command helper split into dispatcher, command names, profile resolution, envelopes, and inspect views |
| D187 | `archive/knowledge/d187-mock-backend-module-split.md` | mechanical mock backend support split into root, args, behavior, config validation, HTTP helpers, and output |
| D188 | `archive/knowledge/d188-session-login-test-split.md` | mechanical session login CLI test split into start, finish, cancel, direct lifecycle, and real-smoke modules |
| D189 | `archive/knowledge/d189-get-cli-test-split.md` | mechanical get CLI test split into success, output, validation, failure, and existing session modules |
| D190 | `archive/knowledge/d190-get-session-test-split.md` | mechanical session-backed get CLI test split into replay, fallback/redaction, and real-smoke modules |
| D191 | `archive/knowledge/d191-aget-api-test-split.md` | mechanical Aget facade/backend API test split into support, extraction, authorization, session backend, and default browser backend modules |
| D192 | `archive/knowledge/d192-session-import-test-split.md` | mechanical session import CLI test split into browser, command-backed Chrome, owned Chrome, and real-smoke modules |

## Current Verification Expectations

- For behavior changes, inspect the relevant external source snapshot under `references/repos/crawl4ai` or `references/repos/agent-browser` before porting and record the source-backed decision in the appropriate workpad file.
- Run focused tests for the touched behavior plus `cargo fmt --check` and, at stable points, `cargo test`.
- Before starting another task after a stable point, make an explicit commit decision. The user wants iterative stable commits on this branch.
- `CLAUDE_REVIEW.md` is an untracked review artifact in this worktree; do not rewrite or commit it unless explicitly asked.

## Open Questions

- Can pure Rust browser automation provide reliable persistent profiles and CDP attach, or do we need a small Node/Playwright sidecar?
- Which HTML-to-markdown path gives quality close to curl.md/Firecrawl?
- Can objective/keyword narrowing be implemented without an LLM, or should it start as deterministic section scoring?
- What is the cleanest OpenCode plugin packaging model for a local binary-backed tool?
- How should `aget` represent and redact authenticated/private content before returning it to an agent?
- What policy, reliability, and license constraints apply to YouTube caption extraction, `yt-dlp` audio download, and local ASR model redistribution?
- Which comparable projects already solve local authenticated web extraction, and are any suitable to reuse rather than rebuilding?
- What should be the exact consent boundary for importing or using credentials/session data from the user's existing browser?
- Should current-tab extraction be implemented through CDP debug-port setup first, or deferred until an extension/native bridge is justified?
- Should the MVP include browser-control actions at all, or stay strictly page-fetch/extract/status to avoid duplicating browser MCP tools?
- Can Crawl4AI's profile/session support cleanly reuse a user-authorized dedicated browser profile for authenticated markdown extraction without leaking data outside the machine?
- Should `aget` initially wrap Crawl4AI for browser-rendered markdown, or use `agent-browser` for browser control and own the markdown pipeline?
- Can Crawl4AI create/open a profile directly at a target login URL without manual scripting, and can it do so without using shared CDP ports or disturbing existing browsers?
