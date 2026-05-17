# aget Session Wrapper PoC Spec

## Product Spec

### Summary

`aget` is a local-first session and extraction tool for agents. Given a URL, visible browser surface, or saved session, it produces clean agent-ready markdown while keeping authenticated content and credential-equivalent session material local by default.

The PoC should be a thin wrapper over existing tools:

- `agent-browser` for Chrome profile snapshotting and browser state export.
- Crawl4AI for browser-rendered markdown extraction from Playwright-compatible state.
- cmux as an optional integration for users already browsing in cmux panes.

The product value is not the extraction backend itself. The product value is safe, explicit, composable local session management for agent web access.

### Problem

Hosted URL-to-markdown tools are excellent for public pages but are the wrong default for authenticated/private content. Browser automation tools can interact with logged-in pages but usually expose raw browser state, lack markdown-quality output, or force awkward profile/login workflows.

Users need a local tool that can:

- Start with no ambient auth by default.
- Create independent named sessions.
- Import only approved session material from a browser or browser surface.
- Combine sessions intentionally for a request.
- Fetch authenticated pages locally as clean markdown.
- Avoid sending cookies, local storage, private HTML, screenshots, or extracted content to hosted services.

### Goals

- Provide an empty-session default: `aget get <url>` should use no saved cookies unless explicitly requested.
- Provide `aget <url>` as a shorthand alias for `aget get <url>`.
- Support independent named sessions: `google`, `facebook`, `hellointerview`, `github-work`, `github-personal`.
- Support combining sessions per request: `aget get <url> --session google --session hellointerview`.
- Support creating a new session from a combination: `aget session compose hi-oauth --session google --session hellointerview`.
- Support OAuth-style workflows where provider state is useful during login but can be excluded later.
- Support users with and without cmux.
- Support agent-safe non-interactive operation by default.
- Support bounded runtime with configurable timeouts.
- Allow passing extraction options through to the backend without baking every extractor feature into the top-level command model.
- Treat all session state as credential-equivalent bearer material.
- Keep v1 as a thin wrapper with a clear migration path to owned v2 internals.

### Non-Goals For PoC

- Do not implement a custom browser engine or crawler.
- Do not implement multi-step site action APIs such as search, add-to-cart, posting, checkout, or account mutation.
- Do not bypass paywalls, access controls, anti-bot systems, or site policy.
- Do not automate credential entry.
- Do not send authenticated content or session state to hosted extraction services.
- Do not depend on cmux as a required runtime.
- Do not require users to use their primary browser profile by default.

### Core Concepts

#### Empty Session

The default fetch context contains no imported cookies or storage.

```bash
aget get https://example.com
```

This must not silently use Chrome, cmux, OS browser cookies, prior OAuth sessions, or any saved `aget` session.

#### Session

A session is a named local bundle of credential-equivalent browser state scoped by explicit domains and origins.

Examples:

```bash
aget session list
aget session inspect hellointerview
aget session delete hellointerview
```

Each session records:

- Name.
- Allowed cookie domains.
- Allowed storage origins.
- Cookie metadata, with values hidden by default in inspection.
- localStorage/sessionStorage entries when imported.
- Source: `cmux`, `agent-browser`, `chrome-profile`, `manual`, or `composed`.
- Creation and last-used timestamps.
- Sensitivity flags.
- Optional expiry hints.

#### Session Combination

Requests may use multiple sessions.

```bash
aget get https://www.hellointerview.com/dashboard \
  --session google \
  --session hellointerview
```

`aget` combines the selected sessions into a temporary replay state for the extraction backend. It does not mutate the source sessions unless explicitly requested.

Conflicts are explicit:

- Same cookie name/domain/path from multiple sessions is a conflict.
- Same storage origin/key from multiple sessions is a conflict.
- Default behavior should fail with a clear message.
- Later versions may support `--prefer-session <name>`.

#### Composed Session

A composed session is a named session created from other sessions.

```bash
aget session compose hi-oauth \
  --session google \
  --session hellointerview
```

This is useful for OAuth provider plus relying-party flows. Composition should preserve source provenance for each cookie/origin.

#### Provider Sessions

Provider sessions, such as `google` or `facebook`, are useful during login but often should not be used for normal crawling after the relying-party app session has been established.

The UX should make this distinction visible:

```bash
aget session inspect hi-oauth
```

Example output:

```text
Session: hi-oauth
Includes:
- google: accounts.google.com, .google.com
- hellointerview: www.hellointerview.com

Warning: includes OAuth provider session material.
Recommended after login: aget session test hellointerview <url>, then prune provider domains if no longer needed.
```

### Primary User Journeys

#### Public Fetch

```bash
aget get https://example.com
```

Equivalent shorthand:

```bash
aget https://example.com
```

Expected behavior:

- Uses empty session.
- Fetches/render-extracts locally.
- Outputs markdown.
- Records no auth state.

#### Import From cmux Browser Pane

For users already browsing in cmux:

```bash
aget session import cmux \
  --surface surface:8 \
  --name hellointerview \
  --domain www.hellointerview.com
```

Expected behavior:

- Reads cookies from the cmux browser surface.
- Uses explicit `--domain` filters, not URL-only filtering.
- Post-filters returned cookies against the allowlist.
- Optionally reads local/session storage for explicit origins.
- Saves a scoped local session.
- Does not call `cmux browser state save` by default.

#### Import From Existing Chrome Login

For users without cmux:

```bash
aget session import chrome \
  --profile Default \
  --name hellointerview \
  --domain www.hellointerview.com
```

Expected behavior in v1:

- Uses `agent-browser --profile Default` as the bridge.
- Exports decrypted state with `agent-browser state save` into a temporary file.
- Filters that state to the requested domains/origins.
- Stores only the scoped session in `aget` storage.
- Deletes the raw broad temporary state immediately.
- If Chrome must be quit for a clean snapshot, stops and asks the user rather than closing Chrome.

#### Fetch With One Session

```bash
aget get https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals \
  --session hellointerview
```

Expected behavior:

- Builds a temporary Playwright state file from the selected session.
- Runs Crawl4AI locally with that state.
- Deletes the temporary state file.
- Writes authenticated markdown to local output.
- Marks output as sensitive.

Extractor options can reduce or reshape output:

```bash
aget get https://www.hellointerview.com/... \
  --session hellointerview \
  --format markdown \
  --selector main \
  --max-chars 8000 \
  --wait-for "Video Content"
```

#### Fetch With Combined Sessions

```bash
aget get https://www.hellointerview.com/login \
  --session google \
  --session hellointerview
```

Expected behavior:

- Combines sessions only for this run.
- Shows a warning if provider domains are included.
- Does not persist the combination unless `session compose` is used.

#### OAuth Cleanup

After a login flow:

```bash
aget session test hellointerview https://www.hellointerview.com/dashboard
aget session prune hellointerview --drop-provider-domains
```

Expected behavior:

- Verifies whether the relying-party session works without provider cookies.
- Removes provider domains only when explicitly requested.

### UX Principles

- No ambient auth: auth is always selected explicitly.
- No surprise browser lifecycle changes.
- No broad session exports by default.
- Non-interactive by default so agents can call commands reliably.
- Any command that may require user action must fail with a clear machine-readable reason unless `--interactive` or a dedicated interactive subcommand is used.
- No cookie values in normal logs.
- Sensitive temporary files are created in OS temp storage and deleted immediately.
- Authenticated outputs are sensitive too.
- Every run should explain which sessions were used and which domains were allowed.
- Every imported session should be inspectable without revealing values.
- Timeouts should be explicit and configurable; no command should hang indefinitely.

### Security And Privacy Requirements

- Treat cookies, localStorage, sessionStorage, and exported browser state as credential-equivalent.
- Store persisted session material outside project repos by default.
- Use restrictive filesystem permissions for session files.
- Avoid writing raw broad state files except as short-lived temp files.
- Do not include session values in command output unless `--show-secrets` is explicitly passed.
- Do not commit session files or authenticated outputs.
- Keep hosted services out of authenticated fetch paths.
- Make provider-session inclusion explicit.
- Always post-filter imported cookies/storage against allowlists, even if the backend claims to filter.

## V1 Implementation Spec: Thin Wrapper

### Architecture

V1 is a CLI wrapper around existing backends.

```text
aget CLI
  session store
  session filter/composer
  temp Playwright state writer
  backend adapters
    cmux adapter
    agent-browser adapter
    Crawl4AI adapter
```

V1 should be boring and explicit. It should prove the UX and session model before replacing underlying utilities.

### Language

Use Rust if the project direction is still Rust, but do not block the PoC on Rust purity. A small CLI in Rust that shells out to existing tools is enough. A Python/Node prototype is acceptable only if clearly marked throwaway.

Recommended v1 shape:

- Rust CLI for product surface, filesystem layout, filtering, and process orchestration.
- Python helper script for Crawl4AI extraction if direct SDK use is simplest.
- External commands for `agent-browser` and `cmux`.

### Dependencies

Runtime tools:

- `npx -y agent-browser` for Chrome profile/session import.
- `uv run --with crawl4ai` or a bundled Python environment for Crawl4AI extraction.
- `cmux` optional, detected dynamically.

V1 should degrade cleanly:

- If cmux is unavailable, hide or error only on `session import cmux` and `current --from cmux`.
- If `agent-browser` is unavailable, error only on Chrome/agent-browser imports.
- If Crawl4AI is unavailable, fetch commands requiring rendered markdown should explain setup.

### Storage Layout

Default local storage:

```text
~/.aget/
  sessions/
    hellointerview.json
    google.json
  runs/
    <run-id>/
      content.md
      metadata.json
  tmp/
```

Session files should not live in the project repo.

V1 session file shape:

```json
{
  "version": 1,
  "name": "hellointerview",
  "source": {
    "type": "cmux",
    "surface": "surface:8"
  },
  "created_at": "2026-05-06T00:00:00Z",
  "updated_at": "2026-05-06T00:00:00Z",
  "sensitive": true,
  "allowed_cookie_domains": ["www.hellointerview.com"],
  "allowed_storage_origins": ["https://www.hellointerview.com"],
  "cookies": [
    {
      "name": "hi.session-token-2",
      "value": "...",
      "domain": "www.hellointerview.com",
      "path": "/",
      "expires": 1780671846,
      "httpOnly": true,
      "secure": true,
      "sameSite": "Lax",
      "source_session": "hellointerview"
    }
  ],
  "origins": [
    {
      "origin": "https://www.hellointerview.com",
      "localStorage": []
    }
  ]
}
```

V1 can store plaintext locally with restrictive permissions if needed for speed, but the CLI must mark this as a PoC limitation. Encryption at rest should be a near-term hardening task.

### Commands

All commands should support:

```bash
--json
--timeout <duration>
--verbose
--quiet
```

Commands that can require human login or browser confirmation should also support:

```bash
--interactive
```

Without `--interactive`, these commands should return a clear error such as `requires_user_action` and instructions for the user-facing command to run.

#### `aget get`

```bash
aget get <url> [--session <name> ...] [--out <path>] [--extractor crawl4ai]
aget <url> [--session <name> ...] [--out <path>] [--extractor crawl4ai]
```

Behavior:

- With no `--session`, uses empty state.
- With sessions, loads and composes them into a temp Playwright state.
- Runs Crawl4AI with `BrowserConfig(storage_state=<temp-state>)`.
- Deletes temp state.
- Writes markdown and metadata.

Top-level extractor options:

```bash
--format <markdown|html|text|json>
--selector <css>
--exclude-selector <css>
--wait-for <text-or-selector>
--max-chars <n>
--include-links
--include-images
--screenshot
--extractor-option <backend.key=value>
```

V1 does not need perfect support for all options across all extractors. It should normalize common options where feasible and forward backend-specific options through namespaced `--extractor-option backend.key=value` values.

#### `aget session import cmux`

```bash
aget session import cmux \
  --surface <surface> \
  --name <name> \
  --domain <domain>... \
  [--origin <origin>...]
```

Implementation:

- Run `cmux --json browser cookies get --surface <surface> --domain <domain>` for each domain.
- Never rely on `cmux --url` output without post-filtering.
- Normalize cookies to Playwright-compatible fields.
- Optional storage import can use `cmux browser storage local get`; if cmux cannot scope storage by origin, use the surface's current origin only and require explicit confirmation for additional origins.
- Save scoped session.
- Non-interactive by default. If the surface is missing, not a browser, or requires user navigation, return a structured error rather than opening a new browser unless explicitly requested.

Acceptance test:

- A local test page sets a cookie.
- Import by domain from cmux.
- `aget get <local-echo-url> --session <name>` shows the cookie in output.

#### `aget session import chrome`

```bash
aget session import chrome \
  --profile Default \
  --name <name> \
  --domain <domain>... \
  [--origin <origin>...]
```

Implementation:

- Create deterministic temporary `agent-browser` session name.
- Run `agent-browser --profile <profile> --session <tmp-session> open <probe-url>`.
- Run `agent-browser --session <tmp-session> state save <tmp-raw-state>`.
- Load raw state.
- Filter cookies and origins to explicit allowlists.
- Save scoped session.
- Delete raw state.
- Close only the named temporary `agent-browser` session.
- Never close the user's real Chrome.

Open caveat:

- If profile snapshot requires Chrome to be closed, V1 should stop and ask the user to quit Chrome manually.
- In non-interactive mode, this should return `requires_user_action` rather than prompting.

#### `aget session compose`

```bash
aget session compose <new-name> --session <name>...
```

Behavior:

- Loads source sessions.
- Detects conflicts.
- Writes a new session with provenance on each cookie/origin entry.

#### `aget session inspect`

```bash
aget session inspect <name> [--show-secrets]
```

Default output hides values:

```text
Session: hellointerview
Cookies:
- www.hellointerview.com hi.session-token-2 secure expires=2026-...
Storage origins:
- https://www.hellointerview.com localStorage keys=3
Warnings:
- sensitive bearer material
```

#### `aget session test`

```bash
aget session test <name> <url> [--must-contain <text>] [--must-not-contain <text>]
```

Behavior:

- Runs a local extraction using the session.
- Checks markers.
- Reports whether the session appears valid.

#### `aget session prune`

```bash
aget session prune <name> --drop-domain <domain>...
aget session prune <name> --drop-provider-domains
```

V1 can implement explicit `--drop-domain`; provider-domain heuristics can be simple or deferred.

### Crawl4AI Adapter

Use the proven helper shape from `workpads/research/benchmarks/crawl4ai_with_storage_state.py`.

The adapter should:

- Accept URL and temp state path.
- Configure `BrowserConfig(storage_state=<path>, channel="chrome" or default chromium)`.
- Use `CrawlerRunConfig(cache_mode=CacheMode.BYPASS, remove_overlay_elements=True)`.
- Return markdown, metadata, and success/failure.
- Map common `aget get` options to Crawl4AI config where possible.
- Pass unsupported or advanced Crawl4AI options through a backend-specific escape hatch.

V1 can shell out to:

```bash
uv run --with crawl4ai <helper.py> --url <url> --state <state> --output <out>
```

The helper should support output formats required by the CLI:

- `markdown` for normal agent context.
- `html` for debugging or later custom extraction.
- `text` for low-noise fallback.
- `json` for metadata plus selected content fields.

### Session Composition To Playwright State

Before extraction, selected sessions are combined into a temporary file:

```json
{
  "cookies": [],
  "origins": []
}
```

Rules:

- No selected sessions means empty `cookies` and `origins`.
- Multiple selected sessions are merged.
- Duplicates with identical value are deduplicated.
- Conflicting duplicates fail.
- Temporary file is deleted after backend exits.

### cmux Adapter Notes

Use cmux only when available and explicitly requested.

Known behavior from benchmark:

- `cmux --json browser cookies get --domain 127.0.0.1` returned scoped cookies correctly.
- `cmux --json browser cookies get --url http://127.0.0.1:9168` returned broad cookies in this environment.
- Therefore V1 must call per-domain export and still post-filter returned cookies.
- `cmux browser state save` should not be used by default because it exports broad state.

### agent-browser Adapter Notes

Known behavior from benchmark:

- `agent-browser --profile Default` can snapshot an authenticated Chrome profile after manual login.
- `agent-browser state save` exports decrypted Playwright-like state that Crawl4AI can consume.
- Raw exported state is broad and sensitive; use only as short-lived temp input.
- `agent-browser state load` did not successfully replay an externally-built scoped local cookie state in the local test. Do not rely on agent-browser as the replay backend in V1.

### Output Metadata

Each run should write metadata:

```json
{
  "url": "https://...",
  "created_at": "...",
  "sessions": ["hellointerview"],
  "extractor": "crawl4ai",
  "sensitive": true,
  "output": "content.md",
  "warnings": [
    "authenticated session used",
    "output may contain private content"
  ],
  "timing_ms": {
    "total": 1234,
    "extractor": 1000
  },
  "limits": {
    "max_chars": 8000,
    "truncated": false
  }
}
```

### Verification Plan

V1 must include at least these tests or scripts:

#### Unit Tests

- CLI parsing: `aget <url>` is equivalent to `aget get <url>`.
- Session file load/save redacts values in normal inspection.
- Cookie domain allowlist filtering accepts only permitted domains.
- Storage origin allowlist filtering accepts only permitted origins.
- Multiple sessions compose deterministically.
- Duplicate identical cookies deduplicate.
- Duplicate conflicting cookies fail.
- Provider-domain pruning removes only configured domains.
- Extractor options normalize into backend config.
- Timeout parsing and propagation works.

#### Integration Tests

- Empty fetch does not include test cookie.
- cmux domain-scoped import captures only the requested domain cookie.
- cmux imported cookie replays through Crawl4AI.
- Multiple sessions compose into one Playwright state.
- Conflict detection rejects duplicate cookie conflicts.
- Temp state files are deleted after extraction.
- `session inspect` redacts values by default.
- `aget get --format markdown|html|text|json` returns the expected shape where supported.
- `aget get --max-chars` truncates deterministically and records truncation metadata.
- Missing optional backend returns a structured actionable error.
- Non-interactive commands fail fast with `requires_user_action` when user action is needed.

Manual verification:

- Import an authenticated site session from Chrome using `agent-browser`.
- Fetch the authenticated page via Crawl4AI.
- Confirm logged-in markers are present and paywall/sign-in markers are absent.

#### E2E Tests

Use a local test web app rather than third-party sites for repeatable CI/e2e.

The test app should provide:

- Public page.
- Login page that sets app cookies.
- OAuth-like provider page on a separate host/port that sets provider cookies and redirects back.
- Authenticated dashboard.
- Page requiring app session only.
- Page requiring app plus provider session for testing composition.
- JS-rendered content page.
- Search endpoint and form, reserved for future action/API-mode tests.

E2E scenarios:

- `aget <public-url>` fetches with empty session.
- Import app session and fetch authenticated dashboard.
- Compose provider plus app sessions and fetch a page requiring both.
- Prune provider session and verify app-only page still works.
- Verify broad state is never persisted after Chrome import.
- Verify temp Playwright state is removed after every run, including failures.
- Verify authenticated output metadata is marked sensitive.

CI should run unit and local integration tests on every change. E2E browser tests can run locally and in CI if dependencies are available; otherwise CI should clearly mark them skipped with a reason.

### Agent Execution Contract

Because `aget` is designed for agents, command behavior must be predictable:

- Default output should be concise human-readable text.
- `--json` should return machine-readable structured output with stable fields.
- Non-zero exit codes should distinguish categories: usage error, backend unavailable, timeout, requires user action, auth failed, extraction failed.
- Commands should never wait indefinitely for login, navigation, or page load.
- Timeouts should be configurable globally and per command.
- Commands should not open visible browser UI unless explicitly requested with `--interactive` or an explicit open/login command.
- Sensitive values should never appear in stdout/stderr unless `--show-secrets` is passed.

Suggested default timeouts:

- CLI command total: 60s.
- Page navigation: 30s.
- Extraction: 45s.
- Backend startup: 20s.
- Login/user action: not allowed in non-interactive commands.

## Future Product Direction: Site APIs And Actions

The session model should leave room for `aget` to become a safe local action/API layer for authenticated sites.

Examples:

- Search an online supermarket as the user.
- Add an item to cart.
- Post or comment on a user's behalf.
- Submit forms after explicit approval.
- Extract a reusable local “site API” from observed browser/network behavior.

This is out of scope for v1, but it should influence design now:

- Sessions must be composable and explicitly selected for each action.
- Mutating actions must be distinguishable from read-only extraction.
- Future tools need confirmation policies: dry-run, preview, require approval, execute.
- Network/API discovery should be local and provenance-rich.
- Stored site APIs should record required domains, session dependencies, request shapes, and risk level.
- Agent-facing commands should make side effects explicit.

Possible future commands:

```bash
aget api discover <site> --session <name>
aget api list <site>
aget api call <site.search> --session supermarket --param query="milk"
aget action preview <site.add_to_cart> --session supermarket --param sku=123
aget action run <site.add_to_cart> --session supermarket --param sku=123 --confirm
```

V1 should not build this, but it should avoid decisions that make it impossible. In particular, session provenance, selected-session metadata, non-interactive execution, JSON outputs, and read/write operation classification should exist early.

## V2 Direction: Owned Implementation

V2 should replace the wrapper internals gradually while preserving the same product model.

Likely v2 ownership areas:

- Native session store with encryption at rest.
- Direct browser/CDP integration for controlled profiles and current-tab extraction.
- Built-in scoped cookie and storage import from supported browsers.
- First-class OAuth flow assistant with temporary provider sessions.
- Owned extraction pipeline: rendered HTML to readability-pruned markdown.
- Token budgeting and objective/selector narrowing.
- Native structured extraction modes: markdown, text, HTML, JSON/schema.
- MCP/OpenCode integration.
- Safe action/API mode for selected sites, with explicit read/write classification and user approval policies.
- Better provenance: URL, final URL, selected DOM region, extraction strategy, session names, timestamps.

V2 should keep the same external principles:

- Empty session by default.
- Explicit named sessions.
- Composable sessions.
- Local-only authenticated content.
- Provider-session separation.
- No broad browser-state export unless explicitly requested.
