# Artifact Lifecycle Command Design

## Scope

`aget artifacts` manages internal run artifacts under `AGET_HOME/runs`.

It must not manage:

- saved auth sessions under `AGET_HOME/sessions`
- temporary browser/session files under `AGET_HOME/tmp`
- cache files under `AGET_HOME/cache`
- caller-owned files passed with `--output`, unless that path is inside the
  selected internal run directory
- release artifacts under repo-local `dist/`

This command family is a local CLI surface only. It has no server, MCP layer,
hosted API, or site-specific behavior.

## Current Artifact Contract

Every successful or failed extraction has an internal run directory:

```text
AGET_HOME/runs/run-<pid>-<nanos>/
  metadata.json
  content.md       # only when the caller did not pass --output outside the run dir
  backend-stdout.json / backend-stderr.txt  # optional backend diagnostic files
```

`metadata.json` records:

- `ok`
- `url`
- `final_url` for successful runs
- `content_format`
- `extractor`
- `artifacts.content`
- `artifacts.metadata`
- `sessions`
- `sensitive`
- `warnings`
- `timing_ms`
- `limits`
- `output_options`
- `error` for failed runs

When the caller passes `--output`, `artifacts.content` may point outside
`AGET_HOME/runs`. Lifecycle commands must treat that file as caller-owned.

## Command Shape

Top-level:

```text
aget artifacts list
aget artifacts inspect <run-id>
aget artifacts delete <run-id> [--yes]
aget artifacts prune [--older-than <duration>] [--keep-last <n>] [--max-bytes <bytes>] [--dry-run] [--yes]
```

Global flags keep existing behavior:

```text
aget --envelope json artifacts list
aget --envelope json artifacts inspect <run-id>
aget --envelope json artifacts delete <run-id> --yes
aget --envelope json artifacts prune --older-than 30d --dry-run
```

Run IDs are directory basenames such as `run-6762-1779659888819350000`.
Commands reject names containing path separators, `.` segments, or absolute
paths. They resolve only below `SessionStore::home()/runs`.

## List

Human output is a compact table:

```text
RUN ID                         AGE    SIZE     STATUS  SENSITIVE  HOST             CONTENT
run-6762-1779659888819350000   12m    4.2 KiB  ok      no         example.com      internal
```

JSON envelope data:

```json
{
  "runs": [
    {
      "run_id": "run-6762-1779659888819350000",
      "run_dir": "/Users/me/.aget/runs/run-6762-1779659888819350000",
      "modified_at": "2026-05-24T21:58:08Z",
      "age_seconds": 720,
      "size_bytes": 4300,
      "ok": true,
      "sensitive": false,
      "content_format": "markdown",
      "extractor": "aget-owned-extractor",
      "source": {
        "url": "https://example.com/docs",
        "final_url": "https://example.com/docs",
        "host": "example.com"
      },
      "content": {
        "path": "/Users/me/.aget/runs/run-.../content.md",
        "ownership": "internal",
        "exists": true
      },
      "metadata": {
        "path": "/Users/me/.aget/runs/run-.../metadata.json",
        "valid": true
      }
    }
  ],
  "summary": {
    "count": 1,
    "total_size_bytes": 4300,
    "sensitive_count": 0,
    "malformed_count": 0
  }
}
```

Sorting:

- default: newest first by run directory modification time
- future optional flags: `--sort newest|oldest|size`

## Inspect

Human output shows metadata and file ownership, not page content by default.

JSON envelope data:

```json
{
  "run_id": "run-...",
  "run_dir": "...",
  "metadata": {
    "valid": true,
    "path": ".../metadata.json",
    "value": {
      "ok": true,
      "url": "https://example.com/docs",
      "final_url": "https://example.com/docs",
      "sensitive": false
    }
  },
  "files": [
    {
      "path": ".../metadata.json",
      "kind": "metadata",
      "ownership": "internal",
      "size_bytes": 1200
    },
    {
      "path": "/tmp/user-output.md",
      "kind": "content",
      "ownership": "external",
      "exists": true
    }
  ]
}
```

`inspect` may include redacted metadata JSON but must not print artifact content
unless a later explicit content-reading flag is designed. Do not add that flag
in ART-002.

## Delete

Delete removes one internal run directory and files inside it.

Rules:

- Require `--yes` for actual deletion.
- Without `--yes`, print the deletion plan and exit with a usage error in human
  and JSON modes. The message should say to rerun with `--yes`.
- Delete only files physically under the selected run directory.
- If `metadata.artifacts.content` points outside the run directory, report it as
  `external_preserved`.
- If `metadata.artifacts.content` points inside the run directory, it is deleted
  as part of the run directory.
- Refuse to delete if the resolved run path is not under `AGET_HOME/runs`.
- Missing run ID returns a not-found error and does not mutate state.

JSON envelope data for successful deletion:

```json
{
  "run_id": "run-...",
  "deleted": true,
  "deleted_paths": [".../metadata.json", ".../content.md"],
  "preserved_external_paths": ["/tmp/user-output.md"],
  "freed_bytes": 4300
}
```

## Prune

Prune selects multiple internal run directories using retention criteria.

Flags:

```text
--older-than <duration>   # e.g. 7d, 30d, 12h
--keep-last <n>           # always preserve the newest n runs
--max-bytes <bytes>       # delete oldest runs until total internal run size is <= bytes
--dry-run                 # show plan without deleting; default for prune unless --yes is passed
--yes                     # execute the computed plan
```

Selection semantics:

- If no selector is provided, prune returns a usage error and does nothing.
- `--keep-last` is applied as a protection after other selectors.
- `--max-bytes` deletes oldest eligible runs first.
- Sensitive runs are eligible, but human output labels them clearly before
  deletion. Since deletion removes local sensitive data, there is no extra
  prompt beyond `--yes`.
- Malformed run directories may be pruned by age/size, but list/inspect should
  label them `metadata.valid = false`.

Dry-run behavior:

- `prune` defaults to dry-run unless `--yes` is present.
- Explicit `--dry-run --yes` is rejected as conflicting.
- JSON output includes `dry_run: true`, `would_delete`, and `would_free_bytes`.

## Retention Configuration

ART-002 should implement command flags first and no automatic background
pruning. Retention policy is opt-in and local.

The command should define, but does not have to persist, these config names for
future use:

```text
artifacts.retention_days
artifacts.keep_last
artifacts.max_bytes
```

When a config file exists in a later task, CLI flags override config. Until
then, defaults are:

```text
retention_days = null
keep_last = null
max_bytes = null
auto_prune = false
```

This means `aget get` never deletes artifacts implicitly.

## Redaction Policy

Artifact lifecycle commands must treat `metadata.json` as sensitive metadata
when `sensitive: true`.

Default redaction:

- Do not print page content.
- Do not print cookie, localStorage, sessionStorage, backend stdout, or backend
  stderr values.
- For `sensitive: true`, redact query strings and fragments in `url` and
  `final_url` for human output.
- JSON output should use the same redacted URL fields as human output when
  `sensitive: true`. It must not expose a full raw query string or fragment for
  sensitive runs unless a later explicit raw-metadata flag is designed.
- `sessions` can be listed as names because they are already in metadata, but
  values are never shown.
- Malformed metadata errors should include parse errors and paths, not file
  contents.

Recommended URL redaction for human output:

```text
https://example.com/path?token=...#fragment -> https://example.com/path?<redacted>
```

## Confirmation Rules

- `list`: never destructive, no confirmation.
- `inspect`: never destructive, no confirmation.
- `delete`: destructive, requires `--yes`.
- `prune`: destructive when `--yes`; otherwise dry-run.
- Interactive prompts are out of scope for ART-002. Non-interactive explicit
  flags keep agent workflows deterministic.

## Error Model

Use existing `ErrorResponse` shape with stable command names:

```text
artifacts.list
artifacts.inspect
artifacts.delete
artifacts.prune
```

Likely error codes:

- `usage_error`: invalid run ID, missing prune selector, conflicting flags,
  deletion requested without `--yes`
- `io_error`: unreadable `AGET_HOME`, run dir, or metadata file
- `extraction_failed` is not used by artifact lifecycle commands

## ART-002 Implementation Notes

Suggested modules:

```text
src/cli/artifacts.rs
src/main_artifacts.rs
src/artifacts/
  mod.rs
  listing.rs
  metadata.rs
  deletion.rs
```

Tests should cover:

- parser shape and help output
- list JSON shape with empty and populated `AGET_HOME/runs`
- inspect valid metadata and malformed metadata
- delete requires `--yes`
- delete preserves external `--output` files
- delete removes internal run directories
- prune defaults to dry-run
- prune rejects no selector and `--dry-run --yes`
- prune applies `--older-than`, `--keep-last`, and `--max-bytes`
- sensitive metadata redacts human output and never reads content files

Focused verification for ART-002:

```bash
cargo fmt --check
cargo test --test cli artifacts
cargo test --test get_cli get_json_success_writes_run_artifacts_with_empty_state
cargo test
```
