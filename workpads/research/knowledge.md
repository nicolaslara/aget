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

The successful HelloInterview authenticated benchmark used regular Google Chrome for login, not an automation-controlled browser. After the user logged in manually and quit Chrome, `agent-browser --profile Default --session hi-auth-after-manual-chrome-login open <url>` snapshotted Chrome's authenticated `Default` profile and extracted local text/HTML. The result no longer contained sign-in or paywall CTAs and included video metadata/content plus the full premium article body. HelloInterview is the representative verification site here; the intended target class is any user-authorized gated page, including sites like FT and NYT.

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

The `get` command should accept common output-shaping options (`--format`, `--selector`, `--max-chars`, `--wait-for`) and an extractor-specific escape hatch so v1 can expose Crawl4AI features without freezing the final `aget` API too early.

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
--wait-for <text-or-selector>
--max-chars <n>
--extractor-option <backend.key=value>
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
    "content": "/Users/name/.aget/runs/run-id/content.md",
    "metadata": "/Users/name/.aget/runs/run-id/metadata.json"
  },
  "sessions": [],
  "sensitive": false,
  "warnings": [],
  "timing_ms": {"total": 1234},
  "limits": {"max_chars": null, "truncated": false}
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

### D24: I6 output shaping is Rust-owned where limits affect contract stability

I6 adds `aget get` output shaping flags for `--format markdown|html|text|json`, CSS include/exclude selectors, `--wait-for`, `--max-chars`, and repeated `--extractor-option backend.key=value`. Rust forwards only backend-supported options to the Crawl4AI helper: format, selector, exclude selector, wait condition, and extractor options. Earlier WIP accepted `--only-main` and `--max-tokens` as metadata-only flags, but those were later removed before OpenCode integration because they were not enforced.

`--wait-for` is intentionally CSS-only in v1 for authenticated-session safety. The helper accepts `css:<selector>` and plain CSS selector strings, but rejects `js:` waits and obvious JavaScript function syntax before importing or running Crawl4AI. This prevents user-supplied wait conditions from executing JavaScript in a browser context that may include replayed local session state.

The Crawl4AI helper treats extractor options as an explicit allowlist, not an arbitrary escape hatch. V1 supports `crawl4ai.target_elements`, `crawl4ai.excluded_tags`, `crawl4ai.only_text`, `crawl4ai.word_count_threshold`, `crawl4ai.wait_until`, `crawl4ai.page_timeout`, `crawl4ai.wait_for_timeout`, `crawl4ai.delay_before_return_html`, and `crawl4ai.wait_for_images` when the installed Crawl4AI config constructor accepts the key. Unknown or unnamespaced keys fail before importing or running Crawl4AI, so dangerous options such as `crawl4ai.js_code` are not silently ignored or executed.

Character truncation is enforced after backend extraction in Rust using `.chars()` so results are deterministic across backends and cannot cut a UTF-8 scalar in half. After a successful backend parse, Rust sanitizes the retained backend stdout capture so untruncated content is not left in the run directory when `--max-chars` later shortens final content. For `--format json`, the CLI still returns a complete JSON response envelope; only the extracted `content` string is truncated.

For `--format text`, the Crawl4AI helper now prefers `result.extracted_content`, then derives plain text from `cleaned_html` or raw `html` with a stdlib HTML parser, and only falls back to markdown if no HTML content is available. This keeps real backend text output from silently being markdown in the common no-`extracted_content` case.

The stable output metadata now includes `output_options` plus expanded `limits` fields: `truncated_by`, `content_chars_before_truncation`, and `content_chars_after_truncation`. This preserves the I4/I5 session/sensitivity behavior while giving agents enough metadata to decide whether to refetch with larger limits or a narrower selector.

### D25: Explicit copy/import is the MVP auth ownership model

R4a compared three auth/session ownership models: direct use of existing browser data, a dedicated `aget` browser/profile, and explicit copy/import into `aget`'s local session store. The MVP default should remain explicit copy/import into scoped `aget` sessions. Direct existing-browser use and dedicated login/profile flows are useful advanced modes, but they have larger consent, lifecycle, and reliability surfaces.

| Model | Fetch-time store | UX | Technical feasibility | Platform constraints | Privacy risk | Credential leakage risk | Profile lock/corruption risk | Auditability |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Direct existing browser data | User's live browser/profile or debug session at fetch time | Most seamless if already logged in; highest risk of surprising the user because the tool touches the active browser environment | Feasible through CDP/debug-port, WebDriver/BiDi, extension/native bridge, or tool-specific auto-connect; CDP is browser-version-dependent and not a stable testing API | Chrome user data directories are platform/channel-specific; Chrome remote debugging exposes full local browser control; Firefox profiles are OS-locked while in use; WebDriver BiDi is the standards path but still requires explicit remote-control setup | Highest, because every fetch may see ambient browser state unrelated to the target task | Highest, because a local automation endpoint or extension can expose cookies, storage, DOM, and private page content broadly | Medium to high if reusing profile directories directly; concurrent browser/profile use can fail or risk data loss across versions | Weak unless every direct access is explicitly logged with consent, target origin, profile/session identifier, and redacted result metadata |
| Dedicated `aget` browser/profile login | `aget`-owned profile at fetch time | Clear ownership once created; user logs in through a tool-owned browser/profile instead of normal browser | Feasible with Playwright persistent contexts, agent-browser persistent profile paths, or future Rust CDP/WebDriver adapter | OAuth/SSO can reject automation-controlled browsers; users may need visible login flows; cross-browser support varies; profile location and lifecycle are `aget`-owned | Medium, because data is isolated from the user's main browser but still broad within the profile | Medium, because profile data remains credential-equivalent and may include provider cookies/storage beyond the relying-party app | Low to medium if each profile is single-owner and not shared with other running browser processes | Strong: `aget` can name the profile, record consent, target domains/origins, and warn when provider domains are present |
| Explicit copy/import into `aget` storage | `aget`'s scoped local session JSON at fetch time | Slightly more explicit setup, but best agent UX afterward: `aget get <url> --session <name>` never reads ambient browser state | Proven locally via cmux domain cookie import and agent-browser state export feeding Crawl4AI; requires robust filtering from broad source state to explicit cookie domains/storage origins | Browser/profile export can require Chrome to be quit or remote debugging enabled; exported state files are plaintext unless encrypted; backend output formats differ and must be normalized | Lowest for normal fetches because only pre-approved scoped state is used; import step is the risky boundary | Medium at import time because raw exported state is credential-equivalent; low during fetch if raw state is filtered, redacted, and deleted | Low for persisted `aget` sessions because original profile is not reused at fetch time; import may still hit locks or incomplete snapshots | Strongest: every session records source, allowed domains/origins, sensitivity, and provenance; normal inspect redacts values |

Recommendation:

- MVP default: explicit copy/import into `aget` storage. Fetches use only `aget`'s scoped local session file plus temporary Playwright state. This matches the no-ambient-auth rule and makes the consent boundary auditable.
- Near-term implementation order: manual fixtures, cmux import, and agent-browser/Chrome import as explicit import commands. Import commands must post-filter by allowlist, delete raw broad state after success and failure, and return `requires_user_action` instead of closing or disturbing the user's browser.
- Advanced/deferred: direct CDP/auto-connect/current-browser access should be an explicit advanced mode, not a default fetch path. A dedicated `aget login/profile` flow is attractive after import works, but should wait until the product can open the exact target login URL, explain profile ownership, and avoid automating credentials.

Audit and logging rules for all models:

- Log metadata only: timestamp, command, source type, session/profile name or salted hash, requested domains/origins, target origin, result code, warning class, and whether sensitive data was used.
- Do not log raw cookies, storage values, access tokens, passwords, encryption keys, full private page content, unredacted provider identifiers, or raw browser-state file paths when they may reveal account names.
- Treat full URLs as potentially sensitive in authenticated contexts; record origin by default and keep full URL only in local run metadata when needed for reproducibility.
- Apply data minimization to import and audit records: collect only explicitly allowed domains/origins, retain raw broad exports for the shortest possible time, and support deletion/disposition of sessions and logs.

Open questions for the later security/privacy model:

- What encryption-at-rest boundary is required before broader use: only session files, or also run metadata, logs, cache, and temporary raw state?
- Should audit logs be a separate feature with retention controls, or should provenance stay embedded in session/run metadata for the MVP?
- Should authenticated run metadata record full URLs by default, origin-only by default, or configurable redacted URLs?
- What exact consent UX is required before direct CDP/auto-connect access to an existing browser, given that local debugging endpoints expose broad browser control?
- How should `aget` detect and report partial or stale imports caused by locked browser profiles without leaking profile paths or cookie names?

### D26: R4 persistent profile strategy favors explicit state import over live profile reuse

R4 compared Playwright persistent contexts, CDP attach, and WebDriver-based profile reuse for authenticated local extraction. The recommendation stays aligned with D25: the MVP should use explicit copy/import into `aget` storage for normal fetches, with `agent-browser`/Chrome import as an import-time bridge. Direct use of a live existing browser/profile should remain an advanced/deferred mode because it has the broadest control surface, browser/version sensitivity, and profile-locking risk.

| Strategy | Login reuse behavior | Risks | Platform constraints | Fit for `aget` |
| --- | --- | --- | --- | --- |
| Playwright persistent context | Uses an on-disk `userDataDir` and keeps cookies/local storage/profile data across launches. `storageState` can also export cookies, localStorage, and IndexedDB for replay into a fresh context, but Playwright does not persist `sessionStorage` through the storage-state API. | Persistent profile directories are credential-equivalent local state. Reusing a user's default Chrome profile is explicitly discouraged; shared mutable state can leak across tasks or break when tests/fetches mutate server-side state. | Browsers do not allow multiple running instances with the same user data directory. The safe pattern is a dedicated automation profile directory, not the user's main profile. | Good for a future `aget login/profile` flow where `aget` owns the profile lifecycle. For I7, exported storage state is the better bridge because `aget` can filter and persist only scoped session material. |
| CDP attach / Chrome remote debugging | Can attach to an already-running or explicitly launched Chromium/Chrome instance and inspect/control pages through DevTools Protocol WebSockets. It can observe the browser's live authenticated page state when the endpoint has access. | A CDP endpoint is effectively a live browser control/data endpoint for open pages. It exposes authenticated DOM/storage/cookies via debugging capabilities, is Chrome/CDP-version sensitive, and has a sharp consent boundary. | Chrome 136+ blocks `--remote-debugging-port`/`--remote-debugging-pipe` against the default Chrome data directory unless a non-standard `--user-data-dir` is supplied. User data directories contain cookies/history/bookmarks and use singleton/lock files; cross-version profile reuse can cause degraded behavior, crashes, or data loss. | Useful as an explicit advanced/current-browser mode later. Not the MVP default. It may be used indirectly by `agent-browser` during import, but `aget` should treat raw exported state as short-lived and filter it before persistence. |
| WebDriver / WebDriver BiDi | Standardizes browser automation sessions and, with BiDi, bidirectional messaging/subscriptions. Persistent login reuse is possible only through browser-specific launch options such as Chrome `--user-data-dir` or Firefox `-profile`. | Profile reuse inherits browser-profile risks while providing less direct, portable auth-state semantics than Playwright storage state. The spec does not standardize a safe persistent-profile/auth-state model. | WebDriver capabilities are portable for session creation, timeouts, proxy, and browser metadata, but profile persistence is vendor-specific. BiDi improves eventing/control but does not define profile persistence. | Good standards direction for future pure browser automation research, but not the shortest path for I7. It does not replace the current `agent-browser` plus filtered storage-state import plan. |

MVP auth/session strategy after R4:

- Keep `aget get` empty-session by default.
- Keep normal authenticated fetches on scoped `aget` session files, never ambient browser stores.
- Implement I7 as explicit Chrome import through `agent-browser`: create/use a named temporary agent-browser session, export raw state to a private temp file, filter by explicit domains/origins, save only scoped state, and delete the raw broad export on success and failure.
- If Chrome/profile state cannot be acquired cleanly, return `requires_user_action`; do not close or disturb the user's running browser.
- Defer a first-class `aget login/profile` persistent-context flow until the product can own a dedicated profile directory, open the exact target login URL, explain profile ownership, and document sessionStorage limitations.
- Defer direct CDP/current-browser attach until it has explicit consent UX, endpoint exposure warnings, and redacted audit metadata.

Confidence: High for the MVP direction. The recommendation is supported by primary Playwright, Chrome/Chromium, WebDriver, and Selenium sources, and it matches local benchmark evidence from D17/D18 plus the explicit ownership model from D25. Remaining uncertainty is implementation-specific: how reliably `agent-browser` can export state across Chrome profile lock/version conditions without requiring user action.

### D27: I7 Chrome import uses agent-browser only as a scoped acquisition backend

I7 adds `aget session import chrome --profile <profile> --name <name> --domain <domain>...`. The implementation generates a unique temporary agent-browser session name (`aget-import-<pid>-<timestamp>`), runs the equivalent of `agent-browser --profile <profile> --session <temp> open about:blank`, `agent-browser --session <temp> state save <raw-state-path>`, then `agent-browser --session <temp> close`, and never uses broad close/close-all. `AGET_AGENT_BROWSER_COMMAND` exists for fake-command tests and is executed directly with `Command::new`, not through a shell.

The raw agent-browser state is treated as bearer material because it can contain live cookies plus local/session storage. It is written only under `~/.aget/tmp` to a pre-created `0600` temp file, parsed as Playwright-compatible `{cookies, origins}`, filtered by the explicit domain allowlist, and deleted on success and failure. Persisted sessions contain only allowlisted cookies and localStorage origins whose parsed origin host matches the same exact/suffix rules used by cmux import; unfiltered raw state is never saved to `sessions/`. Conflicting duplicate cookies or same-origin localStorage keys are rejected instead of silently choosing one value; disjoint localStorage keys for the same origin can be merged during session composition.

Chrome/profile acquisition can fail when Chrome is still running, the profile is locked/in use, the user is not logged in, or no auth state is present for the allowlist. These cases map to stable `requires_user_action`; `aget` does not try to quit Chrome or automate login. Missing `agent-browser` maps to `backend_unavailable`, and malformed storage-state JSON maps to `extraction_failed`.

Manual verification commands for a user-authorized Chrome profile:

```bash
export AGET_HOME="$(mktemp -d)"
cargo run -- --json session import chrome --profile Default --name hi --domain hellointerview.com --domain www.hellointerview.com
cargo run -- session inspect hi
cargo run -- --json get "https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals" --session hi --timeout 60
```

If the import returns `requires_user_action`, manually quit Chrome after saving work and rerun the same import command. Do not run any command that closes Chrome on the user's behalf. After a successful manual import, inspect `~/.aget/tmp` and confirm no `agent-browser-raw-state-*.json` files remain; inspect the saved session only with `--show-secrets` if explicitly needed because values are live bearer material.

### D28: I8 multi-session composition keeps sessions explicit and provenance-preserving

I8 adds repeated `aget get <url> --session <name> --session <name>` support and `aget session compose <new-name> --session <name>...`. Request-time composition loads the named local sessions in flag order, builds one temporary Playwright storage-state file, returns the selected session names in JSON/metadata, and marks the run sensitive whenever any session is selected. The empty-session default remains unchanged.

Persisted composition saves a new `SessionSource::Composed { sessions }` session without mutating its sources. Cookie provenance is preserved when already present and filled from the contributing source session otherwise. Storage-origin provenance is preserved for single-source origins; multi-source origins keep merged localStorage entries but omit a single origin-level source because no one source owns the whole origin.

Conflict handling remains strict and redacted. Cookie conflicts report only name/domain/path, same-origin localStorage conflicts report only key/origin, and `session compose` rejects a target that matches any source or already exists because there is no `--force` flag. Disjoint localStorage keys for the same origin are merged deterministically so provider/app sessions can compose without losing separate storage entries.

Post-review hardening added generic session-backed backend failure errors plus artifact redaction for cookie/localStorage values that reached the backend, plain `session inspect` provenance output, target-overwrite rejection, and a local app/provider cookie-flow test. Verification passed `cargo test --test get_cli`, `cargo test --test session_cli`, full `cargo test`, `cargo fmt --check`, and `git diff --check`. LSP diagnostics remain limited by the known local rust-analyzer proc-macro version mismatch; cargo compile/tests are clean.

### D29: Response format and page content format are separate concepts

I8a keeps `--json` as a compatibility alias and adds `--envelope` as the clearer agent control-plane response flag. `--format` remains the fetched page content format. This means `aget --envelope get <url> --format markdown` should be read as: return structured status/error/artifact/session metadata to the caller, with markdown as the extracted page content. Human-facing `aget get <url>` still prints markdown directly by default.

The naming is still not perfect because `--format json` means JSON page content while `--json` remains accepted as a response-envelope alias. OpenCode integration should prefer `--envelope` and treat the structured response envelope as the behavior source of truth; a later API cleanup can still consider clearer content-format names if `--format json` proves confusing.

### D30: Agent-driven login bootstrap uses an explicit user-action loop

I8b implements a generic user-driven login bootstrap rather than a site-profile system. `aget session login start <name> --url <target>` opens a visible, `aget`-owned `agent-browser` profile/session at the target URL. The user completes the site's login manually in that browser. `aget session login finish <name>` then exports browser state, filters it to the URL-derived allowed domains, saves the scoped result as the normal local `<name>` session, and removes the raw temp state. `cancel` closes only the pending `aget` login session.

The flow deliberately does not script, collect, or store credentials, and it does not persist provider cookies by default. If the final relying-party session is insufficient without provider cookies, that should be treated as a product finding requiring explicit provider-session composition rather than silent broad state persistence. `aget get` no longer maps site-specific content markers to `requires_user_action`; agents must interpret fetched content and decide whether to start or retry a login/session flow.

### D31: I8b is blocked on manual real-site verification

Automated/local I8b validation passed after blocker fixes. Review found and fixes addressed HTTPS-only login URLs, duplicate pending starts, finish close failures, and pending cleanup ordering. Remaining blocker is the manual authorized HelloInterview e2e (`real_hellointerview_login_flow_fetches_paywalled_markdown`), which still needs local agent-browser/Crawl4AI setup plus explicit user go-ahead/login.

### D32: Manual agent-flow verification improved bootstrap handling but I8b remains blocked

This was the actual CLI flow an agent would use, not the ignored Rust test. `agent-browser` was not on PATH, so the run used a temporary wrapper at `/var/folders/3y/smwkyhkn7gdfw7rz8cnmd40r0000gn/T/opencode/aget-agent-browser-npx` around `npx -y agent-browser`; `npx -y agent-browser --version` returned `0.27.0`. Unauthenticated `aget --json get <HelloInterview URL> --format markdown --timeout 120` returned `extraction_failed` from Crawl4AI waiting for `body`, not `requires_user_action`.

`session login start hellointerview` initially failed because the bare profile `aget-hellointerview` was treated as a missing Chrome profile; the code now defaults to `AGET_HOME/tmp/agent-browser/aget-hellointerview`, and the real start/cancel smoke passes. `session login finish hellointerview` initially failed on real agent-browser state because cookie `expires` was a float; the parser now accepts floating expires in both login and Chrome import paths. After that fix, `session login finish hellointerview` succeeded and saved a local redacted session with 3 cookies and 1 storage origin.

A safe marker check in the opened agent-browser profile still found paywall/sign-in markers, so the browser was not actually authenticated during the forced continuation. Session-backed `aget --json get <URL> --session hellointerview --format markdown` still failed in Crawl4AI waiting for `body`; retry with `--wait-for html` and longer timeouts still failed waiting for `html`. I8b remains blocked: the login/start/finish mechanics are improved, but the final agent-flow acceptance has not passed.

### D33: Site-specific extraction behavior is out of scope for the binary

The review in `CLAUDE_REVIEW.md` identified that `aget get` had crossed the generic fetcher boundary by matching HelloInterview hostnames/content and returning a site-shaped login CTA. That behavior is out of scope for the binary even if HelloInterview remains a useful representative manual test site.

Decision: `aget` returns fetched content and generic extraction outcomes. It does not classify page content as a paywall/login wall for specific sites, and it does not name built-in sessions in retry advice. Calling agents or future skills decide whether a page's content means login is required and which caller-chosen session name to use.

Task tracking was updated to keep the partially implemented login bootstrap visible as `I8b`, add `I8b-followup` for removing site-specific coupling, add `I8a-followup` for response API stabilization before OpenCode integration, and add `I8d` for extractor/session-glue consolidation before `I9`.

### D34: I8a-followup stabilizes structured CLI output around one envelope

`--envelope` is now the preferred structured-output flag and `--json` remains a compatibility alias. All successful structured command output uses one agent-facing shape:

```json
{"ok": true, "command": "get", "data": {}, "warnings": [], "timing_ms": {"total": 0}}
```

Errors use the matching command-bearing shape:

```json
{"ok": false, "command": "get", "error": {"code": "extraction_failed", "message": "..."}}
```

Per-command payloads now live under `data`; cross-command control-plane fields stay at the top level. For `get`, backend/extraction warnings are promoted to top-level `warnings`, and the fetched page content plus artifacts, sessions, sensitivity, limits, and output options are under `data`. The on-disk run `metadata.json` format is unchanged for now because it is a run artifact rather than the CLI control-plane API.

Focused review found that parse-time errors were still Clap-formatted under `--json`/`--envelope`, and that command-bearing error output needed stronger tests. The fix now emits structured `usage_error` envelopes for parse/validation failures when structured output is requested, while preserving normal Clap help/version output and human-mode parse errors.

Verification updated CLI, get, and session tests to assert the envelope shape before OpenCode integration depends on it. Confidence: high for the CLI contract change, with the remaining product risk deferred to I10 around whether sensitive `get` content should be embedded inline in structured output by default.

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

### D49: OAuth login should prefer real user browsers and verify persisted auth

Manual release testing against Hello Interview showed three distinct auth behaviors:

- OAuth in an automation-controlled `agent-browser` window can be rejected by Google with "This browser or app may not be secure."
- Importing from the user's normal Chrome `Default` profile worked when that profile was already logged in: `aget session import chrome` captured scoped Hello Interview cookies, and the browser fallback extractor fetched the premium article.
- Opening Chrome with a fresh `--user-data-dir` under `AGET_HOME` created a dedicated profile directory and anonymous Hello Interview cookies, but the expected auth cookies (`hi.session-token-2`, `hi.csrf-token`, `hi.callback-url`) did not persist there after the attempted login. Reopening that profile still rendered the logged-out/paywalled page.

Do not treat a custom browser profile as the default login design until it has a proven persistence/import path. The safer product flow is: first detect/import usable existing browser auth, then if auth is missing warn the user that OAuth/user login is needed, suggest importing an existing OAuth session from the user's real browser/profile whenever possible, open the chosen real browser/profile only when user action is required, and verify persisted scoped auth before claiming login success. Dedicated `aget` profiles remain attractive for isolation, but need targeted research around browser choice, OAuth redirects, profile paths, lock handling, and state export before becoming the default.

### D50: API cleanup makes envelope, content format, and inline content explicit

The public CLI/API now separates three concepts that were previously overloaded:

- `--envelope <json|none>` controls the response envelope. `--json` is no longer part of the public API.
- `--content-format <markdown|html|text|json>` controls extracted page content format.
- `--inline-content <auto|always|never>` controls whether the extracted content is embedded in the JSON envelope.

The envelope includes `schema_version: "aget.envelope.v1"`, and `get` data now reports `content_format` instead of `format`. The default `inline-content=auto` omits `data.content` for session-backed/sensitive fetches while still writing local content artifacts, so agents have a safer default for authenticated pages. README now states clearly that `aget` is a proof of concept and that current backend dependencies are part of validating the workflow.

### D51: Current OAuth-safe workflow support is partial, not automatic

Current `aget` supports the building blocks for the desired workflow:

- Fetch first with an empty session and structured envelope output.
- Import a scoped Chrome profile session with explicit `--allow-domain` values.
- Return `requires_user_action` when Chrome/profile import cannot proceed cleanly, including locked-profile and no-auth-state cases.
- Fetch again with an explicit named session.
- Omit `data.content` by default for session-backed/sensitive JSON envelopes while keeping local content artifacts.
- Guide agents, through the project skill and OpenCode tool descriptions, to prefer real-browser session import for OAuth-backed sites.

It does **not** yet support the full workflow as a first-class product/API:

- There is no single `aget` command or tool that runs the complete decision tree: fetch unauthenticated, import a real browser session, verify scoped auth, open the user's chosen browser only if auth is missing, re-import, and verify again.
- Chrome import is the only implemented real-browser import surface. Browser-choice terminology is not designed beyond `--chrome-profile`, and there is no first-class Arc/Brave/Firefox/Safari/default-browser flow.
- Verification is agent-driven rather than `aget`-driven: the agent runs a follow-up fetch and interprets content. That respects the generic-fetcher boundary, but the product still needs a generic verification command or recipe that records whether a session was usable for a target URL without encoding site-specific login/paywall rules in the binary.
- `session login start` is still an automation-owned fallback, not the OAuth-safe default, and it should remain clearly secondary for OAuth-backed sites.
- Deterministic tests cover the pieces (`session import chrome`, profile-lock/no-state `requires_user_action`, session-backed fetch, sensitive inline-content defaults, and mocked-site protected fetch), but not the whole OAuth-safe decision tree as one agent workflow.

Target flow for agents:

1. Fetch the URL without a session and save content to an artifact.
2. If the result appears gated and the user authorizes access, ask which real browser/profile already has access.
3. Import only explicit allowed domains from that user-approved browser/profile into a named local session.
4. Verify by fetching the same target URL with that session, preferably writing content to an artifact and using `--inline-content auto`.
5. If import returns `requires_user_action` or verification still appears gated, ask the user to sign in through their real browser, then re-import and re-verify.
6. Use `session login start` only as an explicit fallback for non-OAuth or controlled flows where automation-owned login is acceptable.

Future public API direction:

- Keep current `aget session import chrome --chrome-profile <profile> --allow-domain <domain>` as the proven PoC path.
- Add an OAuth-safe orchestration layer, likely `aget session authorize` or `aget auth prepare`, that models the decision tree without classifying site-specific content itself.
- Add browser-choice terminology for user-facing flows: `--browser <default|chrome|brave|arc|edge|firefox|safari>`, `--browser-profile <name>`, and `--profile-path <path>`. Only expose import implementations that are actually supported; opening a real browser for user login can support more browsers earlier than state import.
- Add generic session verification, either as `aget session verify <name> --url <url>` or as a documented fetch recipe with caller-provided checks such as selectors or must-contain/must-not-contain text. Verification must remain generic and must not hardcode paywall/login rules.

Confidence: Medium-high. The assessment is grounded in current code, tests, README, and agent skill behavior. The remaining uncertainty is product design: how much of the decision tree belongs in `aget` versus host-agent guidance, and how to verify auth usability generically without introducing site-specific heuristics.

### D52: Dependency migration should proceed through parity-first backend replacement

The validated PoC can now be migrated away from required Crawl4AI and `agent-browser` runtime dependencies, but the migration should preserve the current public CLI/API and backend capability boundaries first. Task I19 is now an umbrella split into source staging, abstraction audit, parity tests, Crawl4AI feature porting, `agent-browser` feature porting, default-backend switch, PoC surface cleanup, and final review.

Source snapshots are local and gitignored:

- Crawl4AI: `references/repos/crawl4ai`, remote `https://github.com/unclecode/crawl4ai.git`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`, Apache-2.0.
- `agent-browser`: `references/repos/agent-browser`, remote `https://github.com/vercel-labs/agent-browser.git`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, Apache-2.0.

Both upstream projects are permissively licensed, so adapting small tests or code is possible if attribution/license obligations are handled. The safer default remains behavior-driven porting: inspect upstream source for implementation strategy, generate or adapt focused parity tests for the `aget` feature subset, then implement owned Rust backends behind the existing interfaces.

Current Crawl4AI-dependent `aget` feature inventory:

- `scripts/crawl4ai_extract.py` is the current Python bridge. It creates `BrowserConfig` with headless Chromium, storage-state input, viewport defaults, optional channel, and a small allowlist of namespaced `crawl4ai.*` options. It creates `CrawlerRunConfig` with cache bypass, CSS wait, overlay removal, optional selector/exclusion, writes private output/metadata files, and selects markdown/html/text/json content from Crawl4AI result fields.
- `src/extraction.rs` owns `ExtractorBackend`, `CommandExtractorBackend`, session-state composition, primary extraction, fallback trigger, final success/error metadata, timeout/error mapping, artifact writing, truncation metadata, redaction, and the constant extractor name `crawl4ai`.
- `src/cli.rs`, README, `.opencode/tools/aget.ts`, and `.cursor/skills/aget/SKILL.md` expose the current Crawl4AI-shaped option surface through `--content-format`, selectors, CSS-only wait, and namespaced backend options.
- Tests that define the parity baseline include `tests/get_cli.rs`, `tests/mock_site_cli.rs`, `tests/aget_api.rs`, `tests/cli.rs`, `tests/support/bin/aget_mock_backend.rs`, and the ignored real Crawl4AI replay test.

Current `agent-browser`-dependent `aget` feature inventory:

- `src/session/agent_browser.rs` owns command execution through `AGET_AGENT_BROWSER_COMMAND`, backend-unavailable/timeout handling, profile-lock/login-needed classification, raw Playwright-style state parsing, cookie/storage allowlist filtering, duplicate conflict detection, domain normalization, private temp files, and raw-state cleanup helpers.
- `src/session/chrome.rs` uses `agent-browser` to open a Chrome profile, save raw state, filter it into a scoped local session, detect no-auth state, close the temp session, and remove raw state.
- `src/session/login.rs` uses `agent-browser` to start visible login sessions, finish by saving/filtering state, close sessions, remove tool-owned login profiles, and cancel pending login state.
- `src/extraction.rs` uses `agent-browser` as the session-backed fallback when Crawl4AI cannot consume composed state, loading state into a temp browser profile, opening the URL, extracting body HTML/text, closing the session, and cleaning fallback profiles.
- `src/aget.rs` already hides these operations behind `BrowserAutomationBackend` and `BrowserFallbackBackend`; I19b should verify those traits are sufficient before new backend work starts.
- Tests that define the parity baseline include `tests/session_cli.rs`, `tests/get_cli.rs`, `tests/mock_site_cli.rs`, `tests/aget_api.rs`, `tests/support/bin/aget_mock_agent_browser.rs`, and the ignored real/manual agent-browser checks.

Upstream implementation paths to inspect before porting:

- Crawl4AI extraction: `crawl4ai/async_webcrawler.py`, `crawl4ai/async_configs.py`, `crawl4ai/browser_manager.py`, `crawl4ai/async_crawler_strategy.py`, `crawl4ai/markdown_generation_strategy.py`, `crawl4ai/content_scraping_strategy.py`, `crawl4ai/html2text/`, `tests/async/`, `tests/browser/`, and `tests/cli/`.
- `agent-browser` browser/session: `cli/src/commands.rs`, `cli/src/connection.rs`, `cli/src/native/browser.rs`, `cli/src/native/state.rs`, `cli/src/native/cookies.rs`, `cli/src/native/storage.rs`, `cli/src/native/cdp/`, `cli/src/native/e2e_tests.rs`, and `cli/tests/doctor_cli.rs`.

Risk notes for I19:

- Replacement must avoid importing Crawl4AI anti-bot/stealth/proxy escalation patterns that conflict with `aget`'s authorization-only boundary.
- Browser/session replacement is security-sensitive because cookies, localStorage, raw browser state, screenshots, stdout/stderr, and authenticated content are credential-equivalent or private artifacts.
- The standard suite should eventually pass with Crawl4AI and `agent-browser` absent from PATH, but command adapters should remain until homegrown backends pass equivalent parity coverage.
- Do not port an entrypoint from memory. For each behavior, inspect the upstream implementation snapshot first and record the source path used for inspiration.

Confidence: Medium. I19a now has local source snapshots and a concrete migration inventory, but no parity tests or implementation have started.

### D53: I19b keeps backend swaps static and adds structured state to extraction requests

I19b audited the current `Aget` backend boundaries against the migration inventory in D52. The existing static-dispatch design is still the right shape for the migration:

- `ExtractorBackend` owns the "URL plus composed session state to extracted content" capability.
- `BrowserAutomationBackend` owns login, Chrome/profile import, and login cancellation/finish flows.
- `BrowserFallbackBackend` owns session-backed browser extraction when the primary extractor fails.
- `SessionStoreBackend` owns local session persistence and lets API-style tests use a non-filesystem store.

The main abstraction leak was that `ExtractorRequest` and `BrowserFallbackRequest` only exposed `state_path`, forcing future in-process backends to parse the temporary Playwright storage-state file that exists for command adapters. The request structs now also carry a borrowed structured `PlaywrightState`, while keeping `state_path` for the command-backed Crawl4AI and `agent-browser` adapters. This lets the homegrown backends start from structured cookies/storage without changing the public CLI/API or removing command compatibility.

API-level swappability coverage now verifies:

- A custom non-command extractor can fetch through `Aget` using a custom in-memory session store.
- A custom non-command browser fallback can handle session-backed fallback extraction.
- A custom non-command browser automation backend can finish login, import a Chrome/profile session, and start/cancel login through `Aget` without shelling out.
- The custom extractor and fallback see both the compatibility state file and the structured composed state.

Deferred abstraction notes:

- `PlaywrightState` remains the internal session-state interchange format for now because current adapters and tests already use that shape. I19d/I19e may rename or wrap it if homegrown backends need a browser-neutral state model.
- `AgetWith` currently uses one browser backend value for both `BrowserAutomationBackend` and `BrowserFallbackBackend`. That matches the current `agent-browser` replacement scope; split values can be introduced later only if extraction fallback and login/import need different implementations.
- `get_url_with_backend` and `get_url_with_backends` still construct a filesystem `SessionStore` from `GetOptions.home`. API callers that need custom stores should use `AgetWith` or `get_url_with_session_store`; keeping the compatibility functions avoids widening this migration pass.

Validation:

- `cargo test --test aget_api`
- `cargo test`

Confidence: High for I19b. The change is narrow, tested, and keeps command-backed adapters available while removing the concrete file-only state dependency for future homegrown backends.

### D54: I19c parity matrix is behavior-driven, not upstream-test copying

I19c defines the dependency parity target for the features `aget` actually uses today. No upstream test code has been copied. Crawl4AI and `agent-browser` are both Apache-2.0 in the local snapshots, but the safer first pass is still behavior-driven parity using `MockSite`, checked-in mock tools, current command-adapter behavior, and existing ignored real-backend smoke tests.

Extractor parity matrix:

| Feature | Required parity | Deterministic coverage |
| --- | --- | --- |
| Public fetch | Fetch a URL without session state and write private content/metadata artifacts. | `tests/mock_site_cli.rs::documents_public_get_json_contract_for_agents`, `tests/get_cli.rs::get_json_success_writes_run_artifacts_with_empty_state` |
| Authenticated replay | Compose selected sessions into request state and replay cookies/localStorage only to matching scopes. | `tests/mock_site_cli.rs::mock_site_replays_cookie_and_storage_sessions`, `tests/mock_site_cli.rs::documents_session_compose_replay_and_scope_rejection_contract`, `tests/get_cli.rs::get_session_uses_named_session_state_and_marks_sensitive` |
| Content formats | Preserve markdown default plus text/html/json output contracts. | `tests/mock_site_cli.rs::backend_parity_covers_extractor_content_formats` |
| Selectors/exclusions/waits | Apply CSS selector, exclusion, and CSS-only wait behavior; reject JavaScript waits. | `tests/mock_site_cli.rs::mock_site_fetch_handles_redirect_output_shaping_and_waits`, `tests/get_cli.rs::get_real_helper_rejects_javascript_wait_before_crawl4ai_import` |
| Backend options | Preserve namespaced `crawl4ai.*` option validation until a replacement namespace is designed. | `tests/get_cli.rs::get_forwards_supported_output_options_and_records_limits`, `tests/get_cli.rs::get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import` |
| Final URL and warnings | Surface backend final URL and warnings in the stable envelope and metadata. | `tests/mock_site_cli.rs::documents_output_limits_out_file_and_warning_contract`, `tests/mock_site_cli.rs::documents_custom_site_routes_for_extraction_features` |
| Limits and artifacts | Truncate only final content, record limit metadata, write content and metadata artifacts. | `tests/mock_site_cli.rs::documents_output_limits_out_file_and_warning_contract`, `tests/get_cli.rs::get_json_format_truncates_only_content_not_response_envelope` |
| Failure mapping | Preserve malformed output, structured failure, missing backend, timeout, and descendant termination behavior. | `tests/get_cli.rs::get_nonzero_and_malformed_backend_results_are_extraction_failed`, `tests/get_cli.rs::missing_backend_returns_backend_unavailable`, `tests/get_cli.rs::get_timeout_returns_stable_error_and_error_metadata`, `tests/get_cli.rs::get_timeout_terminates_backend_descendants` |

Browser/session parity matrix:

| Feature | Required parity | Deterministic coverage |
| --- | --- | --- |
| Chrome/profile import | Import scoped browser state, filter by explicit allowed domains, save session, clean raw state. | `tests/mock_site_cli.rs::mock_site_imported_chrome_session_can_fetch_protected_page`, `tests/session_cli.rs::session_import_chrome_saves_filtered_state_and_cleans_raw_file` |
| Login start/finish/cancel | Open a user-visible login flow, finish by filtering state, cancel/cleanup pending flows. | `tests/mock_site_cli.rs::documents_session_lifecycle_contract_for_agents`, `tests/mock_site_cli.rs::mock_site_login_bootstrap_can_fetch_protected_page_without_manual_action`, `tests/session_cli.rs` login tests |
| State parsing/filtering | Parse Playwright-style cookies/localStorage, accept float cookie expiries, reject conflicts, preserve provenance. | `src/session/agent_browser.rs` unit tests, `tests/session_cli.rs::session_login_finish_saves_only_url_scoped_state_and_cleans_temp_files` |
| Requires-user-action classification | Preserve profile-lock, no-auth-state, and login-needed classification. | `tests/session_cli.rs::session_import_chrome_requires_user_action_for_profile_lock`, `tests/session_cli.rs::session_login_finish_rejects_provider_only_state_without_saving_session` |
| Fallback extraction | On session-backed primary extraction failure, load composed state into browser fallback, extract body content, close/cleanup. | `tests/get_cli.rs::get_session_backend_failure_uses_agent_browser_fallback_with_composed_state`, `tests/get_cli.rs::get_session_fallback_close_failure_preserves_original_sanitized_crawl4ai_error` |
| Redaction and temp safety | Redact state secrets from errors/artifacts and remove raw state/temp profiles. | `tests/get_cli.rs::get_session_backend_failure_redacts_state_secrets_from_errors_metadata_and_artifacts`, `tests/session_cli.rs` raw-state cleanup tests |

Optional real-backend comparison commands:

```bash
cargo test --test get_cli real_crawl4ai_replays_named_session_cookie -- --ignored
cargo test --test session_cli real_hellointerview_login_flow_fetches_paywalled_markdown -- --ignored
cargo test --test session_cli real_cmux_import_replays_loopback_cookie_through_crawl4ai -- --ignored
```

The first command is the direct Crawl4AI replay check. The second exercises the real `agent-browser` plus Crawl4AI authenticated flow and requires manual authorized login. The third remains useful for optional cmux-to-Crawl4AI replay comparison, but cmux is not one of the two replacement targets for I19.

Validation:

- `cargo test --test mock_site_cli backend_parity_covers_extractor_content_formats`

Confidence: Medium-high. The parity target is explicit and mostly backed by existing deterministic tests plus one new content-format parity test. Remaining risk is that I19d/I19e may reveal additional edge cases from upstream implementation inspection; those should extend this matrix before porting each specific behavior.

### D55: I19d starts with an owned static extractor after Crawl4AI source inspection

I19d inspected the local Crawl4AI snapshot before porting extraction behavior. Relevant source paths:

- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py`: `AsyncWebCrawler.arun` composes the high-level fetch pipeline: cache/robots checks, crawler strategy navigation, response handling, HTML processing, markdown/content selection, and result metadata.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`: the Playwright strategy owns browser/page/context acquisition, navigation, and post-load DOM content retrieval.
- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `BrowserConfig` and run configuration show that `aget`'s current adapter uses only a narrow subset: headless Chromium, storage-state input, viewport/channel, CSS waits, selectors/exclusions, and cache bypass.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: scraping/content cleanup is a separable stage after rendered HTML is available.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` and `crawl4ai/html2text/`: markdown generation is a distinct HTML-to-markdown layer after cleanup.

The porting map for `aget` should keep those layers separate:

1. Transport/render layer: fetch a URL with explicit session state.
2. HTML processing layer: select/exclude/wait against page HTML.
3. Content conversion layer: produce markdown/html/text/json.
4. Finalization layer: artifacts, warnings, final URL, truncation, and stable errors remain in `src/extraction.rs`.

Do not port Crawl4AI's anti-bot/proxy/stealth paths, arbitrary JavaScript waits, LLM extraction, cache policy surface, or broad crawler features into this migration slice. Those are either outside the current `aget` dependency contract or conflict with the authorization-only safety boundary.

First implementation slice:

- Added `OwnedExtractorBackend` behind `ExtractorBackend`.
- It is not the default runtime backend yet; I19f owns the default switch.
- It initially used no new third-party dependencies and copied no upstream Crawl4AI code. The later D56 slice adds Rust dependencies for transport and parsing while preserving behavior-driven porting.
- It supports deterministic local/static HTTP extraction: redirects, cookie replay from structured `PlaywrightState`, text/html/json/markdown-as-text output, simple tag/id/class/tag.class selectors, comma-separated simple exclusions, CSS-only wait validation, artifact writes, and owned-backend error metadata.
- It deliberately does not claim browser-rendered parity yet: HTTPS/TLS, JavaScript rendering, localStorage replay through page scripts, richer CSS selectors, Crawl4AI-quality markdown/readability, screenshots, and real browser timeouts remain open for the next I19d/I19e slices.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli backend_parity_covers_extractor_content_formats`
- `cargo test --test aget_api`
- `cargo test --test get_cli get_real_helper_rejects_javascript_wait_before_crawl4ai_import`

Confidence: Medium. The first owned backend slice is narrow, local-only, and tested without Crawl4AI or `agent-browser`, but I19d remains in progress because browser-rendered extraction and localStorage-backed authenticated replay are not yet owned.

### D56: I19d replaces ad hoc owned extractor internals with Rust transport and CSS parsing crates

The second owned-extractor slice replaces the first slice's hand-rolled HTTP/selector implementation with focused Rust crates while keeping the same `ExtractorBackend` boundary:

- `ureq` v3.3.0 for blocking HTTP(S), redirects, and global request timeouts. License: MIT OR Apache-2.0.
- `scraper` v0.27.0 for HTML5 parsing and CSS selector matching. License: ISC.
- `html5ever` v0.39.0 as a direct dependency only for the `TreeSink` trait needed to detach excluded nodes from `scraper`'s parsed tree. License: MIT OR Apache-2.0.

The owned extractor now supports HTTPS-capable transport at the Rust layer and richer CSS selectors than the first static slice, including descendant/child/not-class selectors that are relevant to current `--selector`, `--exclude-selector`, and CSS-only `--wait-for-selector` behavior. It still does not execute page JavaScript and therefore does not replace Crawl4AI's browser-rendered SPA behavior yet.

Safety notes:

- Cookie replay remains explicit and scoped through `PlaywrightState`; secure cookies are only sent to HTTPS URLs.
- JavaScript wait strings are still rejected before selector parsing.
- Backend-specific `crawl4ai.*` options remain unsupported by the owned extractor until an `aget`-owned option namespace is designed; command-backed compatibility remains available.
- No Crawl4AI source or tests were copied in this slice. Upstream Crawl4AI was used only for architecture/source inspection recorded in D55.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. It closes the ad hoc selector/transport gap in the static owned extractor, but I19d remains open for JavaScript-rendered extraction, localStorage replay through page scripts, and markdown/readability quality.

### D57: I19d adds a first owned HTML-to-markdown slice

Before porting markdown behavior, I19d inspected Crawl4AI's markdown and cleaned-HTML flow:

- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py` selects the HTML source for markdown generation (`cleaned_html`, `raw_html`, or `fit_html`) after scraping/content processing.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` uses `DefaultMarkdownGenerator` plus `CustomHTML2Text`, then optionally converts links to citations and produces filtered/fit markdown.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py` removes excluded tags/selectors before cleaned HTML reaches markdown generation.
- `references/repos/crawl4ai/tests/async/test_markdown_genertor.py` and `tests/regression/test_reg_content.py` cover links/citations, content filters, and selector/exclusion behavior at a higher quality bar than the first `aget`-owned slice.

The Rust slice keeps the same layer boundary without copying Crawl4AI code. `OwnedExtractorBackend` now renders `OutputFormat::Markdown` through a small in-process DOM renderer instead of aliasing markdown to normalized text. The renderer currently handles the static/documentation structures `aget` tests directly: headings, paragraphs, emphasis, links/images, unordered/ordered lists, inline code, fenced code blocks, and blockquotes. Text output is unchanged and still uses normalized text.

License and dependency notes:

- The `html2md` Rust crate was rejected for this project because `cargo info html2md` reports GPL-3.0+.
- `ego-tree` v0.11.0 is now a direct dependency so the renderer can traverse the `scraper` DOM explicitly. License: ISC.
- No Crawl4AI source or tests were copied. The local Crawl4AI snapshot was used only to identify source-layer behavior and quality targets.

Remaining markdown/readability gaps are deliberate follow-ups: Crawl4AI-style citations/references, GFM tables, cleaned-main-content/readability pruning, fit markdown, media/link metadata, and broader edge-case parity. Browser-rendered JavaScript and localStorage-backed replay are still separate I19d/I19e gaps.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium for this slice. It replaces the most obvious markdown-as-text gap with tested structural markdown, but it is not yet a full Crawl4AI-quality markdown/readability replacement.

### D58: I19e starts with owned session-backed fallback extraction

Before porting the first `agent-browser` behavior, I19e inspected the local `agent-browser` snapshot paths that implement the current `aget get` fallback shape:

- `references/repos/agent-browser/cli/src/commands.rs`: parses `open`, `state load`, `get html body`, `get text body`, and `close` command shapes.
- `references/repos/agent-browser/cli/src/native/state.rs`: loads Playwright-style storage state by setting cookies and navigating to storage origins before setting local/session storage through CDP.
- `references/repos/agent-browser/cli/src/native/browser.rs`: navigates with CDP, tracks final page URL/title, and extracts DOM content through runtime evaluation.
- `references/repos/agent-browser/cli/src/native/actions.rs` and `native/element.rs`: implement `get html <selector>` as selected element `innerHTML` and `get text <selector>` as selected element text.

The current command fallback in `aget` uses this narrow sequence only after a session-backed primary extraction failure: load composed state into a temporary browser profile, open the URL, read body HTML/text, close the browser session, and clean up temp profile state. The first owned I19e slice therefore adds `OwnedBrowserAutomationBackend` with an owned fallback extraction path for static cookie-backed pages. It reuses the I19d owned fetch/HTML processing pipeline, uses structured `PlaywrightState` directly rather than an `agent-browser` state file, defaults fallback extraction to `body` to match the command fallback shape, and returns extractor metadata as `aget-owned-browser-fallback`.

This is intentionally not the full `agent-browser` replacement. `OwnedBrowserAutomationBackend` returns explicit unsupported-capability errors for Chrome/profile import and login start/finish/cancel until a real CDP/profile implementation is ported. It also does not execute page JavaScript or apply localStorage through a browser context, so localStorage-backed fallback pages still depend on future CDP/browser work.

Validation:

- `cargo test --test mock_site_cli owned_browser_fallback_replays_cookie_backed_session_without_agent_browser`

Confidence: Medium. The slice removes an `agent-browser` dependency path for static cookie-backed fallback extraction and is covered by deterministic MockSite evidence, but the high-risk browser automation work remains open.

### D59: I19d resolves owned markdown links against the page base URL

Before porting this behavior, I19d inspected Crawl4AI's link handling in `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`. Crawl4AI's `DefaultMarkdownGenerator` passes a base URL into its HTML-to-markdown converter and resolves relative markdown links before building citation references. Its tests in `references/repos/crawl4ai/tests/async/test_markdown_genertor.py` cover relative links and image URLs against a supplied base URL.

The owned renderer now resolves link and image URLs with the Rust `url` crate. It uses the final fetched URL as the default markdown base and honors an HTML `<base href="...">` element before extraction, matching Crawl4AI's separation between cleaned HTML and markdown generation. This keeps markdown output agent-ready when a selected content block contains relative links.

License and dependency notes:

- `url` v2.5.8 is now a direct dependency for standards-based URL joining. License: MIT OR Apache-2.0.
- No Crawl4AI source or tests were copied; the deterministic MockSite route was authored in this repo from the observed behavior target.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. It closes a concrete markdown parity gap with primary-source behavior inspection and local deterministic coverage; citation/reference formatting and broader readability quality remain open.

### D60: I19e adds a minimal owned CDP renderer for localStorage-backed fallback

Before porting this behavior, I19e inspected `agent-browser`'s storage-state and CDP paths again:

- `references/repos/agent-browser/cli/src/native/state.rs` loads Playwright-style cookies first, then navigates to each storage origin and sets `localStorage`/`sessionStorage` through `Runtime.evaluate` before the target page is opened.
- `references/repos/agent-browser/cli/src/native/element.rs` extracts element HTML through CDP after resolving the selector.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` launches Chrome with a temporary user data directory, waits for `DevToolsActivePort`, and cleans up the temp profile after shutdown.

`OwnedBrowserAutomationBackend` now keeps the fast static fallback for cookie-only sessions but switches to a minimal owned Chrome/CDP renderer when composed state contains localStorage origins. The CDP path launches a temporary local Chrome profile, attaches to a page target, sets cookies with `Network.setCookies`, navigates each localStorage origin to set storage, opens the requested URL, optionally waits for a CSS selector through an internally generated `document.querySelector(...)` expression, then feeds the rendered document HTML back into the owned extraction/formatting pipeline. This preserves the safety rule that user-provided JavaScript waits are not executed; user input is still limited to CSS selectors and is JSON-quoted inside agent-owned CDP expressions. Owned Chrome profile temp dirs are removed on normal shutdown and are now included in orphan sweeping.

Dependency note:

- `tungstenite` v0.29.0 is now a direct dependency for the blocking local CDP WebSocket transport. License: MIT OR Apache-2.0.

Remaining I19e gaps are still substantial: Chrome/profile import, login start/finish/cancel lifecycle, current-tab attach, richer process diagnostics, screenshot/debug artifacts, and broader rendered-SPA parity. The new CDP path is intentionally scoped to fallback extraction with explicit session state and a throwaway profile.

Validation:

- `cargo test browser_cdp`
- `cargo test session::store::tests::orphan_sweep`
- `cargo test --test mock_site_cli owned_browser_fallback_replays_cookie_backed_session_without_agent_browser`
- `cargo test --test mock_site_cli owned_browser_fallback_renders_local_storage_backed_session_with_chrome -- --ignored` passed locally with system Chrome and confirmed localStorage-driven rendered DOM extraction.
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The CDP command construction and cookie-only fallback path are covered by deterministic tests, and the ignored local Chrome smoke test passed on this machine. Full migration confidence still requires more lifecycle/error-path coverage before switching defaults.

### D61: I19d uses owned CDP rendering for localStorage-backed primary extraction

The D60 renderer exposed a follow-up correctness gap: once `OwnedExtractorBackend` becomes default, a localStorage-backed request could otherwise return a static app shell successfully and never invoke browser fallback. I19d now routes owned primary extraction through the same temporary Chrome/CDP renderer whenever composed session state contains localStorage origins. Cookie-only extraction stays on the static HTTP path.

This still does not make every JavaScript-heavy cookie-backed page render through Chrome; there is no reliable generic signal for that yet. The new rule only covers the explicit structured-state case where static HTTP cannot replay localStorage at all.

Validation:

- `cargo test --test mock_site_cli owned_extractor_backend_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_browser_fallback_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The local Chrome smoke test proves rendered DOM extraction for the primary owned backend on this machine; broader rendered-JavaScript default policy remains an open I19d/I19f decision.

### D62: I19d retries owned extraction through CDP when CSS waits need rendered DOM

I19d now covers a second narrow rendered-JavaScript case without adding public API: when the owned static HTTP path cannot find a requested CSS `--wait-for-selector`, it retries the same extraction through the owned Chrome/CDP renderer. This mirrors the current Crawl4AI contract for CSS waits while preserving the existing safety boundary: JavaScript wait expressions are still rejected, and the only user input evaluated in Chrome is a JSON-quoted CSS selector passed to `document.querySelector(...)`.

This is deliberately not a blanket browser-rendering default. Static pages with matching selectors still stay on the faster HTTP path, and JavaScript-heavy pages without a wait selector remain a future policy/default decision for I19f.

Validation:

- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The ignored Chrome smoke test proves the delayed-DOM wait path locally, and the normal suite keeps static wait-selector behavior covered without requiring Chrome.

### D63: I19d adds first owned markdown table rendering

Before this slice, the owned markdown renderer collapsed HTML tables into plain text. Crawl4AI's markdown behavior and tests treat tables as part of the markdown-quality target, so the owned renderer now emits GitHub-flavored markdown tables for static table structures. Header rows are detected from `<th>` cells; tables without explicit headers use the first row as the markdown header. Cell content goes through the same inline renderer as normal text, so links are still resolved against the page base URL, and pipe characters inside cells are escaped.

This is not a full markdown/readability replacement yet. Remaining quality gaps still include captions, complex row/column spans, Crawl4AI-style citations/references, fit markdown, and broader main-content cleanup.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The static parity test now covers a table with links and literal pipe characters, but complex table semantics remain an explicit follow-up.

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

### D77: I19d ports `crawl4ai.only_text` to owned markdown rendering

The next safe Crawl4AI option to port was `crawl4ai.only_text`. Before changing the owned extractor, I19d inspected the upstream behavior in `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `only_text` replaces text-formatting inline tags from `ONLY_TEXT_ELIGIBLE_TAGS` with their text content, and `references/repos/crawl4ai/crawl4ai/config.py`, where that tag allowlist is defined. The local command helper already parsed this option as a boolean.

The owned extractor now parses `crawl4ai.only_text` with the same boolean spelling set used by the command helper (`true/false`, `1/0`, `yes/no`, `on/off`). When enabled, owned markdown rendering treats Crawl4AI's text-formatting inline tags such as `strong`, `em`, `code`, `span`, `mark`, and `time` as plain text while preserving structural markdown such as headings, lists, links, tables, and preformatted code blocks. This mirrors the safe part of Crawl4AI's option without adding JavaScript execution or broader cleanup policy.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, and `delay_before_return_html`. `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This is a narrow renderer option with deterministic fixture coverage; it does not change broader readability or rendered-page readiness behavior.

### D78: I19d ports `crawl4ai.page_timeout` and `crawl4ai.wait_for_timeout` to owned CDP rendering

The next render-control slice ports the timeout options that have clear browser-operation semantics in Crawl4AI. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `page_timeout` is an integer millisecond timeout for page operations and `wait_for_timeout` optionally overrides the timeout used for `wait_for`, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where `wait_for_timeout` falls back to `page_timeout` when absent.

The owned extractor now parses `crawl4ai.page_timeout` and `crawl4ai.wait_for_timeout` as non-negative integer millisecond values. `page_timeout` is applied to CDP page creation, domain enabling, state loading, navigation, and final DOM reads in `browser_cdp::render_page`; browser process startup still uses the outer `aget --timeout` budget. `wait_for_timeout` applies only to the CSS selector wait and falls back to the page timeout when absent. The CSS-only wait safety rule remains unchanged.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, and `wait_for_timeout`. `word_count_threshold`, `wait_until`, and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`

Confidence: Medium-high. The parser is deterministically covered, and the ignored Chrome smoke covers the positive rendered wait path with explicit page and wait timeouts. This does not add network-idle or image-readiness behavior.

### D79: I19d ports bounded `crawl4ai.wait_until` support to owned CDP rendering

The next render-control slice ports the part of `crawl4ai.wait_until` that has direct owned CDP semantics. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where the default `wait_until` is `domcontentloaded`, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI passes that value into Playwright navigation.

The owned renderer now accepts `crawl4ai.wait_until=domcontentloaded` and `crawl4ai.wait_until=load`. These map to CDP `Page.domContentEventFired` and `Page.loadEventFired` respectively. Unsupported values such as `networkidle` fail explicitly instead of being ignored, because the owned renderer does not yet implement network-idle tracking. The default owned behavior remains the existing load-event wait until a broader readiness policy is chosen.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, `wait_for_timeout`, and bounded `wait_until`. `word_count_threshold` and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`

Confidence: Medium-high. This covers the two CDP lifecycle events the owned renderer can currently prove. It intentionally does not claim Playwright `networkidle` parity.

### D80: I19d ports `crawl4ai.wait_for_images` to owned CDP rendering

The next render-readiness slice ports the safe part of `crawl4ai.wait_for_images`. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `wait_for_images` is a boolean navigation/timing option defaulting to false, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI waits for `domcontentloaded`, sleeps briefly, then checks that all `<img>` elements are complete with a one-second timeout. Crawl4AI logs a warning and continues if images do not finish.

The owned extractor now parses `crawl4ai.wait_for_images` as a boolean using the same boolean spelling set as other owned options. When enabled, it forces the owned CDP rendering path, waits up to one second for `Array.from(document.images).every((img) => img.complete)`, and continues with an agent-visible warning if the image wait times out. This remains a browser-readiness option only; it does not add image description extraction, screenshot capture, external image fetching outside the browser, or JavaScript waits from user input.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, `wait_for_timeout`, bounded `wait_until`, and `wait_for_images`. `word_count_threshold` remained unsupported at this slice until source inspection established whether the current Crawl4AI default scraper applies it to the behavior `aget` actually depends on.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_honors_wait_for_images_option_with_chrome -- --ignored`

Confidence: Medium. The behavior is source-faithful for the explicit image-completion wait and covered by a local Chrome smoke, but broader rendered-page readiness remains a larger I19d gap.

### D81: I19d accepts `crawl4ai.word_count_threshold` with current Crawl4AI default semantics

The last Crawl4AI helper option still rejected by the owned extractor was `crawl4ai.word_count_threshold`. Before changing the owned backend, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `CrawlerRunConfig` accepts and stores `word_count_threshold`, and `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where the default `LXMLWebScrapingStrategy._scrap` receives that parameter but the cleaned-content path currently calls `remove_empty_elements_fast(body, 1)` with a hardcoded threshold. The upstream regression tests also cover config serialization/defaults and browser-context reuse for varying `word_count_threshold`, not output pruning in the default markdown path.

The owned extractor now accepts and validates `crawl4ai.word_count_threshold` as an integer so existing command-helper callers can switch to the owned backend without hitting an unsupported-option failure. It intentionally does not use the value to prune output because that would be stricter than the inspected Crawl4AI default path. Unsupported backend options still fail explicitly.

At this point the owned backend accepts every namespaced Crawl4AI option that the PoC command helper allowed: `excluded_tags`, `target_elements`, `only_text`, `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, `delay_before_return_html`, and `wait_for_images`.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This closes the compatibility-option gap without inventing behavior upstream does not currently prove; the remaining I19d gaps are broader quality/readiness work rather than a helper-option mismatch.

### D82: I19d ports bounded `crawl4ai.wait_until=networkidle` support to owned CDP rendering

The next rendered-readiness slice completes the safe `wait_until` value set that the PoC helper exposed. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `CrawlerRunConfig.wait_until` is the navigation wait condition, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI passes that value into Playwright navigation before later image waits or HTML capture.

The owned CDP renderer now accepts `crawl4ai.wait_until=networkidle`. It sends `Page.navigate` without discarding interleaved CDP events, requires the navigation response plus `Page.domContentEventFired`, tracks same-target `Network.requestWillBeSent`, `Network.loadingFinished`, and `Network.loadingFailed` events, and considers the page idle after there are no in-flight tracked requests for 500 ms. This is a bounded CDP implementation of the Playwright concept, not a broader smart-readiness system: long-polling, websockets, service-worker behavior, virtual scrolling, and app-specific readiness still belong to later I19d work.

The supported owned `crawl4ai.wait_until` values are now `domcontentloaded`, `load`, and `networkidle`.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_honors_networkidle_wait_until_with_chrome -- --ignored`

Confidence: Medium. The local Chrome smoke proves the owned network-idle wait for a delayed same-origin fetch, but this should still be treated as bounded parity rather than full Playwright readiness equivalence.

### D83: I19e classifies Chrome profile-in-use startup as user action

The next owned browser/session parity slice tightens Chrome startup diagnostics for profile imports. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`, where `agent-browser` captures Chrome startup stderr, reports early `DevToolsActivePort` failures with stderr context, copies named profiles into temporary user-data-dir roots, and treats profile reuse/lock situations as launch failures that a caller can surface to the user.

`OwnedBrowserAutomationBackend` now captures Chrome stderr in a private temp file during CDP startup. If Chrome exits before writing `DevToolsActivePort` and stderr indicates a profile-in-use or singleton/lock condition, the owned backend returns `requires_user_action` instead of a generic backend failure. Non-user-action startup failures still keep their original error code but include relevant Chrome stderr lines for diagnosis. This improves owned Chrome/profile import parity without touching the user's running browser or adding ambient current-browser access.

Validation:

- `cargo fmt --check`
- `cargo test owned_session_import_chrome_classifies_profile_in_use_as_requires_user_action --test session_cli`
- `cargo test browser_cdp::tests::parses_devtools_active_port_file`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored`

Confidence: Medium-high. The new deterministic CLI test proves the classification and no-session-saved behavior with a fake Chrome executable. Real Chrome lock/keychain behavior still needs the ignored/manual smoke coverage tracked under I19e/I19h.

### D84: I19e covers scripted owned browser fallback rendering

The next I19e verification slice covers an `agent-browser` parity behavior without adding new production surface. The command-backed fallback path in `src/extraction.rs` loads composed state into a browser session, opens the requested URL, then reads body HTML/text from the rendered page. The upstream `agent-browser` navigation path in `references/repos/agent-browser/cli/src/native/browser.rs` waits for a CDP lifecycle event before callers request page content.

`tests/mock_site_cli.rs` now includes an ignored local-Chrome smoke proving the owned browser fallback renders a cookie-backed script-bearing page after the primary extractor fails. This specifically exercises the fallback path rather than the default owned primary extractor, and verifies the session cookie is replayed to the scripted page.

Validation:

- `cargo test --test mock_site_cli owned_browser_fallback_renders_cookie_backed_scripted_page_with_chrome -- --ignored`

Confidence: Medium-high for this parity slice. The smoke uses local Chrome and a deterministic local site, but broader SPA readiness, current-tab attach, and real logged-in profile/keychain behavior remain open.

### D85: I19e preserves owned browser sessionStorage state

The next I19e state-parity slice ports the sessionStorage behavior that was still only partially represented in `aget`. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/state.rs`, where `StorageState` includes per-origin `sessionStorage`, state export collects `sessionStorage` through `Runtime.evaluate`, and state load navigates to each origin before calling `sessionStorage.setItem(...)`.

`aget` now keeps sessionStorage as first-class scoped session state. `SessionOrigin` stores it beside localStorage, agent-browser/Playwright-state imports preserve it through the same allowlist filtering, composed state merges it with duplicate-key conflict checks, owned CDP fallback loading replays it after navigating to the origin, and CDP state export collects origins that contain either localStorage or sessionStorage. Live owned-login export now attaches to an existing non-internal page target before state export instead of creating a fresh tab, because sessionStorage is tab-scoped and would otherwise be lost. Normal session inspection and extraction redaction now treat sessionStorage values as credential-equivalent bearer material, while `--show-secrets` can still reveal them intentionally.

Validation:

- `cargo check`
- `cargo test session_storage`
- `cargo test --test session_cli session_import_chrome_saves_filtered_state_and_cleans_raw_file`
- `cargo test browser_cdp::tests::parses_origin_storage_runtime_value`
- `cargo test browser_cdp::tests::prefers_existing_non_internal_page_target`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored`
- `cargo test browser_cdp::tests::owned_login_browser_exports_state_from_headed_profile_and_closes -- --ignored`

Confidence: Medium-high. Deterministic tests cover import filtering, redaction, composition, JS expression quoting, target selection, and CDP result parsing. The local-Chrome ignored smokes cover persistent profile export plus live headed-login sessionStorage export, but current-tab attach and broader cross-platform browser-process behavior remain I19e follow-ups.

### D86: I19d matches Crawl4AI selector miss fallback

The next owned extraction parity slice tightens `css_selector` behavior. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `LXMLWebScrapingStrategy._scrap` builds a selected content wrapper when `css_selector` matches but falls back to the full parsed document when the selector has no matches or errors.

`OwnedExtractorBackend` now preserves that no-match behavior for `GetOptions.selector`: a valid selector with no matches falls back to the full document instead of returning `extraction_failed`. This is intentionally different from the default no-selector path, which still uses `aget`'s conservative main-content heuristic, and from `wait_for_selector`, which still must fail when the waited element is absent. The fixture verifies this by selecting a missing class on a page that has `main` plus header/footer; the result includes header and footer rather than using main-content cleanup.

Validation:

- `cargo fmt --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. The behavior is source-faithful for the concrete selector miss case and deterministically covered. Invalid selector handling and broader Crawl4AI cleaned-HTML/readability behavior remain separate I19d follow-ups.

### D87: I19d aligns owned cleaned-HTML tag removal

The next I19d output-shaping slice ports a small part of Crawl4AI's cleaned HTML contract. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `LXMLWebScrapingStrategy._scrap` removes `style`, `link`, `meta`, and `noscript` elements, then removes `script` elements, before serializing cleaned HTML.

`OwnedExtractorBackend` already removed `script`, `style`, and `noscript`; it now also removes `link` and `meta` before generating HTML/text/markdown/json output. This primarily affects `--content-format html`, where previously head/body metadata and preload/canonical links could leak into the cleaned output even though Crawl4AI would drop them. The fixture keeps `<title>` and visible body content while proving `meta`, `link`, `style`, `script`, and `noscript` are absent from owned HTML output.

Validation:

- `cargo fmt --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: High for this narrow cleanup slice. The behavior is directly source-backed and deterministically covered; broader media/link extraction metadata and full readability remain separate I19d work.

### D88: I19d prunes owned cleaned-HTML attributes like Crawl4AI

The next cleaned-output slice ports Crawl4AI's default attribute pruning. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `remove_unwanted_attributes_fast` clears every element's attributes except an important-attribute allowlist and keeps `data-*` only when `keep_data_attributes` is enabled, and `references/repos/crawl4ai/crawl4ai/config.py`, where `IMPORTANT_ATTRS` is `src`, `href`, `alt`, `title`, `width`, `height`, `class`, and `id`.

`OwnedExtractorBackend` now records the selected root/target element IDs before cleanup, then strips non-important attributes before serializing cleaned output. This preserves selector and `crawl4ai.target_elements` matching against original page attributes while making HTML output drop `data-*`, inline style, event handler, ARIA, and relation attributes by default. The fixture proves the important attributes remain on cleaned output, unwanted attributes are absent, and a selector can still match a `data-*` attribute that is later pruned from the serialized content.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for this attribute-pruning slice. The allowlist is source-backed, selection-before-cleanup is covered by a deterministic fixture, and no new backend option or authenticated-browser behavior was added.

### D89: I19d strips base64 image payloads from owned cleaned output

The next cleaned-output slice ports Crawl4AI's base64 image cleanup. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `LXMLWebScrapingStrategy` compiles `BASE64_PATTERN = data:image/[^;]+;base64,...` and, before empty-element and attribute cleanup, replaces matching `<img src="...">` payloads with an empty `src`.

`OwnedExtractorBackend` now blanks `src` on image elements whose value starts with the same `data:image/<mime>;base64,` shape before serialized cleaned output is produced. The markdown renderer also skips images whose cleaned `src` is empty, which prevents the owned URL resolver from turning an emptied image source into a page-URL image reference. The fixture proves base64 payload text is absent from HTML output and does not reappear in markdown image syntax.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for this narrow privacy/output-size slice. The behavior is source-backed, deterministic, and only removes inline image payloads from output; it does not fetch or interpret images.

### D90: I19d removes empty owned cleaned-HTML leaf elements

The next cleaned-output slice ports Crawl4AI's empty-element pruning. Before changing the owned path, I19d re-inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `remove_empty_elements_fast(root, 1)` walks descendants bottom-up after base64 image cleanup and before attribute pruning, removes childless elements with no words, skips a bypass tag set such as `a`, `img`, `br`, table cells/rows, and preserves whitespace-only descendants inside `pre`/`code`.

`OwnedExtractorBackend` now runs a bottom-up cleanup pass in the same order. It removes empty leaf elements, recomputing childlessness after earlier removals so empty wrappers can also disappear. It protects the selected root and `crawl4ai.target_elements` IDs so selector-driven extraction cannot fail by deleting the element it is about to serialize. The fixture proves empty wrapper/span elements are removed while empty anchors, breaks, table cells/rows, and whitespace-only code spans are kept.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and deterministic, with one deliberate guard: selected roots and target elements are preserved to keep the owned extractor's public selector contract stable.

### D91: I19d expands owned markdown tags from Crawl4AI CustomHTML2Text

The next markdown-quality slice ports a small source-backed subset of Crawl4AI's HTML-to-markdown behavior. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`, where the default generator feeds cleaned HTML into `CustomHTML2Text` with links/images/emphasis/code enabled, and `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text`/`CustomHTML2Text` handle horizontal rules, definition lists, strikethrough tags, quoted inline text, and `kbd`/`tt`/`code` as inline code.

`OwnedExtractorBackend` now renders `<hr>` as a markdown horizontal rule, `<dl>/<dt>/<dd>` as term lines with indented definitions, `<del>/<strike>/<s>` as strikethrough, `<kbd>/<tt>` as inline code, and `<q>` with quotes. `crawl4ai.only_text=true` still strips these inline decorations to plain text, matching the owned option's current contract.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: Medium-high. This is deterministic local markdown rendering backed by Crawl4AI's source behavior. It remains a bounded quality slice; nested list fidelity, richer readability scoring, and broader rendered-page readiness are still open I19d work.

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
