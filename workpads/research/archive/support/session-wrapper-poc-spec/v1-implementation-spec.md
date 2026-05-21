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

