# aget

`aget` is a local-first, auth-aware, agent-friendly URL-to-markdown CLI.

It is meant for getting clean, low-token page content into agent workflows without sending private browser state to a hosted service. The project is still in an early, experimental MVP bootstrap.

## Current Status

What works today:

- `aget get <url>`
- `aget <url>` as a shortcut alias
- `--json`
- `--out <path>`
- `--session <name>` for explicit named-session replay
- empty-session default
- local run artifacts
- session list, inspect, and delete commands
- optional cmux cookie import for explicitly allowed domains
- the real demo script

See [`project.md`](./project.md) for the original product goal, [`AGENTS.md`](./AGENTS.md) for agent instructions, [`WORKING.md`](./WORKING.md) for the living workflow, and [`workpads/`](./workpads/) for active project notes.

## Prerequisites

- Rust and Cargo
- `uv`
- the default Crawl4AI/Playwright browser setup for the built-in backend
- optional: `cmux` for `aget session import cmux`

## Quick Start

Run the CLI from the repo root:

```bash
cargo run --quiet -- get https://example.com
cargo run --quiet -- https://example.com
cargo run --quiet -- get https://example.com --json
cargo run --quiet -- get https://example.com --out /tmp/example.md
cargo run --quiet -- get https://example.com/account --session my-session --json
```

Session commands:

```bash
cargo run --quiet -- session list
cargo run --quiet -- session inspect <session-id>
cargo run --quiet -- session delete <session-id>
cargo run --quiet -- session import cmux --surface <surface> --name <name> --domain <domain> [--domain <domain>...]
```

## Real CLI Demo

The repo includes a real end-to-end demo script:

```bash
./scripts/demo_real_cli.sh
```

It exercises a static public page and a JS-rendered page using the real default backend.

## CLI Reference

Concise usage:

```text
aget get <url> [--session <name>] [--json] [--out <path>] [--timeout <seconds>]
aget <url> [--session <name>] [--json] [--out <path>] [--timeout <seconds>]
aget session list
aget session inspect <session-id>
aget session delete <session-id>
aget session import cmux --surface <surface> --name <name> --domain <domain> [--domain <domain>...]
```

Notes:

- `aget get <url>` is the primary command.
- `aget <url>` is an alias for the same fetch path.
- `--json` prints structured output.
- `--out` writes the extracted markdown to a file.
- `--session` explicitly replays one named local session for the request.
- `--timeout` sets the request timeout in seconds.
- `aget session import cmux` imports cookies from a cmux browser surface for explicitly allowed domains only; imported cookies are stored locally as a sensitive named session.
- cmux import reads raw cookie values from the selected local cmux surface. Use only disposable or user-authorized surfaces and domains.

## JSON Output

Example:

```json
{
  "ok": true,
  "url": "https://example.com",
  "final_url": "https://example.com/",
  "format": "markdown",
  "extractor": "crawl4ai",
  "content": "# Example\n...",
  "artifacts": {
    "markdown": "/Users/me/.aget/runs/abc123/output.md",
    "metadata": "/Users/me/.aget/runs/abc123/metadata.json"
  },
  "sessions": [],
  "sensitive": false,
  "warnings": [],
  "timing_ms": {
    "total": 482
  },
  "limits": {
    "max_chars": null,
    "max_tokens": null,
    "truncated": false
  }
}
```

## Privacy and Local Storage

- `aget` starts with an empty session by default.
- Auth/session replay is opt-in per request with `--session <name>`.
- Run artifacts live under `~/.aget/runs`.
- Temporary browser/session state is local and should be cleaned up when no longer needed.
- Only process content you are authorized to access.
- Do not use `aget` to bypass access controls, paywalls, or site policies.

## Development

```bash
cargo test
cargo fmt --check
git diff --check
bash -n scripts/demo_real_cli.sh
```

## Roadmap

The current MVP focuses on known-URL fetches, local browser-backed extraction, session inspection, and compact output. Future work may expand into broader crawling, tab-aware flows, and richer auth/session handling, but those are not part of the current README scope.

## License

The package declares MIT in `Cargo.toml`. This repository does not currently include a separate `LICENSE` file.
