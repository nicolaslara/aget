# R12: Architecture, Boundary, And CLI

## Architecture Summary

Build v1 as a thin local wrapper that proves the product model before replacing internals. Rust owns the CLI, session model, filtering, composition, metadata, temp-file lifecycle, and process orchestration. Existing tools remain backend adapters:

- Crawl4AI is the v1 rendered extraction backend.
- `agent-browser` is the v1 Chrome profile/state acquisition backend.
- cmux is an optional v1 source for users who already browse inside cmux panes.

The first implementation slice should not attempt custom browser automation, crawling, map/search, current-tab browser extension work, or mutating site actions. It should prove that `aget` can fetch with an empty session, store a named scoped session, replay that session through Crawl4AI, and return local markdown with provenance and sensitivity metadata.

## MVP Boundary

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

## CLI Commands

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
