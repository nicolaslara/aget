# Research Knowledge

This file records decisions, lessons, and architecture notes discovered during research.

## Product Premise

`aget` should provide local-only, auth-aware page extraction for agents. It should keep the simple known-URL workflow of `curl.md` and selectively adopt Firecrawl-like capabilities where they matter for agents.

## Initial Decisions

### D1: Local-first is non-negotiable for authenticated content

Authenticated page content, cookies, and headers must not be sent to hosted `curl.md`, hosted Firecrawl, or any third-party service by default.

### D2: Start with a small local tool, not a full Firecrawl fork

Firecrawl is self-hostable and powerful, but likely heavier than needed for the MVP. The first design should evaluate a small local Rust CLI and agent plugin before considering a Firecrawl fork.

### D3: Reuse concepts before reusing code

Borrow UX and architecture patterns from `curl.md` and Firecrawl first. Only reuse code after license, dependency, and architectural fit are understood.

### D4: Dedicated browser profile is the safer default

Using the user's main browser profile directly can risk locking/corruption and unclear automation boundaries. A dedicated persistent profile should be the default. CDP attach to an existing browser can be an advanced option.

### D5: Auth/session models need separate product and security evaluation

There are at least three plausible models: directly automate/read the user's existing browser session, ask the user to log in through an `aget`-managed browser/profile, or explicitly copy/import session material from the user's browser into `aget`'s local store. These differ in consent boundaries: direct mode uses browser data at fetch time, dedicated mode only uses `aget`-owned local state, and copy/import mode can require explicit consent at import time while later fetches only use `aget`'s local database. The MVP should not assume these are equivalent.

### D6: YouTube/video markdown should be transcript-first, local-ASR second

For the later YouTube/video feature, start by retrieving available manual or auto captions. If captions are unavailable or low quality, fall back to local audio transcription. Prefer local ASR over paid APIs by default; `faster-whisper` is a strong iteration backend, while `whisper.cpp` is the strongest portable embedding/Rust-integration candidate.

### D7: Comparable projects validate `aget`, but none replace it directly

Comparable projects split into three clusters: local browser-control MCP servers, extraction/crawl pipelines, and markdown/document converters. Browser-control projects cover authenticated sessions and current-browser access, while extraction projects cover low-noise markdown quality. No reviewed project cleanly combines `curl.md`-style URL-to-agent-context UX, local-only authenticated content handling, dedicated provenance/redaction, and a small Rust-first implementation.

| Project | Local-first fit | Auth/session fit | Extraction fit | Agent integration | License/reuse fit | Recommendation |
| --- | --- | --- | --- | --- | --- | --- |
| TabNab | Strong | Strong real-browser cookies via Playwright/CDP | Medium: markdown/DOM extraction exists | Strong MCP tools | Weak: PolyForm/commercial | Product inspiration only. |
| Chrome DevTools MCP | Strong local server | Medium: debug-port browser attach, content exposed to MCP | Weak: debug/snapshot orientation | Strong MCP/Claude/OpenCode ecosystem | Unknown from README, but official package; inspect before reuse | Use as privacy warning and MCP/browser-debug reference. |
| Real Browser MCP | Strong | Strong existing-browser sessions via extension | Weak-to-medium: text/snapshot/eval, not markdown-first | Strong MCP | Strong: MIT | Strong reference for current-browser/current-tab architecture. |
| Playwright MCP | Strong | Strong persistent profile/storage-state patterns | Medium: snapshots/screenshots/actions, not markdown-first | Strong MCP | Strong: Apache-2.0 | Reference for MVP browser profile and MCP shape. |
| BrowserMCP | Strong | Strong existing-browser sessions via extension | Weak-to-medium | Strong MCP | Strong: Apache-2.0, but build caveats | Reference for extension-assisted current-tab mode. |
| Crawl4AI | Strong | Strong cookies/headers/profile/session patterns | Strong markdown/structured/crawl extraction | Good MCP/CLI/API | Strong: Apache-2.0 | Benchmark extraction behavior; avoid copying Python architecture wholesale. |
| browser-use | Medium-to-strong | Good browser profile/task auth patterns | Medium: automation-first | Strong agent/tool patterns | Strong: MIT | Inspiration for interaction flows, not fetch output. |
| SingleFile | Strong | Strong browser-extension/current-tab precedent | Medium: faithful HTML, not markdown | Emerging MCP adjacency | Weak: AGPL-3.0 | Product/architecture reference only. |
| Browsertrix Crawler | Strong/self-host | Medium browser crawl/session patterns | Medium: archive-first | Weak for agents | Weak: AGPL-3.0 | Crawl architecture reference only. |
| Maxun | Self-hostable | Strong product overlap for behind-login extraction | Strong platform features | MCP-related | Weak: AGPL-3.0 | Competitive/product reference only. |
| Trafilatura | Strong library | None | Strong main-content/text/markdown extraction | Weak direct | Strong: Apache-2.0 | Extraction benchmark/reference. |
| Mozilla Readability | Strong library | None | Strong article extraction baseline | None | Strong: Apache-2.0 | Candidate concept/library baseline for readability mode. |
| MarkItDown | Strong local converter | Weak | Good document/HTML markdown conversion | Good LLM ecosystem fit | Strong: MIT | Output-format and conversion reference. |
| Markdown Web Browser | Strong local service | Medium: Playwright storage-state profiles | Strong visual/OCR markdown with provenance | Good CLI/API/artifacts | Weak: restrictive rider | Architecture inspiration only; safety conflict due bot-bypass positioning. |
| agent-fetch | Strong local CLI/library | Medium: cookies and Netscape cookie files, no browser profile | Strong HTTP extraction strategies | Good agent skill/CLI | Strong: MIT | Strongest reusable extraction-strategy reference. |
| agent-browser | Strong local Rust CLI | Strong CDP sessions, profiles, cookies/storage state | Weak: browser control/snapshots, not markdown | Strong agent skills/CLI | Strong: Apache-2.0 | Strongest Rust browser/session architecture reference. |

### D8: R0 build/reuse recommendation

Build `aget` as a focused local tool rather than adopting a full existing platform. The likely direction is a small Rust CLI for HTTP/config/cache/output plus either pure-Rust CDP automation or a narrow Playwright/Node sidecar if Rust browser automation fails reliability checks. Reuse concepts and tests from permissive projects first: `agent-fetch` for extraction strategy, `agent-browser` for Rust CDP/session architecture, Playwright MCP for profile/MCP conventions, Real Browser MCP/BrowserMCP for extension-assisted current-tab mode, and Crawl4AI/Trafilatura/Readability for extraction-quality benchmarks.

Do not reuse AGPL projects unless `aget` intentionally adopts AGPL obligations. Do not reuse TabNab or Markdown Web Browser code without explicit legal review because their licenses are not a clean permissive fit for this project. Avoid importing stealth, CAPTCHA, bot-detection bypass, proxy escalation, or paywall-bypass features from adjacent tools; they conflict with `aget`'s authorization-only safety boundary.

### D9: Current-tab likely needs extension or explicit debug-port setup

The comparable projects suggest two viable current-browser models. CDP/debug-port tools require the browser to be launched or configured for remote debugging, which is explicit but less seamless. Extension-assisted tools can operate in the user's normal browser with existing sessions and active tabs, but require installing a local extension/native bridge and a sharper consent boundary.

### D10: Initial benchmark results favor Crawl4AI for local markdown and agent-browser for local browser control

R0a smoke tests compared `curl.md`, Firecrawl, Crawl4AI, and `agent-browser` on a static page, a JavaScript-rendered page, and an unauthenticated HelloInterview paywalled page. `curl.md` produced the cleanest simple static markdown but did not render `https://quotes.toscrape.com/js/`, returning only the shell title/login content. Firecrawl and Crawl4AI both rendered the JS page and produced markdown, while `agent-browser` rendered it locally and extracted readable text/HTML but not markdown.

| Tool | Static page | JS-rendered page | HelloInterview unauth | Local/private fit | Main observed gap |
| --- | --- | --- | --- | --- | --- |
| curl.md | Clean markdown with metadata | Failed to capture JS-rendered quote content | Clean public/paywall-limited markdown | Weak for private auth because hosted service receives content | No local authenticated browser session. |
| Firecrawl | Clean markdown after transient API retry | Rendered JS content, but tag words were concatenated | Rendered public/paywall-limited markdown with nav/sidebar noise | Hosted API is unsuitable for private authenticated content by default; self-host is heavier | Hosted/private boundary and self-host complexity. |
| Crawl4AI | Clean local markdown | Rendered JS content locally; spacing around quote authors needed cleanup | Rendered public/paywall-limited markdown locally with nav/sidebar noise | Strongest local markdown/browser candidate so far | Runtime/setup weight and need for content pruning/profile auth evaluation. |
| agent-browser | Local text/HTML extraction | Rendered JS content locally with readable text spacing | Local public/paywall-limited text/HTML | Strongest local browser/session control candidate so far | No built-in markdown/readability extraction. |

The benchmark supports a hybrid hypothesis: Crawl4AI is the closest all-in-one local browser-to-markdown prior art, while `agent-browser` is the strongest Rust/CDP auth/session substrate. A small `aget` MVP could either wrap Crawl4AI first for browser-rendered markdown, or combine `agent-browser`-style browser acquisition with an `agent-fetch`/Readability/Crawl4AI-inspired markdown pipeline.

### D11: Auth benchmarking needs explicit session ownership choices

Unauthenticated HelloInterview extraction returned only public/paywall-limited content across tools. `agent-browser --profile Default` worked as a Chrome profile snapshot mechanism, but the available Chrome profile was not logged in to HelloInterview. A dedicated headed `agent-browser` profile was opened at `/Users/nicolas/.agent-browser/profiles/aget-hellointerview` for manual login before the authenticated benchmark. This reinforces that `aget` needs a first-class status command that distinguishes: not logged in, logged in but non-premium, logged in premium, and extraction blocked by site policy or app state.

### D12: Auth flows must never disturb the user's normal browser

Auth/profile experiments exposed a critical UX and safety requirement: a web-fetch tool must not close, restart, attach to, or otherwise disturb the user's default browser unless the user explicitly asks for that exact mode. Browser lifecycle operations such as broad `close --all`, shared CDP ports, or profile managers that clean up an attached/default process are unacceptable defaults. The safe default is an isolated, named, tool-owned browser profile/process with a target login URL opened explicitly for the user.

For login setup, the tool must open the actual login/callback URL, not a blank browser window. A usable flow should say what profile is being used, what URL was opened, what the user should do, and how the session will be saved. If the tool cannot create that flow without risking the user's normal browser, it should stop and ask for an explicit advanced mode.

### D13: The Crawl4AI skill is SDK-oriented, not enough for safe interactive auth by itself

The downloaded Crawl4AI skill in `/Users/nicolas/Downloads/crawl4ai/SKILL.md` is useful for local markdown, fit markdown, structured extraction, batch crawling, and JavaScript waits. Its auth example uses `CrawlerRunConfig(session_id=...)` plus injected JavaScript to fill credentials and click submit. That pattern is acceptable for controlled test apps or explicit scripted credentials, but it is not the right default for user-driven SSO, 2FA, or premium/private content because it asks the agent to handle credentials/selectors directly.

Crawl4AI still appears viable as the default local markdown extractor. For authenticated content, the safe skill workflow should add a wrapper script or documented recipe that creates a dedicated profile, opens the exact login/callback URL, lets the user complete login in a visible browser, saves the profile, and then runs `crwl crawl --profile <name>`. The stock skill does not yet provide that complete user-safe flow.

### D14: Existing Chrome login plus agent-browser profile snapshot successfully extracts authenticated content

The successful HelloInterview authenticated benchmark used regular Google Chrome for login, not an automation-controlled browser. After the user logged in manually and quit Chrome, `agent-browser --profile Default --session hi-auth-after-manual-chrome-login open <url>` snapshotted Chrome's authenticated `Default` profile and extracted local text/HTML. The result no longer contained sign-in or paywall CTAs and included video metadata/content plus the full premium article body.

This is the best observed auth pattern so far: keep OAuth/login in a normal trusted browser, then use `agent-browser` to snapshot that browser profile for local extraction. It avoids putting credentials in agent context and avoids Google OAuth rejecting automated browsers. The tradeoff is that the user may need to quit Chrome first so profile files are not locked, and the output is text/HTML rather than markdown unless a conversion step is added.

### D15: Crawl4AI skill scripts are not interchangeable with the `crwl` CLI

The downloaded Crawl4AI skill's `basic_crawler.py` succeeded on `example.com` and the JS-rendered quotes page, but produced blank output and a blank screenshot for the HelloInterview page where `crwl crawl` had previously extracted public/paywall-limited content. The skill script enables screenshots and image waits, which may interact poorly with some app pages. For replacing `curl.md`/Firecrawl in agent skills, prefer `crwl crawl` CLI first, and treat the SDK scripts as examples requiring tuning rather than robust defaults.

### D16: Direct Chrome profile copy into Crawl4AI did not authenticate in first test

A proof-of-concept copied Chrome `Default` into `~/.crawl4ai/profiles/chrome-default-hellointerview-r0a` using the same broad structure as `agent-browser`: copy `Local State` plus the profile directory, excluding caches and singleton/lock files. Running `crwl crawl --profile chrome-default-hellointerview-r0a` still returned unauthenticated HelloInterview paywall content.

A second proof-of-concept launched system Google Chrome on that copied profile and connected Crawl4AI over CDP. Crawl4AI extraction worked, but the content was still unauthenticated. The result is negative but not conclusive because Chrome was still running during the copy, so SQLite/WAL/profile lock consistency may have affected the copied state. Still, this suggests `agent-browser --profile Default` is doing more reliably for Chrome auth reuse than a naive profile copy into Crawl4AI.

The subagent recommendation was to treat full profile copy as a baseline experiment only, then narrow later. It flagged macOS Keychain cookie decryption, profile locks, over-copying private data, and plaintext storage-state files as the major risks. A future clean retry should require Chrome fully quit before copying, or should export decrypted auth state from a normal/browser-controlled context rather than copying encrypted cookies directly.

### D17: `agent-browser state save` can bridge auth into Crawl4AI markdown extraction

Copying the already-authenticated `agent-browser` temp Chrome profile into a Crawl4AI profile did not authenticate Crawl4AI. The output matched unauthenticated Crawl4AI markers: `Sign in / Sign up`, `Premium users can view this video once signed in`, and `Purchase Premium to Keep Reading`.

Exporting browser state from the known-good `agent-browser --profile Default --session hi-auth-after-manual-chrome-login` session with `agent-browser state save`, then passing that JSON to Crawl4AI via `BrowserConfig(storage_state=...)`, did authenticate Crawl4AI. The resulting markdown included the logged-in marker `StrongMagentaJackal227` and `Posting as StrongMagentaJackal227`, and did not include the sign-in/paywall markers.

This is the strongest bridge observed in R0a so far, but it is proven on one site/account/session only. It does not yet prove durability across session expiry, 2FA refresh flows, multiple domains, IndexedDB-heavy apps, non-Chrome browsers, or sites that bind sessions to a specific browser/profile fingerprint.

Observed working flow:

1. Let the user authenticate in normal Chrome.
2. Use `agent-browser --profile Default` to snapshot/open a local browser session without handling the user's password/OAuth flow.
3. Use `agent-browser state save` to export decrypted, Playwright-compatible state.
4. Feed that state to Crawl4AI for markdown/readability extraction.

The exported state file contains live cookies and local/session storage. It is credential-equivalent bearer material even though it does not contain the user's password. It should be treated as highly sensitive, stored outside repo/workpads by default, encrypted or short-lived, and deleted after use unless the user explicitly opts into persistence.

Authenticated markdown/HTML outputs are also sensitive because they may contain private or paid content plus account identifiers. They should stay local by default, be excluded from commits unless explicitly approved, and support redaction/short-lived retention in any `aget` design.

### D18: cmux browser panes are controllable/extractable, but not a markdown extractor

The installed `cmux` CLI exposes a local socket API for browser panes. In a disposable browser pane, `cmux browser goto`, `wait`, `get text body`, `get html body`, and `snapshot` worked on a static page and a JavaScript-rendered page. This makes cmux a viable current-workspace/browser-surface control plane for manual browsing, interaction, screenshots, and raw DOM/text extraction.

cmux is not a direct replacement for `aget`: its browser API is action/surface oriented, not URL-to-clean-markdown/readability oriented. It can provide rendered text/HTML and Playwright-like state operations, but an `aget` layer would still need site/session scoping, sensitive-state handling, content pruning, markdown conversion, provenance, caching, and agent-safe output contracts.

`cmux browser state save` worked but exported broad browser cookies/storage, including data unrelated to the current test page. Treat cmux state files as credential-equivalent bearer material. For any `aget` integration, prefer extracting page content from a specific cmux surface over exporting global browser state unless state can be scoped, redacted, encrypted, and explicitly user-approved.

Follow-up scoped-cookie test: a disposable local page set a test cookie in a cmux browser surface. `cmux --json browser cookies get --domain 127.0.0.1` returned only that domain's cookie, and converting it to Playwright `storage_state` let Crawl4AI replay the cookie successfully. However, `cmux --json browser cookies get --url http://127.0.0.1:9168` returned broad cookies in this cmux version, so `aget` must not trust cmux URL scoping alone; it should always post-filter cookies by an explicit allowlist before persistence or replay.

The same Playwright-style state file was not successfully replayed through `agent-browser state load` in the local cookie test. `agent-browser` accepted the domain-based state file but did not send the cookie on request; its state importer also rejected Playwright's `url`-scoped cookie form as missing `domain`. Treat Crawl4AI replay as proven and `agent-browser` replay of externally-built scoped state as unresolved.

### D19: Thin-wrapper PoC should centre session management

The next PoC direction is specified in `workpads/research/session-wrapper-poc-spec.md`. The key product decision is that `aget` should default to an empty session, let users create independent named sessions, and let requests explicitly combine sessions with repeated `--session` flags. This supports OAuth/provider workflows without making provider cookies ambient: users can combine `google` and `hellointerview` for login, then later crawl with only the relying-party session if it tests cleanly.

V1 should be a thin wrapper: use `agent-browser` for Chrome profile state acquisition, cmux as an optional current-pane/session source, and Crawl4AI for markdown extraction from temporary Playwright state. V2 can replace those internals while preserving the same session model.

### D20: Agent-facing CLI must be non-interactive and extensible

The PoC spec now treats `aget <url>` as an alias for `aget get <url>`, keeps commands non-interactive by default, and requires bounded timeouts plus structured `--json` output for agent callers. Commands that need user action should fail with a machine-readable `requires_user_action` result unless explicitly invoked in interactive mode.

The `get` command should accept common output-shaping options (`--format`, `--selector`, `--only-main`, `--max-tokens`, `--max-chars`, `--wait-for`) and an extractor-specific escape hatch so v1 can expose Crawl4AI features without freezing the final `aget` API too early.

Future action/API workflows, such as searching or adding to cart on behalf of a user, are explicitly out of v1 scope but should guide the design: sessions must remain explicit and composable, read-only extraction must be separable from mutating actions, and command outputs must carry enough provenance for agents to make safe decisions.

## MVP Architecture Proposal (R12)

### Architecture Summary

Build v1 as a thin local wrapper that proves the product model before replacing internals. Rust owns the CLI, session model, filtering, composition, metadata, temp-file lifecycle, and process orchestration. Existing tools remain backend adapters:

- Crawl4AI is the v1 rendered extraction backend.
- `agent-browser` is the v1 Chrome profile/state acquisition backend.
- cmux is an optional v1 source for users who already browse inside cmux panes.

The first implementation slice should not attempt custom browser automation, crawling, map/search, current-tab browser extension work, or mutating site actions. It should prove that `aget` can fetch with an empty session, store a named scoped session, replay that session through Crawl4AI, and return local markdown with provenance and sensitivity metadata.

### MVP Boundary

MVP includes:

- `aget <url>` as an alias for `aget get <url>`.
- Empty-session fetch by default.
- Named local sessions with list/inspect/delete.
- One-session replay through temporary Playwright storage state.
- Crawl4AI-backed markdown extraction.
- `--json` output for agents.
- Timeout-bounded command execution.
- Local output metadata and sensitivity warnings.
- Optional cmux domain-scoped cookie import after the core replay path works.

MVP excludes until later:

- Chrome import via `agent-browser` as a required first slice, though the architecture reserves the adapter.
- Session composition CLI, except the internal composition primitive should support zero/one sessions immediately.
- Encryption at rest beyond restrictive file permissions and explicit PoC warnings.
- MCP server.
- Site action/API mode.
- Map/crawl/batch.
- Objective narrowing beyond simple extractor options/truncation.

### CLI Commands

Initial slice:

```bash
aget <url> [--session <name>] [--out <path>] [--json] [--timeout <duration>]
aget get <url> [--session <name>] [--out <path>] [--json] [--timeout <duration>]
aget session list [--json]
aget session inspect <name> [--json] [--show-secrets]
aget session delete <name>
```

Next slice:

```bash
aget session import cmux --surface <surface> --name <name> --domain <domain>...
aget session import chrome --profile <profile> --name <name> --domain <domain>...
aget session compose <new-name> --session <name>...
aget session test <name> <url> [--must-contain <text>] [--must-not-contain <text>]
```

Common `get` options should be represented in the CLI early, even if only some map to Crawl4AI in v1:

```bash
--format <markdown|html|text|json>
--selector <css>
--exclude-selector <css>
--only-main
--wait-for <text-or-selector>
--max-chars <n>
--max-tokens <n>
--extractor-option <key=value>
```

All commands are non-interactive by default. Commands that require user action must return a structured `requires_user_action` error unless invoked through an explicit interactive command or flag.

### Config Layout

Default home:

```text
~/.aget/
  config.toml
  sessions/
  runs/
  cache/
  tmp/
```

Testing override:

```text
AGET_HOME=/tmp/aget-test-home
```

Initial `config.toml` shape:

```toml
[backend.crawl4ai]
command = "uv"
args = ["run", "--with", "crawl4ai"]
helper = "scripts/crawl4ai_extract.py"

[backend.agent_browser]
command = "npx"
args = ["-y", "agent-browser"]

[backend.cmux]
command = "cmux"

[timeouts]
command_seconds = 60
navigation_seconds = 30
extraction_seconds = 45
backend_startup_seconds = 20

[privacy]
persist_sessions_plaintext = true
warn_plaintext_sessions = true
default_output_retention = "local"
redact_inspect_values = true

[providers]
oauth_domains = ["accounts.google.com", ".google.com", "facebook.com"]
```

Plaintext persisted sessions are acceptable only as a PoC compromise if files are outside the repo, created with restrictive permissions, and clearly warned as credential-equivalent. Encryption at rest is a hardening task before broader use.

Required PoC file modes on Unix-like systems:

- `~/.aget`, `sessions/`, `runs/`, `cache/`, and `tmp/`: `0700`.
- Session JSON files, temporary Playwright state files, and metadata files that mention session names or sensitive artifacts: `0600`.
- Public markdown artifacts may also use `0600` by default for consistency; authenticated artifacts must not be world-readable.

### Session Store

Session files live in `~/.aget/sessions/<name>.json`. Each session is scoped by explicit cookie domains and storage origins. Values are hidden by default in CLI output. The session model should preserve provenance so composed sessions can later explain which source contributed each cookie/origin.

The store must support an empty state without reading any browser or saved session:

```json
{"cookies": [], "origins": []}
```

### Cache And Run Layout

Use `runs/` for artifacts and reserve `cache/` for later reuse. MVP should prioritize correctness over cache hits.

```text
~/.aget/runs/<run-id>/
  content.md
  content.html        # optional/debug
  screenshot.png      # optional/debug
  metadata.json
```

`metadata.json` records URL, final URL if known, extractor, sessions used, allowed domains, sensitivity, timings, truncation, artifact paths, warnings, and backend versions if available.

`cache/` is deferred until after the first extraction path works. When implemented, cache keys should include URL, extractor, output format, selector/objective options, freshness mode, and the names or hashes of selected sessions. Authenticated outputs must be sensitive by default and should not be reused across different session selections.

### Output Schema

Human output should be concise by default. `--json` should return stable machine-readable data.

Success shape:

```json
{
  "ok": true,
  "url": "https://example.com",
  "final_url": "https://example.com",
  "format": "markdown",
  "content": "# Example...",
  "artifacts": {
    "markdown": "/Users/name/.aget/runs/run-id/content.md",
    "metadata": "/Users/name/.aget/runs/run-id/metadata.json"
  },
  "sessions": [],
  "sensitive": false,
  "warnings": [],
  "timing_ms": {"total": 1234},
  "limits": {"max_chars": null, "max_tokens": null, "truncated": false}
}
```

Error shape:

```json
{
  "ok": false,
  "error": {
    "code": "requires_user_action",
    "message": "Chrome must be quit before profile snapshot can proceed.",
    "retry": "Quit Chrome manually, then rerun with the same command."
  }
}
```

Initial error categories:

- `usage_error`
- `backend_unavailable`
- `timeout`
- `requires_user_action`
- `auth_failed`
- `extraction_failed`
- `session_conflict`
- `privacy_policy_blocked`

### Privacy Model

Session material, temporary storage-state files, cookies, local/session storage, screenshots, authenticated HTML, and authenticated markdown are credential-sensitive or private local data.

MVP rules:

- No ambient auth: `aget get <url>` uses an empty session.
- Sessions are selected explicitly with `--session`.
- Broad backend exports are allowed only as short-lived temp files.
- Temp state files are deleted on success and failure.
- Session values are redacted by default.
- Session files live outside the repo by default.
- `~/.aget` directories must be created with `0700`; session and temp-state files must be created with `0600`.
- `.aget/`, benchmark auth outputs, and tool search artifacts remain ignored.
- cmux import must use explicit domain filters and post-filter results; do not trust cmux URL scoping alone.
- `agent-browser` raw state export must be filtered before persistence.
- Provider/OAuth domains must be visible in `session inspect` and should warn when included.

### Plugin Approach

OpenCode integration should be CLI-backed first. A minimal plugin or command wrapper should call `aget` as a local binary and expose stable tool schemas for:

- `aget_fetch` -> `aget get --json`
- `aget_session_list` -> `aget session list --json`
- `aget_session_inspect` -> `aget session inspect --json`

MCP is deferred until the CLI and session model settle. The CLI-backed OpenCode path is enough for v1 because it preserves one source of behavior and avoids maintaining an always-on local server before the privacy model is hardened.

### Implementation Tasks

Implementation should proceed in independently testable slices:

| Task | Goal | Key tests |
| --- | --- | --- |
| I0 | Rust CLI skeleton with alias, global flags, error categories | CLI parse/unit tests; `aget --help`; `aget get --help` |
| I1 | `AGET_HOME` storage and session model/list/inspect/delete | serialization, redaction, restrictive paths, test-home isolation |
| I2 | Allowlist filtering and Playwright state composition | empty state, one session, duplicate dedupe, conflict rejection, temp cleanup |
| I3 | Crawl4AI adapter and empty-session fetch | local server fetch, `--json` success/error, timeout handling |
| I4 | Local cookie replay with hand-written session | empty fetch has no cookie; session fetch sends cookie |
| I5 | cmux import adapter | optional cmux e2e; domain post-filter; backend-unavailable path |
| I6 | Output shaping and limits | format handling, char truncation, metadata correctness |
| I7 | Chrome import via `agent-browser` | manual authenticated verification; raw state deletion; requires-user-action behavior |
| I8 | Multi-session per-request composition and `session compose` | app+provider local test, no source mutation, provenance |
| I9 | OpenCode CLI-backed plugin/tool wrapper | tool schema calls local CLI and preserves JSON contract |
| I10 | Security/privacy hardening pass | redaction review, cache retention, temp deletion, ignored artifacts, docs |

### Open Risks And Deferred Features

| Area | Risk | MVP decision |
| --- | --- | --- |
| Plaintext sessions | Local compromise exposes bearer material | Permit only as warned PoC with restrictive files; encryption before broader use |
| Crawl4AI dependency | Python/Playwright setup is heavy | Accept for v1 wrapper; replace or embed later |
| Chrome import | Profile locks and broad state export | Use only filtered temp export; stop for user action rather than closing Chrome |
| cmux import | URL scoping returned broad cookies in testing | Use explicit domains and post-filter; cmux remains optional |
| Cache | Authenticated cache reuse can leak context across sessions | Defer cache reuse; write run artifacts only |
| Current-tab | Needs extension or debug-port setup for non-cmux users | Defer beyond v1 first slice |
| MCP | Server lifecycle increases privacy surface | Defer; CLI-backed OpenCode integration first |
| Objective narrowing | Quality requires more research | Start with selectors, truncation, and backend options |
| Site actions/API | Mutating actions need approval model and safety controls | Defer; preserve session provenance and read/write classification vocabulary |
| Pure Rust browser/extraction | Unknown quality/reliability | V2 research/replacement path, not v1 blocker |

### R12 Confidence

Confidence: Medium-high. The proposal is backed by local benchmarks and explicit PoC tests for the key auth bridge and cmux cookie replay. Remaining uncertainty is mostly around product/security hardening choices, especially plaintext session persistence and how quickly to replace Crawl4AI/agent-browser internals.

### D21: I3 empty-session Crawl4AI fetch is implemented

I3 is complete: `aget get` defaults to empty Playwright storage state, shells out to the local Crawl4AI helper, and writes run artifacts as `content.md` and `metadata.json`. The stable JSON contract includes `extractor: "crawl4ai"`, and `scripts/demo_real_cli.sh` is the reusable real CLI demo path showing the default-backend flow.

Real demo runs surfaced Crawl4AI stdout progress logs; the helper now handles this by parsing the final JSON line and redirecting Crawl4AI stdout to stderr so progress noise no longer breaks extraction.

Verification covered fake-backend and local-server tests for success, timeout, error paths, temp cleanup, and noisy stderr; `cargo test`, `cargo fmt --check`, and `git diff --check` passed, and Oracle blocker review was PASS.

Future work stays split as I4 for session replay and I6 for output shaping.

### D22: I4 local cookie replay path is implemented

I4 adds explicit one-session replay for `aget get <url> --session <name>`. The Rust CLI loads the named local session, composes it into a temporary Playwright storage-state file, and passes that file through the existing Crawl4AI helper. Empty fetches still compose `{"cookies": [], "origins": []}` and return `sessions: []`, `sensitive: false`.

Verification now covers two layers. The normal fake-backend integration test asserts that the generated Playwright state contains the hand-written cookie fixture and that command output plus `metadata.json` record `sessions: ["local"]` and `sensitive: true`. An ignored real-backend integration test, `real_crawl4ai_replays_named_session_cookie`, starts a local cookie echo server and verifies default Crawl4AI behavior: empty state does not send the synthetic `sid` cookie, while `--session local` sends `sid=secret-cookie` through the rendered browser request.

The real test also surfaced two useful backend behaviors. Crawl4AI injects its own `cookiesEnabled=true` cookie even with empty user state, so tests should assert absence of the selected session cookie rather than zero Cookie header. Crawl4AI 0.8.6 flags tiny local pages as `minimal_text`, so local replay fixtures need enough visible text to pass structural checks.

Post-implementation review found and fixed three hardening points: session-backed fetches are marked sensitive whenever any session is selected, session names reject path-like values before loading/saving/deleting, and nonzero backend exits cannot be converted into successful `aget` results even if stdout contains `{"ok": true}`.

### D23: I5 optional cmux cookie import is implemented

I5 adds `aget session import cmux --surface <surface> --name <name> --domain <domain>...`. The import path shells out to cmux's cookie API through an optional `AGET_CMUX_COMMAND` override, stores the result as `SessionSource::Cmux { surface }`, marks the session sensitive, and persists only cookies plus explicit `allowed_cookie_domains`. It does not call `cmux browser state save` and does not use cmux URL scoping.

The adapter treats cmux's domain filter as coarse because source research confirmed cmux filters domains by substring. `aget` therefore post-filters every returned cookie by exact/suffix domain rules before persistence. Host-only `example.com` and domain cookie `.example.com` are not accepted when only `docs.example.com` is allowed; the parent domain must be explicitly allowed before broader parent-domain cookies are imported. Because cmux's cookie JSON does not expose `HttpOnly`, imported cookies default to `http_only: true` to avoid weakening browser cookie protections during replay.

Verification covers fake and optional real paths. `tests/session_cli.rs` uses a fake cmux CLI to prove repeated `--domain`, sensitive session persistence, redacted inspect output, disallowed-cookie filtering, and missing-backend `backend_unavailable`. The ignored `real_cmux_imports_loopback_cookie` test uses a user-provided disposable cmux surface, imports a loopback cookie, and replays it through `aget get --session` with a fake Crawl4AI backend that reads the generated Playwright state and sends the cookie to a loopback echo server. The ignored `real_cmux_import_replays_loopback_cookie_through_crawl4ai` test uses the same loopback-only import setup and replays the imported session through the real Crawl4AI backend.

Verification passed: `cargo test --test session_cli`, `cargo test domain_matching_rejects_substring_only_matches`, `cargo test`, `cargo fmt --check`, `cargo check`, `git diff --check`, and a manual fake-cmux CLI smoke. `lsp_diagnostics` remains blocked for macro-heavy Rust files by the local rust-analyzer/proc-macro API mismatch (`proc-macro server's api version (6) is newer than rust-analyzer's (5)`), while cargo compile/tests are clean.

Post-implementation review initially found two blockers and one acceptance gap. The blockers were fixed by defaulting unknown cmux cookies to `http_only: true`, replacing shell-interpreted `AGET_CMUX_COMMAND` execution with direct `Command::new`, and tightening leading-dot domain cookies so parent-domain cookies require the parent domain to be explicitly allowed. The acceptance gap was fixed by adding the ignored real cmux plus real Crawl4AI loopback replay test. Security and context re-reviews then passed with no remaining blockers.

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
