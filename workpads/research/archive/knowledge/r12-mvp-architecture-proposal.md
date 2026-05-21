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

