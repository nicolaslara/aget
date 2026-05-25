# aget

`aget` is a local-first CLI for turning web pages and explicitly approved
browser/session state into agent-ready content.

The current product surface is the Rust binary and its structured envelope.
Normal use does not require external scraper or browser-control tools.

## What Works Today

- `aget get <url>` for HTTP(S), `raw:`, `raw://`, and `file://` inputs.
- `aget current-tab --cdp-port <port> --allow-private-content` for explicitly
  approved local Chrome DevTools tab extraction.
- `aget batch <url> [<url>...]` for bounded concurrent fetches with per-URL
  artifacts and a batch manifest.
- `aget map <url>` and `aget map --artifact <run-id>` for one-page link
  discovery without recursive fetching.
- `aget crawl <url> --limit <n>` for same-origin/path-bounded traversal with a
  manifest and per-page artifacts.
- `aget doctor` for local readiness diagnostics.
- `aget artifacts ...` for listing, inspecting, deleting, and pruning local run
  artifacts under `AGET_HOME`.
- `aget session ...` commands for listing, inspecting, deleting, composing,
  importing, authorizing, and bootstrapping local sessions.
- `--envelope json` for stable agent/tool output.
- `--content-format markdown|html|text|json`.
- `--output <path>` for writing extracted content to a chosen file.
- CSS `--selector`, `--exclude-selector`, and `--wait-for-selector` shaping.
- `--max-chars` deterministic post-extraction truncation.
- Repeated `--session <name>` for explicit named-session replay and composition.
- Replay-time checks that reject sessions outside the requested URL's saved
  scope.
- Local run artifacts under `~/.aget/runs`.
- Project skill guidance in `skills/aget/SKILL.md`.
- Project-local OpenCode tools in `.opencode/tools/aget.ts`.

Planned CLI work is tracked in `workpads/post-migration/tasks.md`.

## Install And Run

Developer build:

```bash
cargo build
cargo run -- --help
```

Developer install from this checkout:

```bash
cargo install --path .
aget --help
```

Current local release artifact:

```text
dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz
dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz.sha256
dist/SHA256SUMS
```

Install from the local release artifact:

```bash
tar -xzf dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz -C /tmp
/tmp/aget-v0.1.0-aarch64-apple-darwin/aget --help
```

Release artifact naming and smoke gates are tracked in
`workpads/post-migration/release-plan.md`.

Prerequisites:

- Rust and Cargo.
- Local Chrome or Chromium when using JavaScript-rendered pages, Chrome import,
  login flows, or current-tab extraction.
- Optional `cmux` only for `aget session import cmux`.

## Quick Start

Fetch a public page as markdown:

```bash
aget get https://example.com
```

Fetch with a structured envelope:

```bash
aget --envelope json get https://example.com
```

Write content to a file:

```bash
aget get https://example.com --output /tmp/example.md
```

Shape output:

```bash
aget --envelope json get https://example.com \
  --content-format markdown \
  --selector main \
  --max-chars 4000
```

Extract an explicitly approved current tab:

```bash
aget --envelope json current-tab \
  --cdp-port 9222 \
  --allow-private-content \
  --output /tmp/current-tab.md
```

Fetch several pages into a manifest:

```bash
aget --envelope json batch https://example.com https://example.org \
  --concurrency 2 \
  --output-dir /tmp/aget-batch
```

Discover links from one page without crawling them:

```bash
aget --envelope json map https://example.com/docs \
  --same-origin \
  --same-path \
  --max-links 200
```

Crawl a bounded docs section:

```bash
aget --envelope json crawl https://example.com/docs/ \
  --limit 25 \
  --max-depth 2 \
  --concurrency 2
```

## Command Reference

Top-level commands:

```text
aget get <url>
aget batch <url> [<url>...]
aget map <url>
aget map --artifact <run-id>
aget crawl <url> --limit <n>
aget current-tab --cdp-port <port> --allow-private-content
aget session <command>
aget artifacts <command>
aget doctor
```

`aget get`:

```text
aget get <url>
  [--session <name>...]
  [--envelope <json|none>]
  [--output <path>]
  [--timeout <seconds>]
  [--content-format <markdown|html|text|json>]
  [--inline-content <auto|always|never>]
  [--selector <css>]
  [--exclude-selector <css>]
  [--wait-for-selector <css>]
  [--max-chars <n>]
  [--backend-option <aget.key=value>...]
```

`aget batch`:

```text
aget batch <url> [<url>...]
  [--file <path>]
  [--stdin]
  [--session <name>...]
  [--envelope <json|none>]
  [--output-dir <path>]
  [--timeout <seconds>]
  [--content-format <markdown|html|text|json>]
  [--selector <css>]
  [--exclude-selector <css>]
  [--wait-for-selector <css>]
  [--max-chars <n>]
  [--backend-option <aget.key=value>...]
  [--concurrency <1-8>]
  [--fail-fast]
```

`aget batch` accepts positional URLs, newline-delimited `--file` input, or
newline-delimited `--stdin` input. It writes `manifest.json`, `manifest.md`,
and per-item content/metadata artifacts under `--output-dir`; without
`--output-dir`, it creates a local batch run directory under `AGET_HOME`.
Duplicate inputs are reported as skipped. Partial failures are reported in the
manifest and return a non-zero exit code.

`aget map`:

```text
aget map <url>
aget map --artifact <run-id>
  [--session <name>...]
  [--envelope <json|none>]
  [--selector <css>]
  [--exclude-selector <css>]
  [--wait-for-selector <css>]
  [--same-origin|--any-origin]
  [--same-path|--any-path]
  [--include <pattern>...]
  [--exclude <pattern>...]
  [--max-links <n>]
  [--content-type <type>...]
  [--output <markdown|json>]
  [--backend-option <aget.key=value>...]
```

`aget map` fetches one page as HTML or reads one successful internal `get` run
artifact, extracts links, deduplicates normalized absolute URLs, drops
fragments, and emits `links.json` plus `links.md` under a local `AGET_HOME` run
directory. It does not fetch discovered links. By default it keeps only links
on the same origin and same path prefix as the source; use `--any-origin` or
`--any-path` to widen discovery. Artifact mode rejects caller-owned external
`--output` content paths.

`aget crawl`:

```text
aget crawl <url> --limit <n>
  [--max-depth <n>]
  [--concurrency <1-8>]
  [--same-origin|--any-origin]
  [--same-path|--any-path]
  [--allow-domain <domain>...]
  [--include <pattern>...]
  [--exclude <pattern>...]
  [--session <name>...]
  [--envelope <json|none>]
  [--output-dir <path>]
  [--content-format <markdown|html|text|json>]
  [--selector <css>]
  [--exclude-selector <css>]
  [--wait-for-selector <css>]
  [--max-chars <n>]
  [--backend-option <aget.key=value>...]
```

`aget crawl` requires `--limit` and caps v1 crawls at 100 pages, depth 5, and
concurrency 8. It fetches pages through the same `get` pipeline, discovers
links from fetched HTML, and writes `manifest.json`, `manifest.md`, and
per-page artifacts under `--output-dir` or a local `AGET_HOME` run directory.
Traversal defaults to the start URL's origin and path prefix; `--any-origin`,
`--any-path`, and `--allow-domain` must be explicit. Crawl records partial
failures in the manifest and returns a non-zero exit code when any page fails.

`aget current-tab`:

```text
aget current-tab --cdp-port <port>
  --allow-private-content
  [--envelope <json|none>]
  [--output <path>]
  [--timeout <seconds>]
  [--content-format <markdown|html|text|json>]
  [--inline-content <auto|always|never>]
  [--selector <css>]
  [--exclude-selector <css>]
  [--wait-for-selector <css>]
  [--max-chars <n>]
  [--backend-option <aget.key=value>...]
```

Session commands:

```text
aget session list
aget session inspect <name>
aget session delete <name>
aget session compose <new-name> --session <name> [--session <name>...]
aget session authorize <name> --url <url> --browser chrome --browser-profile <profile> --allow-domain <domain>
aget session import cmux --surface <surface> --name <name> --allow-domain <domain>
aget session import browser --browser chrome --browser-profile <profile> --name <name> --allow-domain <domain>
aget session import chrome --chrome-profile <profile> --name <name> --allow-domain <domain>
aget session login start <name> --url <login-or-target-url> [--session <provider>...]
aget session login finish <name>
aget session login cancel <name>
```

Artifact commands:

```text
aget artifacts list
aget artifacts inspect <run-id>
aget artifacts delete <run-id> --yes
aget artifacts prune --older-than <duration> [--keep-last <n>] [--dry-run|--yes]
aget artifacts prune --keep-last <n> [--dry-run|--yes]
aget artifacts prune --max-bytes <bytes> [--keep-last <n>] [--dry-run|--yes]
```

Run `aget <command> --help` for full option details.

## Output Model

By default, `aget get` prints extracted page content directly. Markdown is the
default content format.

`--envelope json` prints a stable control-plane response:

```json
{
  "ok": true,
  "schema_version": "aget.envelope.v1",
  "command": "get",
  "data": {
    "url": "https://example.com",
    "final_url": "https://example.com/",
    "content_format": "markdown",
    "content": "# Example\n...",
    "artifacts": {
      "content": "/Users/me/.aget/runs/abc123/output.md",
      "metadata": "/Users/me/.aget/runs/abc123/metadata.json"
    },
    "sessions": [],
    "sensitive": false
  },
  "warnings": [],
  "timing_ms": {
    "total": 482
  }
}
```

Error envelopes use the same schema:

```json
{
  "ok": false,
  "schema_version": "aget.envelope.v1",
  "command": "get",
  "error": {
    "code": "extraction_failed",
    "message": "aget extraction failed"
  }
}
```

`--inline-content auto` includes `data.content` for non-sensitive fetches and
omits it for session-backed or current-tab output. Use `--output` and read the
artifact path for large or private content. Use `--inline-content always` only
when private content should be embedded in the JSON envelope.

## Sessions And Auth

`aget` starts with an empty session by default. Authenticated state is used only
when the caller passes explicit local session names.

Import a scoped browser session from an approved Chrome profile:

```bash
aget --envelope json session import browser \
  --browser chrome \
  --browser-profile Default \
  --name workdocs \
  --allow-domain docs.example.com
```

Fetch with the imported session:

```bash
aget --envelope json get "https://docs.example.com/account" \
  --session workdocs \
  --content-format markdown \
  --output /tmp/workdocs.md
```

Import and verify in one flow:

```bash
aget --envelope json session authorize workdocs \
  --url "https://docs.example.com/account" \
  --browser chrome \
  --browser-profile Default \
  --allow-domain docs.example.com \
  --must-contain "Account" \
  --output /tmp/workdocs-check.md
```

Start a controlled user-driven login flow:

```bash
aget --envelope json session login start workdocs \
  --url "https://docs.example.com/login"
# user completes login in the opened browser
aget --envelope json session login finish workdocs
```

For OAuth-backed sites, a reusable provider session may be injected into the
controlled login browser:

```bash
aget --envelope json session login start workdocs \
  --url "https://docs.example.com/login" \
  --session oauth
aget --envelope json session login finish workdocs
```

Provider sessions are for login bootstrap only. Do not pass a provider session
to `aget get` for target-site content. Replay-scope checks are a guardrail that
reject out-of-scope session use; they do not turn provider credentials into
target-site authorization. Fetch target content with the relying-party session
saved by `login finish`.

## Browser Support

Chrome/CDP is the verified browser path today.

`aget session import browser --browser chrome` is the browser-neutral command
for the verified Chrome path. Other browser names currently return structured
unsupported or usage errors until they are proven safe.

Prefer named Chrome profiles through `--browser-profile`. Explicit
`--profile-path` is advanced: it may launch that local profile directory
directly, so use it only with a disposable or explicitly approved profile path.
Import stores only the scoped exported cookies/storage in the named `aget`
session. It does not copy, retain, or manage the original browser profile as
part of `aget` state. For login flows, the default aget-owned temporary profile
is cleaned up after finish or cancel; a caller-provided custom profile path is
caller-owned and is not deleted or pruned by `aget`.

`current-tab` never scans profiles or common ports. The caller must provide an
explicit local CDP port and `--allow-private-content`.

## Privacy And Storage

- Only process content the user is authorized to access.
- `aget` does not collect, script, or store credentials.
- Session replay is opt-in per request with `--session <name>`.
- Session replay is rejected when selected sessions are outside the request host
  scope.
- Run artifacts live under `~/.aget/runs`.
- `artifacts delete` and `artifacts prune` remove only internal run directories
  under `AGET_HOME/runs`; caller-provided `--output` files outside those run
  directories are preserved.
- `session delete` removes saved auth/session state only. It does not delete
  previous extracted content artifacts or caller-provided `--output` files.
- Imported browser cookies and localStorage are credential-equivalent bearer
  material. `session inspect` redacts values by default.
- `current-tab` may read private authenticated browser content from the selected
  tab. Prefer `--output` and avoid `--inline-content always` unless explicitly
  requested.
- Do not use `aget` to bypass access controls, paywalls, or site policies.

`aget` is a generic fetcher. It does not contain site-specific paywall, login,
rate-limit, or access-state detection. The calling agent interprets returned
content and decides whether to ask the user for a session, a narrower selector,
or a different URL.

## OpenCode Integration

This repo includes project-local OpenCode tools in `.opencode/tools/aget.ts`.
They call the installed `aget` binary with `--envelope json` and return the same
structured envelopes as terminal usage.

Available tools:

- `aget_fetch`
- `aget_session_list`
- `aget_session_inspect`
- `aget_session_import_chrome`

Use `AGET_OPENCODE_BIN` to point OpenCode at a development binary:

```bash
cargo build
AGET_OPENCODE_BIN="$PWD/target/debug/aget" opencode
```

## Development

```bash
cargo fmt --check
cargo test
git diff --check
bash -n scripts/demo_real_cli.sh
```

Testing layers:

- Focused unit and integration tests for parser, envelope, extraction, session,
  subprocess, artifact, and redaction behavior.
- `tests/mock_site_cli.rs` and `tests/support/mock_site.rs` for deterministic
  local auth/session coverage without real credentials or live sites.
- Behavior-level parity tests for historical source projects only where `aget`
  implements the corresponding feature. These tests should be local,
  license-aware, and adapted to `aget` behavior rather than copied blindly.
- Ignored/manual real-browser checks for confidence in user-authorized local
  Chrome flows.

## Historical Notes

Early `aget` prototypes used external source projects as implementation and
behavior references. The current default runtime is owned Rust code. Historical
source snapshots remain useful for parity research and license-aware behavior
comparison, but they are not normal user prerequisites.

## License

The package declares MIT in `Cargo.toml`. This repository does not currently
include a separate `LICENSE` file.
