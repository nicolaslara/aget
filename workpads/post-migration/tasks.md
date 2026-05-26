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
  (`e95e4ddc16a3bf59128ec02ad92ae83ac73d8e25d74d78338391cf93bb2e832a` after
  regenerating for the CRAWL-001 binary).
  README install instructions name the produced artifact, and release-binary
  `doctor --quick` passed with `ok: true`.

### ✅ Task REL-003: Publish v0.1.0 GitHub release and define install path

Depends on: REL-002.

Acceptance criteria:

- Publish a GitHub Release for `v0.1.0` from the current release commit.
- Attach the verified local release artifact and checksums:
  - `dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz`
  - `dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz.sha256`
  - `dist/SHA256SUMS`
- Release notes summarize user-visible scope:
  - CLI-only release.
  - Owned default extractor/browser/session paths.
  - `doctor`, `artifacts`, `batch`, `map`, and bounded `crawl`.
  - Auth/session support including explicit session replay and
    `login start --session <provider>`.
  - Known limits: Chrome/CDP required for browser-backed flows, cmux optional,
    macOS ARM artifact only for this first published binary, no server/MCP
    layer, no notarized app bundle, no Windows artifact yet.
- Verify installation from the attached tarball, not only from
  `target/release/aget`:

```bash
tmpdir="$(mktemp -d)"
tar -xzf dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz -C "$tmpdir"
"$tmpdir/aget-v0.1.0-aarch64-apple-darwin/aget" --version
"$tmpdir/aget-v0.1.0-aarch64-apple-darwin/aget" --envelope json doctor --quick
rm -rf "$tmpdir"
```

- Document supported install paths in README and/or release notes:
  - `cargo install --path .` for local checkout development.
  - `cargo install --git https://github.com/nicolaslara/aget` for source
    install from GitHub.
  - tarball download/unpack for binary install.
- Document and verify global skill installation:
  - local checkout install into `$CODEX_HOME/skills/aget` from `skills/aget`.
  - release/source install instructions for making the `aget` skill globally
    available to Codex.
  - note that Codex must be restarted to pick up newly installed skills.
- Record the release URL, artifact hash, install-smoke commands, and results in
  `workpads/post-migration/references.md`.
- Create follow-up tasks if not completed in this task:
  - GitHub Actions multi-target release builds for macOS ARM, macOS Intel,
    Linux x86_64, and Linux ARM64.
  - Homebrew tap/formula.
  - crates.io publish decision.
  - Windows artifact only after a real Windows smoke path exists.

Status note:

- Completed with the published `v0.1.0` GitHub Release:
  `https://github.com/nicolaslara/aget/releases/tag/v0.1.0`. The release
  targets commit `b88ff890f0eb114ffdef380b85cd6637cdc5ccaa` and attaches the
  macOS ARM tarball plus checksum files. The tarball includes `aget`,
  `README.md`, `LICENSE`, and `skills/aget/SKILL.md`; SHA-256 is
  `b02a6f348a5adfbfd280b7b915cc485095dd4f67990c04d700871e8b37763182`.
  Downloaded-asset checksum, tarball `aget --version`, tarball
  `doctor --quick`, packaged-skill install, local `cargo install --path`, and
  fresh `cargo install --git ... aget` source-install smokes passed.

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

### ✅ Task MAP-001: Implement `aget map`

Depends on: BACKLOG-001.

Acceptance criteria:

- Extract and dedupe links from one URL or artifact.
- Filter by host, path, and content type.
- Do not recursively crawl.
- Provide JSON and markdown output.

Status note:

- Completed with `aget map` for one fetched URL or one successful internal
  `aget get` artifact run. URL mode fetches source HTML through the existing
  get pipeline; artifact mode reads only internal run content and rejects
  caller-owned external `--output` paths. Link extraction deduplicates
  normalized absolute URLs, drops fragments, skips non-page schemes, defaults
  to same-origin and same-path filtering, supports `--any-origin`,
  `--any-path`, simple `--include`/`--exclude` patterns, `--content-type`
  inference filters, and `--max-links`. Output includes JSON envelope data plus
  local `links.json`, `links.md`, and run metadata under `AGET_HOME/runs`.
  Tests cover parser/help, URL mapping and filters, artifact mapping, external
  artifact rejection, and conflicting inputs.

### ✅ Task CRAWL-001: Implement bounded `aget crawl`

Depends on: BATCH-001, MAP-001.

Acceptance criteria:

- Use a map plus get pipeline.
- Require `--limit`.
- Default to same-origin/path-bounded traversal.
- Record crawl manifest and artifacts.
- Do not add site-specific bypass behavior.

Status note:

- Completed with `aget crawl <url> --limit <n>`. The command requires
  `--limit`, caps v1 crawls at 100 pages, depth 5, and concurrency 8, and
  traverses with same-origin and same-path defaults unless widened by
  `--any-origin`, `--any-path`, or `--allow-domain`. Each fetched page uses the
  existing `get` pipeline, stores per-page artifacts, discovers next links from
  fetched HTML, and records `manifest.json` plus `manifest.md` under
  `--output-dir` or `AGET_HOME/runs`. Partial failures are recorded in the
  manifest and return a non-zero exit code without adding site-specific bypass
  behavior. Tests cover parser/help, required limit and bounds, bounded
  same-path traversal, manifest artifacts, and partial failure output.

## Phase 5: Next Product Backlog

### ✅ Task AGENT-001: Expose current CLI surface in agent integrations

Depends on: BATCH-001, MAP-001, CRAWL-001.

Acceptance criteria:

- Add `.opencode/tools/aget.ts` wrappers for `batch`, `map`, `crawl`,
  `artifacts list/inspect`, and `doctor`.
- Update `skills/aget/SKILL.md` with when to choose `get`, `batch`, `map`, and
  `crawl`, including private-content and artifact-first guidance.
- Keep the integration CLI-first: no server, MCP, daemon, or direct Rust API
  dependency.
- Tests or deterministic command snapshots cover the generated CLI args for new
  wrappers.
- README OpenCode/skill guidance names the new multi-URL commands.

Status note:

- Completed with thin OpenCode CLI/envelope wrappers for `batch`, `map`,
  `crawl`, `artifacts list`, `artifacts inspect`, and `doctor`. The aget skill
  now has command-choice guidance for `get`, `batch`, `map`, `crawl`,
  artifacts, and doctor, including artifact-first/private-content routing.
  README names the new wrappers and states the integration remains CLI-only.
  Deterministic Bun snapshots cover generated CLI args, and direct wrapper
  smoke exercised `doctor`, `batch`, `map`, `crawl`, and `artifacts list`
  against `target/release/aget`.

### ✅ Task CACHE-001: Add cache, freshness, and token/cost metadata

Depends on: ART-002, BATCH-001, MAP-001, CRAWL-001.

Acceptance criteria:

- Define cache storage under `AGET_HOME/cache` with clear keys, TTL/freshness
  behavior, and privacy boundaries.
- Add CLI controls such as `--fresh`, `--cache-policy`, or an equivalent
  current-state design before implementation.
- Add metadata for cache hit/miss, fetched bytes, content bytes, estimated
  tokens, and elapsed timing in envelopes and artifact metadata.
- Ensure authenticated/session-backed content does not become reusable across
  scopes or users by accident.
- Add tests for cache hit/miss behavior, stale refresh, metadata shape, and
  session/privacy boundaries.

Status note:

- Completed with cache storage under `AGET_HOME/cache`, shared `--fresh`,
  `--cache-policy`, and `--cache-ttl` controls for `get`, `batch`, `map`, and
  `crawl`; cache/usage metadata in envelopes, artifacts, and multi-URL
  manifests; and privacy boundaries that keep session-backed, current-tab,
  `raw:`, and `file://` content out of reusable cache entries. Tests cover
  hit/miss, stale refresh, metadata shape, and session-backed ineligibility.

### ✅ Task SEARCH-001: Add objective and keyword narrowing

Depends on: ART-002.

Acceptance criteria:

- Define a deterministic local narrowing command or flags, such as
  `aget search-page --artifact <run-id> --query <text>` or
  `aget get <url> --objective <text>`.
- Narrow by headings, anchors, paragraphs, lists, tables, and keyword/snippet
  scoring without requiring an LLM inside `aget`.
- Preserve artifact-first operation so large/private pages can be searched from
  local files without embedding private content in envelopes.
- Output JSON and markdown snippets with source URL, heading/path context, and
  character offsets or stable section identifiers where practical.
- Add tests for heading ranking, keyword matches, no-match output, and sensitive
  artifact handling.

Status note:

- Completed with artifact-first `aget search-page --artifact <run-id> --query
  <text>`. The command uses deterministic local section scoring over existing
  page artifacts, writes JSON/markdown search result artifacts, returns
  snippets with section IDs and character offsets, handles no-match output
  without failure, and requires `--allow-private-content` before emitting
  snippets from sensitive artifacts. OpenCode exposes `aget_search_page`.

### 📋 Task EXTRACT-001: Add structured extraction from artifacts

Depends on: ART-002, SEARCH-001.

Acceptance criteria:

- Define a CLI surface for structured extraction from a page or crawl artifact,
  such as `aget extract --artifact <run-id> --schema <path>` or
  `aget extract --manifest <path> --field <name>`.
- Support deterministic extraction primitives first: tables, links, headings,
  definition lists, metadata fields, and JSON/HTML selectors.
- Keep model/LLM interpretation outside `aget` unless a later task explicitly
  approves a local-only structured inference backend.
- Output schema-valid JSON with provenance to source URL/artifact/selector.
- Add tests for table extraction, selector extraction, crawl-manifest
  extraction, malformed schema/input errors, and private artifact behavior.

Status note:

- Pending. Current `--content-format json` wraps extracted content, but it does
  not provide task-specific structured data extraction.

### 📋 Task REL-004: Automate CI and multi-target release builds

Depends on: REL-003.

Acceptance criteria:

- Add GitHub Actions for `cargo fmt --check`, `cargo test`, release build, and
  stale dependency-surface grep.
- Add release workflow or documented manual workflow for macOS ARM, macOS Intel,
  Linux x86_64, and Linux ARM64 tarballs with checksums.
- Keep Windows artifacts deferred until a real Windows smoke path exists.
- Ensure generated release artifacts include `aget`, README, LICENSE, and
  checksum files matching `release-plan.md`.
- Record CI status and release-build evidence in `references.md`.

Status note:

- Pending. Current release artifacts are produced manually on the local host.

### 📋 Task REL-005: Define Homebrew tap/formula path

Depends on: REL-003.

Acceptance criteria:

- Decide whether to publish a dedicated tap, contribute to an existing tap, or
  defer Homebrew until multi-target automation exists.
- Draft a formula for the published release artifact with SHA-256 verification.
- Include install and smoke commands for `aget --version` and
  `aget --envelope json doctor --quick`.
- Record the decision and evidence in `references.md`.

Status note:

- Pending. `v0.1.0` is published as a GitHub Release, but there is no Homebrew
  formula or tap path yet.

### 📋 Task REL-006: Decide crates.io publish policy

Depends on: REL-003.

Acceptance criteria:

- Decide whether `aget` should be published to crates.io now, later, or never.
- Check package metadata needed for crates.io, including description, README,
  license, repository, include/exclude rules, and workspace package shape.
- If publishing is deferred, document the reason and keep `cargo install --git`
  as the supported source install path.
- Record the decision and any package dry-run evidence in `references.md`.

Status note:

- Pending. `cargo install --git ... aget` is verified for `v0.1.0`; crates.io
  remains undecided.

### 📋 Task REL-007: Add Windows artifact only after a real smoke path

Depends on: REL-004.

Acceptance criteria:

- Define a Windows build and smoke environment for `aget --version`,
  `doctor --quick`, static `get`, artifact commands, and no-command-path
  behavior.
- Keep Windows release artifacts unpublished until that smoke path passes.
- If Windows support remains deferred, document the concrete blocker and the
  minimum test matrix needed to publish confidently.

Status note:

- Pending. `v0.1.0` intentionally publishes no Windows artifact.

### ✅ Task REL-008: Simplify global Codex skill installation

Depends on: REL-003.

Acceptance criteria:

- Add a checkout-friendly helper for installing `skills/aget` into
  `$CODEX_HOME/skills/aget`.
- Keep release-tarball skill installation documented and compatible with the
  packaged `skills/aget/SKILL.md`.
- Verify the helper with a temporary `CODEX_HOME`.
- Install the local checkout skill into the user's Codex skill directory and
  verify `SKILL.md` resolves there.

Status note:

- Completed with `scripts/install-codex-skill.sh`, README/skill/release-plan
  updates, temporary copy/symlink validation, and a local global symlink install
  at `/Users/nicolas/.codex/skills/aget`. Codex must be restarted before a
  running session sees a newly installed or replaced global skill.

### 📋 Task DEBUG-001: Add screenshots and debug traces

Depends on: ART-002, DR-002.

Acceptance criteria:

- Define screenshot/debug artifact policy for browser-backed extraction,
  current-tab, and failed extraction diagnostics.
- Add explicit CLI flags for screenshot or trace capture; do not capture visual
  private content by default.
- Store screenshots/traces under run artifacts with sensitivity metadata and
  artifact lifecycle compatibility.
- Ensure `artifacts inspect/delete/prune` understands the new files.
- Add tests for opt-in behavior, metadata shape, lifecycle cleanup, and
  redaction/sensitivity flags.

Status note:

- Pending. Browser/CDP extraction exists, but visual/debug artifacts are not yet
  exposed.

### 📋 Task ACT-001: Design safe generic interact/actions model

Depends on: SEARCH-001, DEBUG-001.

Acceptance criteria:

- Design a generic CLI action model for user-authorized local browser flows:
  click, type, wait, select, submit, extract, and capture.
- Define consent, audit, redaction, timeout, confirmation, and private-content
  boundaries before implementation.
- Keep the binary generic; do not add site-specific login/paywall/CAPTCHA
  bypass behavior or site-shaped retry advice.
- Define JSON envelope output, artifact layout, and failure semantics.
- Include a test strategy using deterministic local pages and fake-browser seams
  where possible.

Status note:

- Pending design task. Interact/actions are high value for authenticated app
  flows, but they need a stricter safety and audit model before implementation.
