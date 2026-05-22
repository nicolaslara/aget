# R12: Privacy, Integration, Implementation, And Risks

## Privacy Model

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

## Plugin Approach

OpenCode integration should be CLI-backed first. A minimal plugin or command wrapper should call `aget` as a local binary and expose stable tool schemas for:

- `aget_fetch` -> `aget get --json`
- `aget_session_list` -> `aget session list --json`
- `aget_session_inspect` -> `aget session inspect --json`

MCP is deferred until the CLI and session model settle. The CLI-backed OpenCode path is enough for v1 because it preserves one source of behavior and avoids maintaining an always-on local server before the privacy model is hardened.

## Implementation Tasks

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

## Open Risks And Deferred Features

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

## R12 Confidence

Confidence: Medium-high. The proposal is backed by local benchmarks and explicit PoC tests for the key auth bridge and cmux cookie replay. Remaining uncertainty is mostly around product/security hardening choices, especially plaintext session persistence and how quickly to replace Crawl4AI/agent-browser internals.
