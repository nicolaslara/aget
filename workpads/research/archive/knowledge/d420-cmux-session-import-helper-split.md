# D420: Cmux Session Import Helper Split

Date: 2026-05-23

## Decision

Split the cmux session import implementation from a single `src/session/cmux.rs`
file into a route module plus focused helpers:

- `src/session/cmux/mod.rs`: public module route and re-exports.
- `src/session/cmux/import.rs`: session construction, allowed-domain filtering,
  and duplicate-cookie conflict detection.
- `src/session/cmux/command.rs`: `cmux browser cookies get` invocation, private
  stdout/stderr files, timeout handling, backend error classification, and JSON
  parsing.
- `src/session/cmux/domains.rs`: allowed-domain matching and normalization.
- `src/session/cmux/types.rs`: public import options and deserialized cmux
  response types.

## Rationale

Cmux import is already a small compatibility boundary, but the old file mixed
process execution, cookie filtering, conflict handling, and response models.
The split keeps `CmuxImportOptions` and `import_cmux_session` unchanged while
making the security-sensitive filtering path easier to inspect independently
from command execution.

## Validation

- `cargo test --lib session::cmux`
- `cargo test --test session_cli imports_cmux`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`
- Line-count check for the split modules:
  - `cmux/mod.rs`: 7 lines
  - `cmux/import.rs`: 67 lines
  - `cmux/command.rs`: 86 lines
  - `cmux/domains.rs`: 51 lines
  - `cmux/types.rs`: 27 lines
