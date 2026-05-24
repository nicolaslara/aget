# Post-Migration Knowledge

## Current Direction

`aget` is now a Rust CLI product. The source of truth for behavior is the CLI
and structured envelope, backed by the local `AgetExtractor`, `AgetBrowser`, and
session store implementations.

MCP is out of scope. Future integrations should call the CLI directly or wrap
the CLI envelope in a thin host-specific tool.

## Immediate Priorities

1. Make README, skill, OpenCode tool guidance, `project.md`, and active workpad
   routing match the current code.
2. Remove active Crawl4AI and `agent-browser` dependency surfaces. Historical
   notes and parity references may remain only when clearly labeled.
3. Build a parity ledger from historical Crawl4AI and `agent-browser` tests for
   features `aget` implements, then add/fix local tests for uncovered behavior.

## Completed Setup

- PM-001 created this active workpad and switched repo routing away from the old
  research bootstrap state.
- DOC-001 audit found that README, the aget skill, and the OpenCode tool still
  expose Crawl4AI or `agent-browser` as active compatibility surfaces, while
  current CLI help exposes only `get`, `current-tab`, and `session`. The next
  documentation pass should make README/skill CLI-first and leave old
  dependencies only as historical/parity context.
- DOC-002 rewrote README as CLI-first current-state documentation. It no longer
  presents old external command adapters as setup or normal compatibility
  backends, and it documents provider-session replay scope, Chrome/CDP support
  limits, run-artifact retention, and the post-migration roadmap.
- DOC-003 aligned `skills/aget/SKILL.md` with the README by removing active
  compatibility-backend guidance and old dependency names from error guidance.

## Commit Decisions

- 2026-05-24: No commit after DOC-001 through DOC-003 yet. The user asked to
  execute the full post-migration plan, and no explicit commit was requested for
  this partial pass.
- 2026-05-24: No commit immediately after PAR-001. The task is documentation
  and matrix-only, the active goal is continuing into PAR-002, and no new
  explicit commit approval was given after the previous pushed commit.
- 2026-05-24: No commit immediately after PAR-002/PAR-003. The repo rule
  requires explicit commit approval, and the current goal continuation did not
  ask for another commit.

## Product Boundaries

- `session delete` deletes saved auth/session state. It does not delete previous
  extraction artifacts or caller-provided `--output` files.
- Provider sessions are for login bootstrap injection through
  `session login start`; they are not replayed against unrelated target-site
  fetches.
- Chrome/CDP is the verified browser path. Other browser families and profile
  modes should return structured unsupported/deferred outcomes until proven.
- Historical source snapshots may inform parity tests, but they are not runtime
  dependencies and should not drive ordinary product changes by default.
- Current-tab CDP endpoint reporting is useful provenance/debug context and is
  not a closure blocker when the user explicitly supplies `--cdp-port` and
  `--allow-private-content`; keep warning wording factual rather than treating
  the endpoint itself as secret page content.
- Browser/profile import stores scoped exported cookies/storage in named `aget`
  sessions. It does not retain an entire source browser profile. Default
  aget-owned login profiles are cleanup-owned by `aget`, while custom
  `--profile-path` login profiles are caller-owned and are not deleted.
- DEP-001/CLI-001 removed active command-backed Crawl4AI and agent-browser
  runtime surfaces. Compatibility adapters and mock-command fixtures were
  deleted rather than kept behind dev-only tests because they no longer
  represent supported product behavior.
- PAR-001 inventory maps historical source-project tests to current `aget`
  features without copying upstream tests. Main implemented-but-partial areas
  for PAR-002 are raw HTML edge cases, deterministic CSS wait coverage, GFM
  table edge cases, selector no-match/invalid behavior, browser-state fixture
  import, provider-session CLI coverage, same-scope stale-session replacement,
  and artifact redaction on failure metadata.
- PAR-002 resolved the matrix's implemented-but-uncovered rows with local
  deterministic tests where the current architecture supports them. The one
  explicit deviation is provider-session injection at CLI level: it is covered
  through the public `Aget` API with an injected test browser backend, but a
  deterministic CLI test is deferred until the CLI has a supported fake-browser
  test seam.
- PAR-003 fixed a parity bug found by the new raw HTML tests: fragment-only
  links in `raw:` input should remain fragment links instead of being resolved
  against the synthetic raw input URL. Explicit `aget.base_url` and HTML
  `<base>` tags remain the supported way to resolve relative links in raw
  content.
- DR-001 defines `aget doctor` as a local CLI diagnostic surface. Static fetch
  readiness is separate from optional Chrome/CDP and cmux readiness; missing
  optional components should warn rather than fail the whole doctor run. Doctor
  must not scan CDP ports, read current-tab target URLs, dump session JSON, or
  report cookie/storage values.
- DR-002 implemented `aget doctor` as a CLI-only readiness surface with stable
  check IDs and JSON envelope output. Missing Chrome/CDP or cmux remains a
  warning because static fetch can still work; failed `AGET_HOME` layout or
  writability is a failure.
- Review follow-up accepted on provider-session wording: provider sessions are
  login-bootstrap inputs for `session login start`, not target fetch sessions.
  Replay-scope enforcement is a guardrail and should not be described as
  permission to use provider credentials for relying-party content.
- Review follow-up accepted on retention wording: import persists only scoped
  cookies/storage in named `aget` sessions. Whole source browser profiles are
  not retained by `aget`; caller-provided custom login profile paths remain
  caller-owned and are not pruned by `aget`.

## Execution Order

Follow this order unless the user redirects:

1. PM-001
2. DOC-001 through DOC-003
3. DEP-001 and CLI-001
4. PAR-001 through PAR-003
5. DR-001 and DR-002
6. REL-001 and REL-002
7. ART-001 and ART-002
8. BACKLOG-001, then BATCH-001, MAP-001, CRAWL-001
