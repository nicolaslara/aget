# Research References

This compact file records current source routing.
Detailed reference rows are archived under `workpads/research/archive/references/`; open those files when exact benchmark output paths, source URLs, license notes, or historical architecture evidence are needed.

## Current Routing

| Need | Open |
| --- | --- |
| Prior-art sources and comparable projects | `archive/references/source-projects-and-comparables.md` |
| Historical benchmark runs and auth bridge reproduction notes | `archive/references/benchmarks.md` |
| Architecture inputs, implementation plans, and current code-boundary maps | `archive/references/architecture-inputs.md` |
| Browser automation primary sources plus Rust dependency candidates | `archive/references/browser-automation-and-rust-candidates.md` |
| Later YouTube/transcript/local-ASR candidates | `archive/references/later-media-inputs.md` |

## Active Source Snapshots

| Project | Local / Source | Use |
| --- | --- | --- |
| Crawl4AI | `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255` | Inspect before porting extraction, markdown, readiness, session, or option behavior. License: Apache-2.0. |
| Crawl4AI content pruning | `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py` | Source for owned readability scoring signals. |
| agent-browser | `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7` | Inspect before porting browser/CDP/session/profile behavior. License: Apache-2.0. |
| cmux | Source review at commit `7142e31d3a749c241843655cac2771927505860c` | `browser cookies get` returns broad cookie data; `aget` must post-filter by explicit allowlist. |
| agent-fetch | `references/repos/agent-fetch` | Extraction-strategy inspiration for Readability/text-density/JSON-LD/Next.js/RSC/WordPress/selectors. License: MIT. |
| Markdown Web Browser | `references/repos/markdown_web_browser` | Product comparison only; license rider and bot-detection framing make direct reuse unsuitable. |

## Current Architecture Pointers

| Topic | Primary Paths | Notes |
| --- | --- | --- |
| Product intent and workflow | `project.md`, `WORKING.md`, `workpads/WORKPADS.md`, `workpads/research/tasks.md`, `workpads/research/knowledge.md` | Load before task work. |
| Aget facade/orchestration | `src/aget/` | Public API and dependency wiring across extractor, browser automation/fallback, and session store backends. |
| Aget facade/backend API coverage | `tests/aget_api.rs`, `tests/aget_api/` | Direct API coverage for extractor/session-store wiring, browser fallback, authorization, session import/login, and default AgetBrowser backend behavior. |
| AgetExtractor boundary | `src/aget_extractor.rs`, `src/extraction/` | Local Crawl4AI-like extraction engine and backend wrapper. Details: `architecture-inputs.md`. |
| AgetBrowser boundary | `src/aget_browser.rs`, `src/browser_cdp/`, `src/session/chrome/`, `src/session/login/` | Local browser/CDP/profile/session engine. Details: `architecture-inputs.md`. |
| Session model/store/import | `src/session/` | Keep auth state local, scoped, and explicitly imported. |
| Binary session CLI execution | `src/main_session/` | Dispatches `aget session` commands and keeps command names, profile argument normalization, JSON envelope shaping, and inspect/redaction views split from `src/main.rs`. |
| Agent-facing usage | `.cursor/skills/aget/SKILL.md`, `.opencode/tools/aget.ts`, `README.md` | Keep command/default-backend wording aligned with current runtime behavior. |
| Mock-site integration coverage | `tests/support/mock_site.rs`, `tests/mock_site_*` | Local deterministic public/auth/session/rendering behavior. |
| Get CLI coverage | `tests/get_cli.rs`, `tests/get_cli/`, `tests/support/get_cli.rs` | Get artifacts, output shaping, validation/failures, and session replay/fallback. |
| Session CLI coverage | `tests/session_cli/`, `tests/support/session_cli.rs` | Session import, authorize, login lifecycle, and backend compatibility. |
| Mock backend test tools | `tests/support/bin/aget_mock_backend.rs`, `tests/support/bin/aget_mock_backend/`, `tests/fixtures/mock-tools/` | Checked-in local mock binaries used by CLI/integration tests. |

## External Primary Sources

| Topic | Source | Notes |
| --- | --- | --- |
| OpenCode custom tools | https://opencode.ai/docs/custom-tools | Project-local tools live under `.opencode/tools/`; TypeScript definitions use `tool()` from `@opencode-ai/plugin`. |
| Chrome remote debugging hardening | https://developer.chrome.com/blog/remote-debugging-port | Chrome 136+ ignores remote debugging flags for the default Chrome data directory; reinforces dedicated-profile automation. |
| Chromium user data directories | https://chromium.googlesource.com/chromium/src/+/HEAD/docs/user_data_dir.md | Primary source for Chrome/Chromium profile paths and user-data-dir behavior. |
| Chrome DevTools Protocol | https://chromedevtools.github.io/devtools-protocol/ | Reference for the current local CDP implementation. |
| Playwright authentication | https://playwright.dev/docs/auth | Storage-state reuse precedent; sessionStorage caveat remains relevant. |
| Playwright persistent contexts | https://playwright.dev/docs/api/class-browsertype#browser-type-launch-persistent-context | Dedicated profile login/profile reuse reference. |
| OWASP Logging Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html | Logs should exclude or mask tokens, passwords, session IDs, and sensitive personal data. |
| NIST Privacy Framework | https://www.nist.gov/privacy-framework | Data minimization, provenance, audit records, and deletion/disposition framing. |

## Rust Dependencies In Current Direction

| Need | Candidate | Notes |
| --- | --- | --- |
| HTTP | `ureq` | Blocking HTTPS-capable client used by the local static extractor. License: MIT OR Apache-2.0. |
| HTML parse/select | `scraper`, `html5ever`, `ego-tree` | Current DOM parse, selector, removal, and traversal stack. Licenses recorded in the archive. |
| URL parsing/joining | `url` | Used for base-URL-aware markdown links and targets. License: MIT OR Apache-2.0. |
| Browser/CDP WebSocket | `tungstenite` | Blocking WebSocket client for local Chrome DevTools Protocol. License: MIT OR Apache-2.0. |
| CLI | `clap` | Standard CLI framework. |

## Reference Archive

| Archive | Contents |
| --- | --- |
| `archive/references/source-projects-and-comparables.md` | Original source-project and comparable-project rows, including licenses and local clone notes. |
| `archive/references/benchmarks.md` | R0a benchmark table and positive auth bridge reproduction notes. |
| `archive/references/architecture-inputs.md` | Historical architecture inputs, large AgetExtractor/AgetBrowser slice maps, and default-runtime rows. |
| `archive/references/browser-automation-and-rust-candidates.md` | Browser automation standards/docs and Rust candidate dependency notes. |
| `archive/references/later-media-inputs.md` | YouTube transcript, `yt-dlp`, Whisper-family, and ASR follow-up references. |
