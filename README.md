# aget

`aget` is a local-first, auth-aware, agent-friendly URL-to-markdown CLI.

It is meant for getting clean, low-token page content into agent workflows without sending private browser state to a hosted service. The project is still in an early, experimental MVP bootstrap.

## Current Status

What works today:

- `aget get <url>`
- `aget <url>` as a shortcut alias
- `--json` / `--envelope` structured output
- `--out <path>`
- output shaping with `--format`, CSS selectors, wait conditions, and deterministic character limits
- repeated `--session <name>` flags for explicit named-session replay and composition
- empty-session default
- local run artifacts
- session list, inspect, and delete commands
- session compose for persisting a deterministic composed session from named sources
- experimental login start/finish/cancel for user-driven session bootstrap with caller-chosen session names
- optional cmux cookie import for explicitly allowed domains
- optional Chrome profile import through `agent-browser` for explicitly allowed domains
- the real demo script

See [`project.md`](./project.md) for the original product goal, [`AGENTS.md`](./AGENTS.md) for agent instructions, [`WORKING.md`](./WORKING.md) for the living workflow, and [`workpads/`](./workpads/) for active project notes.

## Prerequisites

- Rust and Cargo
- `uv`
- the default Crawl4AI/Playwright browser setup for the built-in backend
- optional: `cmux` for `aget session import cmux`
- optional: `agent-browser` for `aget session import chrome`

## Quick Start

Run the CLI from the repo root:

```bash
cargo run --quiet -- get https://example.com
cargo run --quiet -- https://example.com
cargo run --quiet -- get https://example.com --envelope
cargo run --quiet -- get https://example.com --out /tmp/example.md
cargo run --quiet -- get https://example.com --format text --selector main --max-chars 4000 --json
cargo run --quiet -- get https://example.com/account --session my-session --json
cargo run --quiet -- get https://example.com/account --session provider --session app --json
```

Session commands:

```bash
cargo run --quiet -- session list
cargo run --quiet -- session inspect <session-id>
cargo run --quiet -- session delete <session-id>
cargo run --quiet -- session compose <new-name> --session <name> [--session <name>...]
cargo run --quiet -- session login start <name> --url <login-or-target-url>
cargo run --quiet -- session login finish <name>
cargo run --quiet -- session login cancel <name>
cargo run --quiet -- session import cmux --surface <surface> --name <name> --domain <domain> [--domain <domain>...]
cargo run --quiet -- session import chrome --profile <profile> --name <name> --domain <domain> [--domain <domain>...]
```

Agent-driven authenticated markdown flow:

```bash
cargo run --quiet -- --json get "https://docs.example.com/account" --format markdown
cargo run --quiet -- --json session login start workdocs --url "https://docs.example.com/account"
# User completes the site login in the opened browser.
cargo run --quiet -- --json session login finish workdocs
cargo run --quiet -- --json get "https://docs.example.com/account" --session workdocs --format markdown --out /tmp/workdocs.md
```

`workdocs` is only a local session name chosen by the caller. `aget` does not ship site-specific login, paywall, or access-state detection; the calling agent interprets fetched content and decides whether to ask the user to log in or retry with a session.

## Real CLI Demo

The repo includes a real end-to-end demo script:

```bash
./scripts/demo_real_cli.sh
```

It exercises a static public page and a JS-rendered page using the real default backend.

## CLI Reference

Concise usage:

```text
aget get <url> [--session <name>...] [--json|--envelope] [--out <path>] [--timeout <seconds>]
              [--format <markdown|html|text|json>] [--selector <css>]
              [--exclude-selector <css>] [--wait-for <text-or-selector>]
              [--max-chars <n>]
              [--extractor-option <backend.key=value>...]
aget <url> [--session <name>...] [--json|--envelope] [--out <path>] [--timeout <seconds>]
           [--format <markdown|html|text|json>] [--selector <css>]
           [--exclude-selector <css>] [--wait-for <text-or-selector>]
           [--max-chars <n>]
           [--extractor-option <backend.key=value>...]
aget session list
aget session inspect <session-id>
aget session delete <session-id>
aget session compose <new-name> --session <name> [--session <name>...]
aget session login start <name> --url <login-or-target-url> [--profile <agent-browser-profile>]
aget session login finish <name>
aget session login cancel <name>
aget session import cmux --surface <surface> --name <name> --domain <domain> [--domain <domain>...]
aget session import chrome --profile <profile> --name <name> --domain <domain> [--domain <domain>...]
```

Notes:

- `aget get <url>` is the primary command.
- `aget <url>` is an alias for the same fetch path.
- `--envelope` prints the agent control-plane response envelope: `{ "ok": true, "command": "...", "data": {...}, "warnings": [], "timing_ms": {...} }` for success or `{ "ok": false, "command": "...", "error": {...} }` for failure. It does not change the fetched page content format. `--json` is kept as a compatibility alias for the same structured response mode.
- `--out` writes the extracted markdown to a file.
- `--format` requests `markdown`, `html`, `text`, or `json` page content from the extractor; markdown remains the default. For `text`, the Crawl4AI helper prefers extracted content and otherwise derives plain text from cleaned/raw HTML before falling back to markdown as a last resort.
- `--selector`, `--exclude-selector`, `--wait-for`, and repeated `--extractor-option backend.key=value` are forwarded to the Crawl4AI helper when supported. `--wait-for` is CSS-only in v1 for authenticated-session safety: use `css:<selector>` or a plain CSS selector; JavaScript waits are rejected. Supported Crawl4AI option keys use the `crawl4ai.` namespace: `crawl4ai.target_elements`, `crawl4ai.excluded_tags`, `crawl4ai.only_text`, `crawl4ai.word_count_threshold`, `crawl4ai.wait_until`, `crawl4ai.page_timeout`, `crawl4ai.wait_for_timeout`, `crawl4ai.delay_before_return_html`, and `crawl4ai.wait_for_images`; unsupported keys fail instead of being ignored. List values are comma-separated, booleans accept `true`/`false`, and numeric fields use integer or decimal values as appropriate.
- `--max-chars` truncates extracted content in Rust after backend extraction using Unicode scalar values; it never truncates the JSON response envelope.
- Repeated `--session` flags replay named local sessions for the request in the order provided. Cookie conflicts and same-origin localStorage key conflicts are rejected instead of preferring one session; disjoint localStorage keys for the same origin are merged.
- `aget session compose <new-name> --session <name>...` saves the same deterministic composition as a named local session, preserving cookie and storage-origin source provenance while redacting secret values in errors and inspect output by default.
- `aget session login start <name> --url <url>` opens a visible `aget`-owned `agent-browser` profile for user-driven login. It does not collect or script credentials. `finish` exports local browser state, persists only URL-scoped cookies/storage as a normal local session, then removes the raw temp state. `cancel` closes only the pending `aget` login session.
- `aget` is a generic fetcher. It returns page content and extraction outcomes; it does not detect site-specific paywalls, login walls, rate limits, or content quirks. Site-specific reasoning belongs to the calling agent or a future agent skill.
- `--timeout` sets the request timeout in seconds.
- `aget session import cmux` imports cookies from a cmux browser surface for explicitly allowed domains only; imported cookies are stored locally as a sensitive named session.
- cmux import reads raw cookie values from the selected local cmux surface. Use only disposable or user-authorized surfaces and domains.
- `aget session import chrome` uses `agent-browser` to snapshot a Chrome profile into a temporary local state file, filters cookies and storage by explicit `--domain` allowlists, stores only the scoped result, then deletes the raw temp state. Chrome may need to be quit manually if the profile is locked.

## Envelope Output

Success example:

```json
{
  "ok": true,
  "command": "get",
  "data": {
    "url": "https://example.com",
    "final_url": "https://example.com/",
    "format": "markdown",
    "extractor": "crawl4ai",
    "content": "# Example\n...",
    "artifacts": {
      "content": "/Users/me/.aget/runs/abc123/output.md",
      "metadata": "/Users/me/.aget/runs/abc123/metadata.json"
    },
    "sessions": [],
    "sensitive": false,
    "limits": {
      "max_chars": null,
      "truncated": false,
      "truncated_by": null,
      "content_chars_before_truncation": 13,
      "content_chars_after_truncation": 13
    },
    "output_options": {
      "format": "markdown",
      "selector": null,
      "exclude_selector": null,
      "wait_for": null,
      "extractor_options": {}
    }
  },
  "warnings": [],
  "timing_ms": {
    "total": 482
  }
}
```

Error example:

```json
{
  "ok": false,
  "command": "get",
  "error": {
    "code": "extraction_failed",
    "message": "Crawl4AI extraction failed"
  }
}
```

## Privacy and Local Storage

- `aget` starts with an empty session by default.
- Auth/session replay is opt-in per request with `--session <name>`.
- Run artifacts live under `~/.aget/runs`.
- Temporary browser/session state is local and should be cleaned up when no longer needed.
- Imported Chrome sessions and localStorage values are credential-equivalent bearer material. `session inspect` redacts values by default; use `--show-secrets` only when explicitly needed.
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
