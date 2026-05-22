# aget

`aget` is a local-first, auth-aware, agent-friendly URL-to-markdown CLI.

It is meant for getting clean, low-token page content into agent workflows without sending private browser state to a hosted service.

`aget` has switched its default fetch, browser fallback, Chrome import, and login lifecycle paths to owned Rust implementations. Crawl4AI and `agent-browser` command adapters remain explicit compatibility/test surfaces, but they are no longer required for normal default-runtime use.

The owned-backend migration is not fully closed: final review evidence, deeper browser/profile parity, and OAuth-safe orchestration remain active workpad items.

## Current Status

What works today:

- `aget get <url>`
- `aget <url>` as a shortcut alias
- `aget current-tab --cdp-port <port> --allow-private-content` for explicitly approved local CDP tab extraction
- `--envelope json` structured output
- `--output <path>`
- output shaping with `--content-format`, CSS selectors, wait conditions, and deterministic character limits
- repeated `--session <name>` flags for explicit named-session replay and composition
- replay-time checks that reject sessions outside the requested URL's saved scope
- empty-session default
- local run artifacts
- session list, inspect, and delete commands
- session compose for persisting a deterministic composed session from named sources
- experimental login start/finish/cancel for user-driven session bootstrap with caller-chosen session names
- optional cmux cookie import for explicitly allowed domains
- optional Chrome profile import for explicitly allowed domains
- project skill guidance at `.cursor/skills/aget/SKILL.md`
- project-local OpenCode tools for CLI-backed fetch/session workflows
- the real demo script

See [`project.md`](./project.md) for the original product goal, [`AGENTS.md`](./AGENTS.md) for agent instructions, [`WORKING.md`](./WORKING.md) for the living workflow, and [`workpads/`](./workpads/) for active project notes.

## Prerequisites

- Rust and Cargo
- local Chrome/Chromium for JavaScript-rendered pages, Chrome import, and login flows
- optional: `cmux` for `aget session import cmux`
- optional compatibility: `uv`/Crawl4AI and `agent-browser` only when explicitly using command-backed adapters in development or tests through `AGET_CRAWL4AI_COMMAND` or `AGET_AGENT_BROWSER_COMMAND`

## Quick Start

Run the CLI from the repo root:

```bash
cargo run --quiet -- get https://example.com
cargo run --quiet -- https://example.com
cargo run --quiet -- get https://example.com --envelope json
cargo run --quiet -- get https://example.com --output /tmp/example.md
cargo run --quiet -- get https://example.com --content-format text --selector main --max-chars 4000 --envelope json
cargo run --quiet -- get https://example.com/account --session my-session --envelope json
cargo run --quiet -- get https://example.com/account --session provider --session app --envelope json
cargo run --quiet -- get 'raw:<html><body><main>Local HTML</main></body></html>'
cargo run --quiet -- current-tab --cdp-port 9222 --allow-private-content --envelope json --output /tmp/current-tab.md
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
cargo run --quiet -- session import cmux --surface <surface> --name <name> --allow-domain <domain> [--allow-domain <domain>...]
cargo run --quiet -- session import browser --browser chrome --browser-profile <profile> --name <name> --allow-domain <domain> [--allow-domain <domain>...]
cargo run --quiet -- session import chrome --chrome-profile <profile> --name <name> --allow-domain <domain> [--allow-domain <domain>...]
```

Agent-driven authenticated markdown flow:

```bash
cargo run --quiet -- --envelope json get "https://docs.example.com/account" --content-format markdown
cargo run --quiet -- --envelope json session import browser --browser chrome --browser-profile Default --name workdocs --allow-domain docs.example.com
cargo run --quiet -- --envelope json get "https://docs.example.com/account" --session workdocs --content-format markdown --output /tmp/workdocs.md
```

For OAuth-backed sites, prefer asking the user to sign in through their real browser and importing a scoped browser session. `session login start` remains an experimental fallback for non-OAuth or controlled flows and may be rejected by OAuth providers.

`workdocs` is only a local session name chosen by the caller. `aget` does not ship site-specific login, paywall, or access-state detection; the calling agent interprets fetched content and decides whether to ask the user to log in or retry with a session.

## Real CLI Demo

The repo includes a real end-to-end demo script:

```bash
./scripts/demo_real_cli.sh
```

It exercises a static public page and a JS-rendered page using the real default backend.

## Compatibility Backends

The default runtime does not automatically fall back to Crawl4AI or `agent-browser`. To compare against the old PoC dependencies or run compatibility tests, select them explicitly:

```bash
AGET_CRAWL4AI_COMMAND='uv run --with crawl4ai python scripts/crawl4ai_extract.py' \
  cargo run --quiet -- get https://example.com --envelope json

AGET_AGENT_BROWSER_COMMAND='npx -y agent-browser' \
  cargo run --quiet -- session import browser --browser chrome --browser-profile Default --name docs --allow-domain example.com
```

When these variables are unset, `aget` uses its owned Rust extractor and owned Chrome/CDP browser/session paths.

## CLI Reference

Concise usage:

```text
aget get <url> [--session <name>...] [--envelope <json|none>] [--output <path>] [--timeout <seconds>]
              [--content-format <markdown|html|text|json>] [--selector <css>]
              [--exclude-selector <css>] [--wait-for-selector <text-or-selector>]
              [--inline-content <auto|always|never>] [--max-chars <n>]
              [--backend-option <backend.key=value>...]
aget <url> [--session <name>...] [--envelope <json|none>] [--output <path>] [--timeout <seconds>]
           [--content-format <markdown|html|text|json>] [--selector <css>]
           [--exclude-selector <css>] [--wait-for-selector <text-or-selector>]
           [--inline-content <auto|always|never>] [--max-chars <n>]
           [--backend-option <backend.key=value>...]
aget current-tab --cdp-port <port> --allow-private-content [--envelope <json|none>] [--output <path>]
                 [--content-format <markdown|html|text|json>] [--selector <css>]
                 [--exclude-selector <css>] [--wait-for-selector <text-or-selector>]
                 [--inline-content <auto|always|never>] [--max-chars <n>]
                 [--backend-option <backend.key=value>...]
aget session list
aget session inspect <session-id>
aget session delete <session-id>
aget session compose <new-name> --session <name> [--session <name>...]
aget session login start <name> --url <login-or-target-url> [--profile <aget-profile-path>]
aget session login finish <name>
aget session login cancel <name>
aget session import cmux --surface <surface> --name <name> --allow-domain <domain> [--allow-domain <domain>...]
aget session import browser --browser <chrome|chromium|brave|edge|arc|firefox|safari> (--browser-profile <profile> | --profile-path <path>) --name <name> --allow-domain <domain> [--allow-domain <domain>...]
aget session import chrome --chrome-profile <profile> --name <name> --allow-domain <domain> [--allow-domain <domain>...]
```

Notes:

- `aget get <url>` is the primary command.
- `aget <url>` is an alias for the same fetch path.
- `aget current-tab` extracts the selected tab from an already-running local browser CDP endpoint. It requires both `--cdp-port` and `--allow-private-content`; it does not scan profiles or common ports, navigate, create tabs, or close the browser. Current-tab results are marked sensitive, so `--inline-content auto` omits `data.content` from JSON envelopes by default.
- `--envelope json` prints the agent control-plane response envelope: `{ "ok": true, "schema_version": "aget.envelope.v1", "command": "...", "data": {...}, "warnings": [], "timing_ms": {...} }` for success or `{ "ok": false, "schema_version": "aget.envelope.v1", "command": "...", "error": {...} }` for failure. It does not change the fetched page content format.
- `--output` writes the extracted markdown to a file.
- `--content-format` requests `markdown`, `html`, `text`, or `json` page content from the extractor; markdown remains the default.
- `--inline-content` controls whether `data.content` is embedded in the JSON envelope. `auto` includes content for non-sensitive fetches and omits it for session-backed/sensitive fetches by default. `always` embeds content explicitly; `never` returns artifact paths and metadata only.
- `aget get` accepts HTTP(S) URLs plus explicit local-content inputs `raw:`, `raw://`, and `file://`. Session replay remains scoped to HTTP(S) hosts.
- `--selector`, `--exclude-selector`, `--wait-for-selector`, and repeated `--backend-option backend.key=value` shape extraction. `--wait-for-selector` is CSS-only in v1 for authenticated-session safety: use `css:<selector>` or a plain CSS selector; JavaScript waits are rejected. `AgetExtractor` currently supports `crawl4ai.target_elements`, `crawl4ai.excluded_tags`, `crawl4ai.base_url`, `crawl4ai.exclude_all_images`, `crawl4ai.exclude_domains`, `crawl4ai.exclude_external_images`, `crawl4ai.exclude_external_links`, `crawl4ai.exclude_internal_links`, `crawl4ai.exclude_social_media_domains`, `crawl4ai.exclude_social_media_links`, `crawl4ai.only_text`, `crawl4ai.process_iframes`, `crawl4ai.remove_forms`, `crawl4ai.remove_overlay_elements`, `crawl4ai.keep_data_attributes`, `crawl4ai.word_count_threshold`, `crawl4ai.delay_before_return_html`, `crawl4ai.page_timeout`, `crawl4ai.wait_for_timeout`, `crawl4ai.wait_until` with `domcontentloaded`, `load`, or `networkidle`, `crawl4ai.wait_for_images`, `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, `crawl4ai.max_scroll_steps`, `crawl4ai.flatten_shadow_dom`, `crawl4ai.body_width`, `crawl4ai.bypass_tables`, `crawl4ai.close_quote`, `crawl4ai.default_image_alt`, `crawl4ai.emphasis_mark`, `crawl4ai.escape_backslash`, `crawl4ai.escape_dash`, `crawl4ai.escape_dot`, `crawl4ai.escape_plus`, `crawl4ai.escape_snob`, `crawl4ai.google_doc`, `crawl4ai.google_list_indent`, `crawl4ai.handle_code_in_pre`, `crawl4ai.hide_strikethrough`, `crawl4ai.ignore_emphasis`, `crawl4ai.ignore_images`, `crawl4ai.images_as_html`, `crawl4ai.images_to_alt`, `crawl4ai.images_with_size`, `crawl4ai.ignore_links`, `crawl4ai.inline_links`, `crawl4ai.links_each_paragraph`, `crawl4ai.ignore_mailto_links`, `crawl4ai.ignore_tables`, `crawl4ai.include_sup_sub`, `crawl4ai.mark_code`, `crawl4ai.open_quote`, `crawl4ai.pad_tables`, `crawl4ai.preserve_tags`, `crawl4ai.protect_links`, `crawl4ai.single_line_break`, `crawl4ai.skip_internal_links`, `crawl4ai.strong_mark`, `crawl4ai.ul_item_mark`, `crawl4ai.unicode_snob`, `crawl4ai.use_automatic_links`, `crawl4ai.wrap_links`, `crawl4ai.wrap_list_items`, and `crawl4ai.wrap_tables` for compatibility with existing callers; unsupported keys fail instead of being ignored.
- `--max-chars` truncates extracted content in Rust after backend extraction using Unicode scalar values; it never truncates the JSON response envelope.
- Repeated `--session` flags replay named local sessions for the request in the order provided. Cookie conflicts and same-origin localStorage key conflicts are rejected instead of preferring one session; disjoint localStorage keys for the same origin are merged.
- `aget session compose <new-name> --session <name>...` saves the same deterministic composition as a named local session, preserving cookie and storage-origin source provenance while redacting secret values in errors and inspect output by default.
- `aget session login start <name> --url <url>` opens a visible `aget`-owned Chrome profile for user-driven login. It does not collect or script credentials. `finish` exports local browser state, persists only URL-scoped cookies/storage as a normal local session, then removes the raw temp state. `cancel` closes only the pending `aget` login session.
- `aget` is a generic fetcher. It returns page content and extraction outcomes; it does not detect site-specific paywalls, login walls, rate limits, or content quirks. Site-specific reasoning belongs to the calling agent or a future agent skill.
- `--timeout` sets the request timeout in seconds.
- `aget session import cmux` imports cookies from a cmux browser surface for explicitly allowed domains only; imported cookies are stored locally as a sensitive named session.
- cmux import reads raw cookie values from the selected local cmux surface. Use only disposable or user-authorized surfaces and domains.
- `aget session import browser --browser chrome` is the browser-neutral import surface for the verified local Chrome/CDP path. It filters cookies and storage by explicit `--allow-domain` allowlists, stores only the scoped result, then deletes raw temp state. Other browser values currently return a structured `usage_error` instead of pretending unsupported imports are safe.
- `aget session import chrome` remains a Chrome-specific compatibility spelling for the same verified import path. Chrome may need to be quit manually if the profile is locked.

## Envelope Output

Success example:

```json
{
  "ok": true,
  "schema_version": "aget.envelope.v1",
  "command": "get",
  "data": {
    "url": "https://example.com",
    "final_url": "https://example.com/",
    "content_format": "markdown",
    "extractor": "aget-owned-extractor",
    "content": "# Example\n...",
    "page_metadata": {
      "title": "Example",
      "description": "Example page"
    },
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
      "content_format": "markdown",
      "selector": null,
      "exclude_selector": null,
      "wait_for_selector": null,
      "backend_options": {}
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
  "schema_version": "aget.envelope.v1",
  "command": "get",
  "error": {
    "code": "extraction_failed",
    "message": "aget-owned extraction failed"
  }
}
```

## Privacy and Local Storage

- `aget` starts with an empty session by default.
- Auth/session replay is opt-in per request with `--session <name>`.
- Session replay is rejected when the selected session is not scoped to the requested host.
- Current-tab extraction is opt-in per request with an explicit local CDP port and private-content acknowledgement. It may read authenticated browser content from the selected tab, so prefer `--output` and avoid `--inline-content always` unless the user explicitly wants that content embedded in an envelope.
- Run artifacts live under `~/.aget/runs`.
- Temporary browser/session state is local. `aget` removes normal temp state on success/failure, sweeps old orphaned raw-state files on startup, and removes tool-owned login profiles after successful completion or cancel.
- Imported Chrome sessions and localStorage values are credential-equivalent bearer material. `session inspect` redacts values by default; use `--show-secrets` only when explicitly needed.
- Optional compatibility backend subprocesses run with a minimal environment instead of inheriting the full parent shell environment.
- Only process content you are authorized to access.
- Do not use `aget` to bypass access controls, paywalls, or site policies.

## OpenCode Integration

This repo includes project-local OpenCode custom tools in `.opencode/tools/aget.ts`. They call the local `aget` CLI with `--envelope json` and return the same structured envelopes as terminal usage.

Available tools:

- `aget_fetch`: fetch a URL, optionally with local sessions and output-shaping options.
- `aget_session_list`: list local session names.
- `aget_session_inspect`: inspect one local session with secret values redacted.
- `aget_session_import_chrome`: import a scoped session from a user-approved Chrome profile.

Install or build `aget` before starting OpenCode:

```bash
cargo install --path .
# or for development:
cargo build
AGET_OPENCODE_BIN="$PWD/target/debug/aget" opencode
```

Privacy notes:

- The tools do not read ambient browser auth. Authenticated fetches require explicit `sessions`.
- `aget_session_inspect` does not expose `--show-secrets`; inspect output stays redacted.
- Fetched authenticated content can still be sensitive. By default, `--inline-content auto` omits `data.content` for session-backed/sensitive fetches and returns artifact paths instead; use `inline_content: "always"` only when the user explicitly wants content embedded in the envelope.

## Development

```bash
cargo test
cargo fmt --check
git diff --check
bash -n scripts/demo_real_cli.sh
```

Testing layers:

- Use focused unit and fake-backend tests for narrow parser, envelope, subprocess, and redaction behavior.
- Use `tests/mock_site_cli.rs` and `tests/support/mock_site.rs` for deterministic e2e-style auth/session coverage without real credentials, real sites, or manual login.
- Use behavior-focused parity tests for the Crawl4AI and `agent-browser` features that `aget` actually depends on. These are local `aget` tests built from source inspection and mock fixtures, not copied upstream test suites.
- Keep ignored/manual real-site checks only for confidence that local backends still work against user-authorized live pages.

## Roadmap

The current MVP focuses on known-URL fetches, local browser-backed extraction, session inspection, and compact output. Future work may expand into broader crawling, tab-aware flows, and richer auth/session handling, but those are not part of the current README scope.

## License

The package declares MIT in `Cargo.toml`. This repository does not currently include a separate `LICENSE` file.
