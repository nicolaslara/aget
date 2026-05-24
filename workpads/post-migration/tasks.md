# Post-Migration Tasks

## Phase 1: Source-Of-Truth Reset

### ✅ Task PM-001: Reset active workpad and phase docs

Acceptance criteria:

- `workpads/WORKPADS.md` points to `post-migration` as the active workpad.
- `AGENTS.md` no longer blocks implementation as research-only.
- MCP is explicitly out of scope for current project direction.
- Historical research and migration notes remain reachable through
  `workpads/research/`.
- Verify with a workpad/docs grep that active routing does not point agents back
  to pre-migration research as the executable backlog.

Status note:

- Completed in the post-migration planning pass. `AGENTS.md` and
  `workpads/WORKPADS.md` now route future work to this workpad, MCP is explicit
  out of scope, and `workpads/research/` remains available as historical
  research/migration evidence.

### ✅ Task DOC-001: Audit public docs against actual CLI and skill

Depends on: PM-001.

Acceptance criteria:

- Record `aget --help`, `aget get --help`, `aget current-tab --help`, and
  `aget session --help` snapshots or exact checked commands in `references.md`.
- List every stale README, skill, OpenCode tool, `project.md`, and workpad claim
  that conflicts with current CLI behavior.
- Classify every active Crawl4AI or `agent-browser` mention as historical,
  parity-only, code-facing, or removable.
- Verify command examples use current flags such as `--envelope`,
  `--content-format`, `--output`, `--wait-for-selector`, `--backend-option`,
  `--allow-domain`, `--browser-profile`, and `--chrome-profile`.

Status note:

- Completed with the 2026-05-24 audit in `references.md`. Current CLI help was
  captured for top-level, get, current-tab, session, authorize, import, browser
  import, and login start commands. The main stale surfaces are README
  compatibility backend guidance, skill compatibility-backend error guidance,
  OpenCode `crawl4ai.*` backend-option schema text, and `project.md` command
  sketches that predate the current CLI.

### ✅ Task DOC-002: Rewrite README as CLI-first current-state docs

Depends on: DOC-001.

Acceptance criteria:

- README starts from installed CLI usage and current supported commands.
- Commands match CLI help.
- No active Crawl4AI or `agent-browser` prerequisites, environment variables, or
  compatibility-backend setup remain in normal user docs.
- Historical basis appears only in a short, clearly labeled history note if it
  remains useful.
- README explains provider-session login injection versus target-site replay,
  Chrome/CDP support limits, and run-artifact retention.
- Roadmap names `doctor`, release artifacts, artifact lifecycle, and bounded
  `batch`/`map`/`crawl` as future CLI work.

Status note:

- Completed with the CLI-first README rewrite. Normal user docs now start from
  install, `get`, `current-tab`, and `session`; active Crawl4AI/`agent-browser`
  prerequisite and compatibility-backend sections were removed; provider-session
  replay scope, Chrome/CDP support limits, artifact retention, and the planned
  `doctor`/release/artifact/`batch`/`map`/`crawl` roadmap are documented.

### ✅ Task DOC-003: Align `skills/aget/SKILL.md` with README and CLI

Depends on: DOC-002.

Acceptance criteria:

- Skill uses release binary or local CLI examples only.
- Skill has no active compatibility-backend advice for Crawl4AI or
  `agent-browser`.
- Safety/session guidance stays explicit: agents do not collect credentials,
  bypass access controls, or use ambient browser auth without user action.
- README, skill, and `.opencode/tools/aget.ts` use the same command vocabulary.

Status note:

- Completed by removing active compatibility-backend guidance from the aget
  skill. The skill now keeps provider-session injection on the current
  Chrome/CDP login path, describes `backend_unavailable` in terms of current
  local components, and continues to use the README/CLI vocabulary.

## Phase 2: Historical Dependency Surface Removal

### ✅ Task DEP-001: Remove active Crawl4AI and agent-browser dependency surfaces

Depends on: DOC-001.

Acceptance criteria:

- Audit code, scripts, README, skills, OpenCode tools, active workpad routing,
  and tests for active Crawl4AI or `agent-browser` dependency surfaces.
- Remove production code paths, environment variables, scripts, and docs that
  imply Crawl4AI or `agent-browser` are supported runtime dependencies.
- Keep historical notes and source-reference rows only where they clearly
  describe prior influence, migration evidence, or parity research.
- Decide whether compatibility adapters and mock-command fixtures are deleted,
  moved behind dev-only historical tests, or archived. Record the decision
  before deleting code.
- Default-path errors and warnings use `AgetExtractor`, `AgetBrowser`, or
  generic primary extractor/browser wording, not old backend names.
- Confirm `cargo test` and a no-command-path smoke pass without Crawl4AI or
  `agent-browser` installed or on `PATH`.
- Verify with a tracked-file/source grep and record allowed historical matches.

Status note:

- Completed by deleting the command-backed compatibility adapters, scripts, and
  mock-command fixtures; renaming internal browser-state filtering away from old
  source-project vocabulary; and reducing tests to owned extractor/browser/cmux
  behavior. Remaining source-project names are limited to historical/parity
  workpad notes. Validation: `cargo test`, no-command-path smoke, and active
  source grep.

### ✅ Task CLI-001: Rename backend option namespace away from `crawl4ai.*`

Depends on: DEP-001.

Acceptance criteria:

- User-facing options use an `aget.*` or feature-owned namespace, or become
  first-class CLI flags where appropriate.
- CLI help, README, skill, tests, and OpenCode tool descriptions no longer
  expose `crawl4ai.*` as the current namespace.
- Pre-1.0 compatibility aliases are removed or rejected with clear errors unless
  a documented compatibility window is explicitly chosen.
- Unsupported option errors remain explicit and tested.

Status note:

- Completed by moving user-facing backend options to the `aget.*` namespace in
  CLI parsing, README/skill/OpenCode guidance, and tests. Unsupported old or
  unnamespaced options fail with explicit structured errors.

## Phase 3: Historical Parity Coverage

### ✅ Task PAR-001: Build upstream-test coverage matrix

Depends on: PM-001.

Acceptance criteria:

- Inventory Crawl4AI and `agent-browser` tests for features `aget` implements.
- Update `parity-matrix.md` with source path, commit, license note, local
  coverage path, status, and gap for each relevant behavior.
- Mark out-of-scope or unimplemented upstream behavior as `not-applicable`
  instead of creating implied commitments.
- Keep matrix scope to implemented features: fetch, rendered waits,
  markdown/html/text/json output, selectors/exclusions, session replay,
  current-tab, Chrome import, login lifecycle, provider-session injection,
  cleanup, error classification, and redaction.

Status note:

- Completed with `workpads/post-migration/parity-matrix.md`. The matrix
  records Apache-2.0 source snapshots, upstream test paths, current local
  coverage paths, covered/partial/not-applicable status, and concrete PAR-002
  actions. Out-of-scope upstream behavior such as deep crawl, hosted/server
  APIs, screenshots, interactive browser actions, and `doctor` is explicitly
  routed to later backlog tasks or marked not-applicable.

### ✅ Task PAR-002: Add missing parity/regression tests

Depends on: PAR-001, DEP-001.

Acceptance criteria:

- Each `implemented but uncovered` matrix row gets a deterministic local test or
  an explicit product-decision deviation.
- Tests do not require external services, real credentials, Crawl4AI, or
  `agent-browser`.
- No incompatible upstream code or tests are copied.

Status note:

- Completed with behavior-level local tests for raw HTML edge cases, GFM table
  formatting, same-scope login-session replacement, and session-backed failure
  metadata redaction. Existing deterministic coverage was recorded for CSS
  waits, selector fallback semantics, browser-state cookie/local/session-storage
  filtering, domain scope, and provider-session injection at the public API
  layer. The only matrix deviation is CLI-level provider-session fake-browser
  testing, deferred until the CLI has a supported test backend seam.

### ✅ Task PAR-003: Fix failures discovered by parity tests

Depends on: PAR-002.

Acceptance criteria:

- Failing parity rows become focused bug subtasks.
- Implementation is fixed or the feature is downgraded/documented.
- Focused parity tests, `cargo fmt --check`, and `cargo test` pass.

Status note:

- Completed during the PAR-002 test pass. The new raw-fragment parity test
  exposed that fragment-only links in `raw:` input were being resolved against
  the whole synthetic `raw:` URL. The owned extractor now avoids using `raw:`
  inputs as markdown base URLs unless an explicit `aget.base_url` or `<base>`
  tag is provided. Focused tests, `cargo fmt --check`, `git diff --check`, and
  full `cargo test` pass.

## Phase 4: CLI Product Backlog

### ✅ Task DR-001: Design `aget doctor`

Depends on: DOC-002.

Acceptance criteria:

- Define checks for binary version/build info, `AGET_HOME`, directory/file
  permissions, session store, run artifacts, Chrome/CDP availability,
  current-tab prerequisites, optional cmux, and OpenCode tool binary resolution
  if configured.
- Do not check for Crawl4AI or `agent-browser`.
- Define human-readable output and `--envelope json` output.
- Define redaction policy for diagnostics.

Status note:

- Completed in `workpads/post-migration/doctor-design.md`. The design defines
  check IDs/categories/statuses, human and JSON output, exit semantics, optional
  component handling, and redaction policy. Doctor must not check for Crawl4AI,
  `agent-browser`, MCP, hosted services, or arbitrary browser ports.

### ✅ Task DR-002: Implement `aget doctor`

Depends on: DR-001.

Acceptance criteria:

- Add `aget doctor`.
- Reports `ok`, `warn`, and `fail` diagnostics.
- Static fetch support does not fail just because Chrome is missing.
- Tests cover healthy, missing optional dependency, bad permissions, and JSON
  shape.

Status note:

- Completed with a CLI `doctor` command, human output, JSON envelope output,
  check filtering, and diagnostics for binary/store/artifacts/Chrome/current-tab
  prerequisites/cmux/OpenCode. Missing optional Chrome or cmux reports `warn`
  instead of failing static-fetch readiness. Tests cover parser shape, help,
  missing optional dependencies, JSON shape, and loose session-file permissions.

### ✅ Task REL-001: Package/release plan

Depends on: DOC-002, DR-002.

Acceptance criteria:

- Define install targets, `cargo install --path .`, release binary naming,
  checksums, license/changelog/version policy, and smoke gate.
- Defer notarized/app-bundle work unless explicitly requested; this is a CLI.
- Release checklist includes README/skill sync and no-command-path smoke.

Status note:

- Completed in `workpads/post-migration/release-plan.md`. The plan defines
  checkout install, release tarball targets, binary/archive naming, checksum
  files and manifest, version policy, license/changelog requirements, and a
  release smoke gate. It explicitly defers notarized/app-bundle packaging and
  keeps release scope to CLI artifacts. The checklist includes README/skill
  sync, no-command-path smoke, `doctor`, and stale dependency-surface grep.

### ✅ Task REL-002: Produce release artifacts

Depends on: REL-001.

Acceptance criteria:

- Release build artifacts and checksums are generated reproducibly.
- README install section matches produced artifacts.
- `aget doctor` validates release-binary basics.

Status note:

- Completed for the current verified host target `aarch64-apple-darwin`.
  Generated `dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz`, a per-archive
  `.sha256`, and `dist/SHA256SUMS` from `target/release/aget` plus README and
  LICENSE. The archive build normalizes file mtimes, owner/group metadata, and
  gzip timestamp data; a repeat packaging pass produced the same SHA-256
  (`b8d245b882a6f3ce3643edc777cf39e3e72cf1aea091300839d22401dec917b8` after
  regenerating for the BATCH-001 binary).
  README install instructions name the produced artifact, and release-binary
  `doctor --quick` passed with `ok: true`.

### ✅ Task ART-001: Design artifact lifecycle commands

Depends on: DOC-002.

Acceptance criteria:

- Choose command shape, preferably `aget artifacts list/inspect/delete/prune`.
- Define retention config, dry-run behavior, sensitive metadata redaction, and
  confirmation rules.
- Commands never delete caller-owned `--output` files outside `AGET_HOME`.

Status note:

- Completed in `workpads/post-migration/artifact-lifecycle-design.md`. The
  design chooses `aget artifacts list/inspect/delete/prune`, defines JSON and
  human output, run ID validation, retention flags/config names, dry-run and
  `--yes` confirmation behavior, sensitive metadata URL redaction, and strict
  deletion boundaries. Caller-owned `--output` files outside the selected
  `AGET_HOME/runs/<run-id>` directory are reported as external and preserved.

### ✅ Task ART-002: Implement artifact lifecycle commands

Depends on: ART-001.

Acceptance criteria:

- List run directories, sizes, age, sensitivity, and source URL metadata.
- Delete or prune only internal artifacts with confirmation or `--yes`.
- Include JSON envelopes and tests.

Status note:

- Completed with `aget artifacts list`, `inspect`, `delete`, and `prune`.
  JSON envelopes use command names `artifacts.*`; list reports run IDs,
  directories, ages, sizes, sensitivity, source URL metadata, content ownership,
  and metadata validity. Delete requires `--yes` and removes only internal run
  directories, preserving caller-owned external `--output` paths. Prune supports
  `--older-than`, `--keep-last`, `--max-bytes`, default dry-run behavior, and
  `--yes` execution. Tests cover parser/help, list/inspect, delete
  confirmation and external-output preservation, prune dry-run/delete behavior,
  and prune usage errors.

### ✅ Task BACKLOG-001: Design `batch`, `map`, and `crawl` backlog

Depends on: DOC-002, ART-001.

Acceptance criteria:

- Define CLI shapes for bounded `aget batch`, `aget map`, and `aget crawl`.
- Define safety limits: same-origin/path defaults, maximum pages, concurrency,
  rate controls, session behavior, output artifacts, and partial failures.
- Keep implementation CLI-first with no server or MCP layer.

Status note:

- Completed in `workpads/post-migration/batch-map-crawl-design.md`. The design
  defines CLI shapes for `batch`, `map`, and `crawl`; shared JSON/manifest
  output; same-origin/path defaults; required crawl `--limit`; max concurrency,
  crawl limit, max depth, and per-host delay boundaries; session replay behavior;
  output artifact layout; and deterministic partial-failure semantics. It keeps
  the implementation CLI-only with no server or MCP layer.

### ✅ Task BATCH-001: Implement `aget batch`

Depends on: BACKLOG-001, ART-002.

Acceptance criteria:

- Fetch explicit URL lists from args, file, or stdin.
- Use bounded concurrency.
- Emit per-URL envelopes/artifacts.
- Report partial failures deterministically.

Status note:

- Completed with `aget batch` for positional URL lists, newline-delimited
  `--file` input, and newline-delimited `--stdin` input. The command uses
  bounded thread concurrency, writes `manifest.json`, `manifest.md`, and
  per-item content artifacts under `--output-dir` or an `AGET_HOME` batch run
  directory, marks exact duplicate inputs as skipped, and returns a non-zero
  exit code when any supported input fails. JSON envelope output uses
  `command=batch` and embeds the batch manifest summary/items. Tests cover
  parser/help, success artifacts, file input duplicates, stdin input, usage
  errors, and partial failure behavior. Implementation also fixed shared
  concurrent fetch safety by making run-artifact and temp-state file names
  unique within a process.

### 📋 Task MAP-001: Implement `aget map`

Depends on: BACKLOG-001.

Acceptance criteria:

- Extract and dedupe links from one URL or artifact.
- Filter by host, path, and content type.
- Do not recursively crawl.
- Provide JSON and markdown output.

### 📋 Task CRAWL-001: Implement bounded `aget crawl`

Depends on: BATCH-001, MAP-001.

Acceptance criteria:

- Use a map plus get pipeline.
- Require `--limit`.
- Default to same-origin/path-bounded traversal.
- Record crawl manifest and artifacts.
- Do not add site-specific bypass behavior.
