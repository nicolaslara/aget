# R12: Config, Session, And Output

## Config Layout

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

## Session Store

Session files live in `~/.aget/sessions/<name>.json`. Each session is scoped by explicit cookie domains and storage origins. Values are hidden by default in CLI output. The session model should preserve provenance so composed sessions can later explain which source contributed each cookie/origin.

The store must support an empty state without reading any browser or saved session:

```json
{"cookies": [], "origins": []}
```

## Cache And Run Layout

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

## Output Schema

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
