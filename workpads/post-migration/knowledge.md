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
- 2026-05-24: No commit immediately after REL-001. The repo rule requires
  explicit commit approval, and the active goal continuation is moving into
  REL-002.
- 2026-05-24: No commit immediately after REL-002. The repo rule requires
  explicit commit approval, and the active goal continuation is moving into
  ART-001.
- 2026-05-25: No commit immediately after ART-001/ART-002. The repo rule
  requires explicit commit approval, and the active goal continuation is moving
  into BACKLOG-001.
- 2026-05-25: No commit immediately after MAP-001. The repo rule requires
  explicit commit approval before committing the completed map pass.
- 2026-05-25: No commit immediately after CRAWL-001. The post-migration workpad
  is complete, validation passed, and the only remaining closure step is an
  explicit user-approved commit for the uncommitted MAP/CRAWL pass.

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
- REL-001 defines releases as CLI binary artifacts only. The first artifact
  targets are macOS and Linux tarballs; Windows remains deferred until a Windows
  smoke path exists. Notarized app bundles are out of scope unless explicitly
  requested.
- REL-001 found and fixed stale demo script flags: use `--envelope json` and
  `--output`, not removed aliases such as `--json` or `--out`.
- Release artifact production is blocked on REL-002 prerequisites recorded in
  the plan: add a top-level `LICENSE` matching Cargo's MIT metadata and create
  `CHANGELOG.md` before publishing artifacts.
- REL-002 produced the first local release artifact for the current verified
  host target, `aarch64-apple-darwin`. The current release output is a local
  `dist/` directory, not a published GitHub release. Cross-target artifacts
  remain future work until each target has its own smoke path.
- REL-002 added top-level `LICENSE` and `CHANGELOG.md` so the tarball includes
  the declared MIT license and the release has a user-visible change log.
- ART-001 defines artifact lifecycle commands as management of internal
  `AGET_HOME/runs/<run-id>` directories only. A `metadata.artifacts.content`
  path outside the selected run directory is caller-owned, even when it was
  produced by `--output`, and must never be deleted by lifecycle commands.
- ART-001 keeps artifact retention explicit and local: `aget get` does not
  auto-prune, `prune` defaults to dry-run unless `--yes` is present, and future
  config names are `artifacts.retention_days`, `artifacts.keep_last`, and
  `artifacts.max_bytes`.
- ART-002 implemented the artifact lifecycle command family. The release
  artifact in `dist/` was regenerated afterward so the packaged binary and
  README include `aget artifacts`.
- BACKLOG-001 defines the bounded multi-URL backlog. Implementation order is
  `batch` first, then `map`, then `crawl`; `crawl` requires `--limit`, defaults
  to same-origin/same-path traversal, and records partial failures in manifests
  instead of pretending the whole run succeeded.
- BATCH-001 implements the first multi-URL command as explicit-list fetch only:
  positional URLs, `--file`, and `--stdin` feed a bounded concurrent `get`
  pipeline; `map` owns link discovery and `crawl` owns recursive traversal.
  Batch manifests are the durable source of truth for partial success, duplicate
  skips, per-item artifact paths, and non-zero exit behavior.
- BATCH-001 uncovered a shared same-process concurrency hazard: artifact run IDs
  and temporary Playwright state filenames previously used only process ID plus
  nanoseconds. Concurrent in-process fetches can collide, so both names now add
  a monotonic per-process counter.
- MAP-001 keeps discovery non-recursive. URL input is fetched as HTML through
  the existing `get` pipeline; artifact input is limited to successful internal
  `get` run content so `map` does not silently read caller-owned external
  output files. Discovered links are metadata, not fetched page content.
- MAP-001 implements lightweight local filtering only: normalized absolute URL
  dedupe with fragments dropped, same-origin/same-path defaults, simple
  include/exclude glob patterns, and content-type inference from URL path
  extensions. Stronger MIME verification belongs to `crawl` or a later
  discovery task because `map` deliberately does not fetch discovered URLs.
- CRAWL-001 completes the bounded multi-URL CLI backlog. `crawl` requires a
  user-supplied limit and keeps traversal generic: no paywall/login/CAPTCHA
  detection, no site-shaped retry advice, and no automatic policy
  interpretation. It records partial failures in the manifest and leaves
  site-specific decisions to the calling agent.
- CRAWL-001 uses fetched HTML for discovery and the existing `get` pipeline for
  per-page artifacts. When the requested crawl content format is not HTML,
  discovery may require an additional HTML fetch for the same URL; this keeps
  map/crawl behavior source-aligned with `get` rather than adding a separate
  extraction path.
- 2026-05-26 next-backlog triage: highest value after post-migration closure is
  distribution and agent usability first (`REL-003`, `AGENT-001`), then
  artifact leverage (`CACHE-001`, `SEARCH-001`, `EXTRACT-001`), then release
  automation (`REL-004`), then higher-risk browser features (`DEBUG-001`,
  `ACT-001`). This order keeps the CLI installable and useful to agents before
  adding broader browser automation surfaces.
- REL-003 packaging should treat the Codex skill as part of the release
  install surface. The macOS ARM tarball now includes `skills/aget/SKILL.md`,
  README documents checkout and tarball skill installation into
  `$CODEX_HOME/skills/aget`, and Codex restart remains required after install
  or replacement.
- Global Codex skill installation should be script-backed for checkouts rather
  than relying only on copy/paste blocks. `scripts/install-codex-skill.sh`
  defaults to copying the skill, supports `--symlink` for local development,
  and is the preferred local checkout install path.
- REL-003 source install needs an explicit package argument:
  `cargo install --git https://github.com/nicolaslara/aget --tag v0.1.0 --locked aget`.
  The repository contains another binary package, so omitting `aget` causes
  Cargo to reject the install.
- AGENT-001 keeps OpenCode integration as thin CLI/envelope wrappers. The
  wrappers build command arguments for `batch`, `map`, `crawl`,
  `artifacts list/inspect`, and `doctor`; tests cover generated args rather
  than invoking OpenCode itself.
- AGENT-001 helper code should stay outside `.opencode/tools` so project-local
  tool discovery sees only actual tool exports. Argument builders live in
  `.opencode/lib/aget_args.ts`; deterministic snapshots live in
  `.opencode/tests/aget_args.test.ts`.
- CACHE-001 makes cache reuse a `get`-pipeline behavior. Only public
  unauthenticated HTTP(S) inputs are reusable; session-backed, current-tab,
  `raw:`, and `file://` inputs record cache metadata but do not read/write
  cache entries.
- CACHE-001 cache keys include URL, output format, extraction-shaping options,
  and normalized backend options. They intentionally ignore caller output paths
  and `--max-chars` because cache entries store full untruncated extracted
  content and each run applies its own output limits.
- CACHE-001 propagates cache controls through `batch`, `map`, and `crawl`.
  Batch and crawl manifests record per-item cache/usage metadata; map URL mode
  records source cache/usage metadata. Corrupt or unreadable cache entries are
  treated as misses so cache state does not block a fresh fetch.
- CACHE-001 usage metadata is approximate budgeting data, not tokenizer-exact
  accounting. It reports fetched bytes when known, final content bytes, rough
  token estimates, and rough token-saved estimates.
- SEARCH-001 starts as artifact-first deterministic narrowing through
  `aget search-page --artifact <run-id> --query <text>`. It does not refetch
  pages or call an LLM; it scores existing local artifact sections by headings,
  phrase/keyword matches, and structured lines such as links, lists, and tables.
- SEARCH-001 treats snippets from sensitive artifacts as private content. A
  sensitive source artifact requires `--allow-private-content` before snippets
  are emitted in human output or JSON envelopes.
- EXTRACT-001 adds artifact-first structured extraction through `aget extract`.
  Built-in fields are deterministic and local: headings, links, tables,
  definitions, and metadata. Schema files add HTML selector fields and JSON
  path fields; LLM interpretation remains outside `aget`.
- EXTRACT-001 keeps source reads bounded. `extract --artifact` accepts only
  successful internal `get` run content and refuses caller-owned external
  `--output` paths. `extract --manifest` requires per-item `aget` metadata
  under `AGET_HOME/runs`, requires manifest item content to match metadata, and
  keeps item content under the manifest directory. Sensitive source artifacts
  require `--allow-private-content`.
- EXTRACT-001 review accepted two privacy findings before closure: manifest
  extraction must not fall back to unsigned manifest fields, and extraction
  must not trust external artifact content paths. Both findings were fixed and
  covered by regression tests.

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
