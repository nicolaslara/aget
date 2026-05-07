# aget Project

## Original Goal

`aget` is a local, auth-aware URL-to-agent-context tool inspired by `wget`, `curl.md`, and Firecrawl.

Project-wide working practices live in [`WORKING.md`](./WORKING.md). That document defines the `/next` loop, confidence assessment, and review-subagent expectations.

The core idea is: **agent get**. Given a URL, current browser tab, site section, or small interaction flow, `aget` should produce clean, low-token, agent-ready markdown or structured data while keeping authenticated/private content local by default.

## Problem

Existing tools solve parts of the problem:

- `curl.md` is excellent for public known-URL-to-markdown workflows, objective narrowing, token savings, and agent integration.
- Firecrawl is excellent for production web data workflows: scrape, search, crawl, map, interact, screenshots, structured extraction, and JavaScript rendering.
- Neither hosted tool is ideal when the content is behind a login and the user wants to reuse local browser authentication without sending cookies, headers, or private page content through a third-party server.

## Product Direction

Build a local-first tool that feels like `curl.md` for simple known URLs but can escalate toward Firecrawl-like capabilities when needed.

Initial command shape:

```bash
aget fetch https://example.com/docs
aget fetch https://example.com/account --browser
aget current-tab
aget map https://example.com/docs
aget crawl https://example.com/docs --limit 50
aget status
```

## Features To Preserve From curl.md

| Feature | Why It Matters For Agents |
| --- | --- |
| Simple URL/CLI UX | Agents need a dead-simple path: given URL, return markdown. |
| Low-token markdown output | Primary value is reducing context cost/noise. |
| Objective-based narrowing | Agent often needs a section or answer, not a whole page. |
| Keyword filtering | Helps focus long docs before consuming context. |
| Fast mode vs smart mode | Agents need both quick and higher-quality extraction paths. |
| Cache bypass / freshness flag | Useful for changelogs, pricing, and frequently updated docs. |
| Token-count / token-saved metadata | Lets agents decide whether to summarize, chunk, or fetch more. |
| Markdown by default, JSON optional | Markdown is best for direct context; JSON is better for tools. |
| OpenCode plugin tool | Agents should call `aget`, not hand-roll shell commands. |
| `webfetch` replacement option | Existing agent behavior can transparently improve. |
| Auth/status commands | User and agent need to know whether local auth is usable. |
| Browser/CLI/API symmetry | Same capability should work from OpenCode, terminal, and scripts. |

## Features To Preserve From Firecrawl

| Feature | Why It Matters For Agents |
| --- | --- |
| JavaScript rendering | Many logged-in/docs apps are SPAs. Plain HTTP fetch is insufficient. |
| Smart wait/load detection | Agents should not guess arbitrary sleeps. |
| Main-content extraction | Removes nav/sidebar/footer/noisy app chrome. |
| Multiple output formats | Markdown for context, HTML/debug/text/JSON for pipelines. |
| Metadata extraction | URL, title, status, canonical URL, language, timestamp, content type. |
| Screenshots | Useful when extraction fails or visual confirmation matters. |
| Actions/interact model | Logged-in flows often require click/search/modal/pagination. |
| Session/browser lifecycle | Required for authenticated local browsing. |
| Custom headers/cookies | Core requirement: send auth locally without hosted servers. |
| Crawl/map concepts | Agents often need a docs section, not just one URL. |
| Batch fetch | Agents often compare several docs/pricing/reference pages. |
| Structured extraction | Useful for tables, pricing, API fields, changelog entries. |
| Robots/policy controls | Important for safe defaults and auditability. |
| Cost analogue | For local Rust: track time, pages, tokens, bytes, browser minutes. |
| Output-to-file convention | Large results should be saved and read incrementally. |

## New Local/Auth Requirements

| Feature | Why It Matters |
| --- | --- |
| Local-only guarantee | Authenticated content must not leave the machine unless explicitly requested. |
| Persistent browser profiles | Login once, reuse session. |
| Existing Chrome/CDP attach | Optionally use a real already-logged-in browser session. |
| Dedicated browser profile | Safer default than automating the main browser profile. |
| Header injection | Support `Authorization`, `Cookie`, and enterprise headers from local config. |
| Cookie jar import/export | Reuse sessions without copying full browser profiles. |
| Current-tab extraction | Extract the page the user already opened. |
| Consent boundary | Clearly mark when reading authenticated/private content. |
| Redaction controls | Strip secrets, emails, tokens, account IDs before model exposure if configured. |
| Site profiles | Per-domain wait selectors, login URL, content selector, auth mode. |
| Provenance report | Source URL, auth mode, extraction method, timestamp, warnings. |

## Candidate Agent-Facing Tools

| Tool | Purpose |
| --- | --- |
| `aget_fetch` | Fetch one URL as markdown using HTTP or local browser auth. |
| `aget_video` | Convert a supported video URL, starting with YouTube, into agent-ready markdown from transcript, metadata, and optional chapter structure. |
| `aget_current_tab` | Extract the active local browser tab. |
| `aget_search_page` | Find sections in a fetched page by objective/keywords. |
| `aget_batch` | Fetch multiple URLs concurrently. |
| `aget_map` | Discover links from a page/site section. |
| `aget_crawl` | Crawl bounded docs/site sections. |
| `aget_interact` | Click/type/wait/extract for dynamic pages. |
| `aget_status` | Show browser/profile/auth/cache/tool status. |
| `aget_login` | Open login flow for a configured profile/site. |
| `aget_cache` | Inspect/clear cached local results. |

## Project Tasks

These are high-level project tasks. Executable research work lives in `workpads/research/tasks.md`; implementation tasks should be added after the research workpad produces an MVP technical direction.

### 📋 P1: Complete prior-art and feasibility research

Outcome:

- Identify similar existing projects and compare their product model, architecture, auth/session handling, licensing, and local-first fit.
- Decide whether `aget` should be built as a small local tool, a wrapper around an existing project, a fork, or a hybrid.
- Produce an MVP architecture recommendation before implementation begins.

### 📋 P2: Define auth/session product model

Outcome:

- Compare three login/session models: direct use of existing browser data, dedicated `aget` login/profile tracking, and explicit copy/import from the user's browser into `aget` storage.
- Define the consent prompts, status UX, provenance metadata, and local storage boundaries for each model.
- Choose the MVP default and mark advanced modes clearly.

### 📋 P3: Define security and privacy model

Outcome:

- Threat-model credential leakage, browser-profile access, local database compromise, agent exfiltration, logs/cache, debug artifacts, and accidental third-party transmission.
- Define security levels for auth handling, redaction, provenance, cache retention, and agent output.
- Decide which actions require explicit user consent and which should be blocked by default.

### 📋 P4: Prepare agent-assisted engineering workflow

Outcome:

- Keep `WORKING.md` current as the living workflow for `/next`, confidence assessment, and review-subagent usage.
- Define review checkpoints before implementation starts, including test adequacy, feature coverage, code quality, maintainability, architecture cohesion, and security/privacy.
- Decide where cross-project knowledge should live when it is broader than a single workpad.

### 📋 P5: Define MVP implementation plan

Outcome:

- Convert the chosen architecture into independently testable implementation tasks.
- Include CLI, browser/session handling, extraction pipeline, local storage, agent integration, and verification strategy.
- Include review-subagent checkpoints for tests, deliverables, code quality, maintainability, and security/privacy.

### 📋 P6: Build and verify MVP

Outcome:

- Implement the reviewed MVP plan only after research approval.
- Verify known-URL markdown, persistent login reuse, status reporting, provenance, and local-only behavior.

## Output Contract Sketch

```json
{
  "url": "https://example.com/docs",
  "finalUrl": "https://example.com/docs",
  "title": "Docs",
  "markdown": "# Docs\n...",
  "metadata": {
    "status": 200,
    "contentType": "text/html",
    "fetchedAt": "2026-05-06T00:00:00Z",
    "authMode": "browser-profile",
    "tokensEstimated": 4200,
    "tokensSavedEstimated": 18000,
    "cache": "MISS"
  },
  "links": [],
  "warnings": []
}
```

## Extraction Modes

| Mode | Behavior |
| --- | --- |
| `http` | Fast direct HTTP fetch, headers allowed. |
| `browser` | Render with persistent browser profile. |
| `current-tab` | Extract already-open browser page. |
| `readability` | Use main article extraction. |
| `selector` | Extract specific CSS selector. |
| `full` | Return full page markdown. |
| `objective` | Narrow to relevant sections. |
| `debug` | Include HTML/screenshot/trace metadata. |

## Later Media Inputs

| Input | Desired Behavior | Notes |
| --- | --- | --- |
| YouTube video | Produce markdown from title, channel, description, chapters, transcript/captions, links, and provenance metadata. | Later feature. Prefer official/available captions or user-accessible transcript data. Do not design this to bypass access controls, DRM, age gates, private video permissions, or platform policy. |

For video inputs, markdown should be useful to agents without requiring audio/video context by default. If transcript data is unavailable, `aget` should report that explicitly rather than fabricating a summary.

Recommended YouTube pipeline for later research:

1. Fetch YouTube's own captions first, using `youtube-transcript-api` or `yt-dlp` subtitle extraction where legally and technically appropriate.
2. If captions are unavailable or low quality, download audio with `yt-dlp` and transcribe locally.
3. Prefer local ASR by default. Candidate backends: `faster-whisper` for iteration speed, `whisper.cpp` for portable embedding/Rust integration, WhisperX for word timestamps or diarization, and `mlx-whisper` for Apple Silicon experiments.
4. Keep paid speech-to-text APIs as optional explicit fallbacks, not defaults.

Video markdown should preserve provenance: source URL, video ID, title/channel, transcript source (`manual-captions`, `auto-captions`, `local-asr`, or `api-asr`), language, timestamp coverage, model/backend if ASR was used, and warnings for missing/low-confidence sections.

## Rust Research Areas

| Need | Candidate Rust Options |
| --- | --- |
| HTTP | `reqwest` |
| HTML parse/select | `scraper`, `html5ever`, `kuchiki` |
| Markdown conversion | custom converter, `html2md`, or bindings if quality is enough |
| Browser automation | `chromiumoxide`, `fantoccini`, CDP client crates |
| YouTube metadata/captions | `youtube-transcript-api`, `yt-dlp` | Later feature. Need policy/license/reliability review before adopting. |
| Local ASR | `whisper.cpp`, `faster-whisper`, WhisperX, `mlx-whisper`, Vosk | Later feature. Prefer local default; compare portability, model size, quality, timestamps, diarization, Rust integration. |
| CLI | `clap` |
| JSON schemas/output | `serde`, `schemars` |
| Cache | filesystem, SQLite via `rusqlite`, or `sled` |
| Token estimate | `tiktoken-rs` |
| Config | TOML + `serde`, `figment`, or `config` |
| Concurrency | `tokio` |

## Priority

1. Known URL to local markdown.
2. Persistent browser profile auth.
3. Objective/keyword narrowing.
4. OpenCode plugin/tool.
5. Current-tab extraction.
6. Batch fetch and cache.
7. Interact/actions.
8. Map/crawl.
9. Structured extraction.
10. Screenshots/debug traces.
11. YouTube/video-to-markdown.

## Safety Boundary

`aget` is for content the user is authorized to access. It should not be designed to bypass paywalls, access controls, or site policies. Authenticated data handling must be explicit, local by default, and auditable.
