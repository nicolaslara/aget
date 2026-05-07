# aget Session Wrapper Implementation Plan

## Objective

Build the v1 PoC as a thin wrapper that proves the `aget` product model:

- Empty session by default.
- Explicit named sessions.
- Per-request session composition via repeated `--session` flags.
- Optional cmux import for users in cmux.
- Chrome/profile import through `agent-browser` for users without cmux.
- Crawl4AI-backed local extraction using temporary Playwright storage state.
- Non-interactive, timeout-bounded, agent-safe command behavior.

## Implementation Principle

The first implementation should optimize for correctness and testability over completeness. The smallest useful product slice is:

```bash
aget <url>
aget get <url>
aget session import cmux --surface <surface> --name <name> --domain <domain>
aget get <url> --session <name>
aget session inspect <name>
```

Chrome import, session composition, and additional extractor options should follow once the local session store and Crawl4AI replay path are tested.

## Proposed Stack

Use Rust for the CLI and session/state logic.

Use external tools for v1 backend work:

- cmux CLI for optional browser-surface cookie import.
- `agent-browser` CLI for Chrome profile/session state acquisition.
- Python/Crawl4AI helper for rendered extraction.

Rust should own:

- CLI parsing.
- Session file format.
- Cookie/storage filtering.
- Session composition.
- Temporary state creation/deletion.
- Command timeout/process orchestration.
- Output metadata.
- Testable pure logic.

## Repository Shape

Expected first implementation layout:

```text
Cargo.toml
src/
  main.rs
  cli.rs
  commands/
    get.rs
    session.rs
  session/
    mod.rs
    store.rs
    model.rs
    compose.rs
    filter.rs
    playwright.rs
  backends/
    mod.rs
    crawl4ai.rs
    cmux.rs
    agent_browser.rs
  output/
    mod.rs
    metadata.rs
  process.rs
  error.rs
scripts/
  crawl4ai_extract.py
tests/
  fixtures/
  integration/
```

The exact names can change, but keep pure session logic separate from shelling out to backends.

## Milestone 0: Project Skeleton

Deliverables:

- `Cargo.toml` and Rust binary entry point.
- CLI parser with top-level alias: `aget <url>` delegates to `aget get <url>`.
- Global flags: `--json`, `--timeout`, `--verbose`, `--quiet`.
- Error type with stable categories.
- Unit test harness in place.

Acceptance criteria:

- `aget --help` works.
- `aget get --help` works.
- `aget https://example.com --dry-run` parses as a get command if `--dry-run` is added, or equivalent parse test exists.
- Unit tests pass.

## Milestone 1: Session Model And Store

Deliverables:

- Session model matching `session-wrapper-poc-spec.md` v1 shape.
- Local store under `~/.aget/sessions` by default.
- Test override via `AGET_HOME` for integration tests.
- `aget session inspect <name>` with redacted values by default.
- `aget session list`.
- `aget session delete <name>`.

Acceptance criteria:

- Session files are not stored in the project repo by default.
- `~/.aget`, `sessions/`, `runs/`, `cache/`, and `tmp/` are created with `0700` permissions on Unix-like systems.
- Session JSON files are created with `0600` permissions on Unix-like systems.
- Values are redacted unless `--show-secrets` is passed.
- Tests can isolate storage using `AGET_HOME`.
- Tests verify restrictive permissions where the platform supports Unix modes.

## Milestone 2: Session Composition And Playwright State

Deliverables:

- Compose zero, one, or many sessions into a Playwright-compatible temporary state.
- Empty session produces `{ "cookies": [], "origins": [] }`.
- Duplicate identical cookies deduplicate.
- Conflicting cookies fail.
- Temporary state files are deleted on success and failure.
- Temporary Playwright state files are created with `0600` permissions on Unix-like systems.

Acceptance criteria:

- Unit tests cover composition and conflicts.
- Integration test verifies temp state cleanup.

## Milestone 3: Crawl4AI Extraction Adapter

Deliverables:

- `scripts/crawl4ai_extract.py` helper based on the proven benchmark script.
- Rust adapter that shells out with a timeout.
- `aget get <url>` using empty session.
- Output files under `~/.aget/runs/<run-id>/` unless `--out` is provided.
- Metadata JSON with URL, extractor, sessions, sensitivity, timing, and truncation fields.

Acceptance criteria:

- `aget https://example.com` returns or writes markdown.
- `aget get <local-test-url>` works in integration tests.
- `--json` returns structured output.
- Timeout failures are classified as timeout errors.

## Milestone 4: cmux Session Import

Deliverables:

- `aget session import cmux --surface <surface> --name <name> --domain <domain>...`.
- cmux availability detection.
- Per-domain cookie import using `cmux --json browser cookies get --domain <domain>`.
- Mandatory post-filtering by explicit allowed domains.
- Normalize cmux cookies to the internal session model.

Acceptance criteria:

- Local e2e test sets a cookie in a cmux browser pane, imports it, and replays it through Crawl4AI.
- Test proves `--url` scoping is not required/trusted.
- Missing cmux returns backend-unavailable error.

## Milestone 5: Extractor Options And Output Limits

Deliverables:

- Add common get options:
  - `--format markdown|html|text|json`
  - `--selector`
  - `--exclude-selector`
  - `--wait-for`
  - `--max-chars`
  - `--max-tokens`
  - `--extractor-option key=value`
- Implement deterministic char truncation first.
- Token estimation can be approximate in v1 but must be recorded as approximate.

Acceptance criteria:

- Output shaping options are represented in metadata.
- Unsupported backend options fail clearly or are forwarded explicitly.
- Truncation metadata is correct.

## Milestone 6: Chrome Import Via agent-browser

Deliverables:

- `aget session import chrome --profile <profile> --name <name> --domain <domain>...`.
- Named temporary `agent-browser` session.
- Raw `agent-browser state save` temp file.
- Strict domain/origin filtering into `aget` session.
- Raw broad state deletion.
- Close only the named temporary `agent-browser` session.

Acceptance criteria:

- Manual authenticated-site verification reproduces the HelloInterview success path.
- If Chrome/profile state cannot be imported non-interactively, command fails with `requires_user_action`.
- No raw broad state remains after success/failure.

## Milestone 7: Session Composition CLI

Deliverables:

- `aget session compose <new-name> --session <name>...`.
- `aget get <url> --session a --session b` per-request composition.
- Conflict reporting with redacted values.

Acceptance criteria:

- Local e2e app can require app+provider sessions.
- Per-request composition does not mutate source sessions.
- Composed session records provenance.

## Testing Strategy

### Unit Tests

Run on every change:

- CLI alias parsing.
- Session model serialization/deserialization.
- Domain and origin allowlist filtering.
- Cookie normalization from cmux and agent-browser shapes.
- Session composition and conflict detection.
- Metadata generation.
- Redaction behavior.
- Timeout parsing.

### Integration Tests

Run locally and in CI where dependencies exist:

- Local HTTP test server for public/authenticated/cookie echo pages.
- Empty fetch has no cookies.
- Session replay sends expected cookies.
- Temp state cleanup on success and failure.
- Missing backend errors are structured.
- `AGET_HOME` isolates test state.

### E2E Tests

Use a local browser-backed test app:

- Public page extraction.
- JS-rendered page extraction.
- App login cookie import.
- OAuth-like provider plus app session composition.
- Provider pruning.
- Sensitive output metadata.

cmux e2e tests should be optional/skipped when cmux is unavailable.

agent-browser/Chrome import e2e should start as manual or ignored tests because it depends on local browser state.

## First Build Slice

Start with this exact slice:

1. Rust CLI skeleton.
2. Session model/store with `AGET_HOME` test override.
3. Playwright state writer from zero/one sessions.
4. Crawl4AI helper and `aget get` with empty session.
5. Local integration test proving cookie replay via a hand-written session file.

Only after that add cmux import.

This avoids entangling session storage, backend orchestration, and cmux quirks before the core replay path is tested.

## Decisions Before Coding

- V1 may persist plaintext sessions as a PoC compromise, but only outside the repo, with explicit credential-equivalent warnings, `0700` directories, `0600` session/temp-state files, and a tracked follow-up for encryption at rest.
- `aget get` should write run artifacts under `~/.aget/runs/<run-id>/` by default and print a concise summary; `--json` returns the stable machine-readable shape; `--out` can write content to a caller-chosen path.
- Crawl4AI should be invoked through `uv run --with crawl4ai` for the PoC to avoid committing to a managed Python environment before the wrapper direction is validated.
- Default timeouts should start with command=60s, navigation=30s, extraction=45s, backend_startup=20s, all configurable through CLI/config.
- Token estimation should start with a simple approximate heuristic recorded as approximate; a tokenizer dependency is deferred until output-shaping behavior is otherwise stable.
