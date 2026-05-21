# Knowledge Archive D1-D48: Foundation, MVP Architecture, And PoC

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

