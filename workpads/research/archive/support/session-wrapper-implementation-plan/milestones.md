# Session Wrapper Implementation Plan: Milestones

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
  - `--extractor-option backend.key=value`
- Implement deterministic char truncation first.
- Token limits are deferred until they can be enforced; do not expose metadata-only token truncation.

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
