## V1 Command Surface

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

### `aget get`

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

### `aget session import cmux`

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

### `aget session import chrome`

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

### `aget session compose`

```bash
aget session compose <new-name> --session <name>...
```

Behavior:

- Loads source sessions.
- Detects conflicts.
- Writes a new session with provenance on each cookie/origin entry.

### `aget session inspect`

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

### `aget session test`

```bash
aget session test <name> <url> [--must-contain <text>] [--must-not-contain <text>]
```

Behavior:

- Runs a local extraction using the session.
- Checks markers.
- Reports whether the session appears valid.

### `aget session prune`

```bash
aget session prune <name> --drop-domain <domain>...
aget session prune <name> --drop-provider-domains
```

V1 can implement explicit `--drop-domain`; provider-domain heuristics can be simple or deferred.
