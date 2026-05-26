# Post-Migration References

## Current Product Surfaces

| Surface | Path / Command | Use |
| --- | --- | --- |
| CLI help | `cargo run -- --help`, `cargo run -- get --help`, `cargo run -- current-tab --help`, `cargo run -- session --help` | Source for README and skill command examples. |
| README | `README.md` | User-facing install, usage, safety, and roadmap guidance. |
| Agent skill | `skills/aget/SKILL.md` | Agent-facing CLI workflow guidance. |
| OpenCode tool | `.opencode/tools/aget.ts` | Thin CLI wrapper surface. Keep aligned with CLI/help and README. |
| Product intent | `project.md` | Original product goals and feature inventory; update stale pre-migration wording through explicit tasks. |
| Migration evidence | `workpads/research/experiments/2026-05-24-final-migration-release-and-browser-smokes.md` | Final owned-backend migration validation record. |

## Historical / Parity Sources

These are not active runtime dependencies. Use them only for historical evidence,
license-aware behavior comparison, and parity-test coverage work.

| Project | Local / Source | Use |
| --- | --- | --- |
| Crawl4AI | `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255` | Historical extraction/readability/markdown behavior and parity-test comparison. License: Apache-2.0. |
| agent-browser | `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7` | Historical browser/CDP/session/profile behavior and parity-test comparison. License: Apache-2.0. |
| cmux | Source review at commit `7142e31d3a749c241843655cac2771927505860c` | Optional session import helper; cookie output must be post-filtered by explicit allowlist. |

## Audit Queries

Use these during DOC-001 and DEP-001:

```bash
rg -n "MCP|Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND" \
  AGENTS.md project.md README.md skills .opencode workpads src tests scripts
```

```bash
cargo run -- --help
cargo run -- get --help
cargo run -- current-tab --help
cargo run -- session --help
```

## DOC-001 Audit: 2026-05-24

CLI help captured with:

```bash
cargo run -- --help
cargo run -- get --help
cargo run -- current-tab --help
cargo run -- session --help
cargo run -- session authorize --help
cargo run -- session import --help
cargo run -- session import browser --help
cargo run -- session login start --help
```

Current top-level commands are `get`, `current-tab`, and `session`. There is no
implemented `doctor`, `status`, `batch`, `map`, or `crawl` command yet.

Stale or cleanup-required public-doc findings:

| Surface | Classification | Finding | Follow-up |
| --- | --- | --- | --- |
| `README.md` | removable | Normal prerequisites still mention optional Crawl4AI and `agent-browser` command adapters via `AGET_CRAWL4AI_COMMAND` and `AGET_AGENT_BROWSER_COMMAND`. | DOC-002 removes from normal user docs; DEP-001 decides code/test removal. |
| `README.md` | removable | A live "Compatibility Backends" section documents command-backed Crawl4AI and `agent-browser` usage. | DOC-002 removes or reduces to a short historical note; DEP-001 removes runtime surfaces. |
| `README.md` | code-facing | CLI reference lists many `crawl4ai.*` backend options as current `AgetExtractor` options. | CLI-001 renames/removes the namespace; DOC-002 should not center it as user guidance. |
| `README.md` | stale roadmap | Roadmap says broader crawling is future, but does not name the new planned sequence: `doctor`, release artifacts, artifact lifecycle, bounded `batch`/`map`/`crawl`. | DOC-002 updates roadmap. |
| `skills/aget/SKILL.md` | removable | Error guidance says `backend_unavailable` can mean Crawl4AI or `agent-browser` is missing. | DOC-003 removes old compatibility-backend advice. |
| `skills/aget/SKILL.md` | removable | Provider-session section tells agents to unset `AGET_AGENT_BROWSER_COMMAND` if a compatibility backend is selected. | DOC-003 removes once compatibility surfaces are no longer active guidance. |
| `.opencode/tools/aget.ts` | code-facing | `backend_options` description exposes the `crawl4ai.*` namespace. | CLI-001 updates tool schema text after namespace decision. |
| `project.md` | stale current-state | Initial command sketch uses `aget fetch`, `aget status`, `aget map`, and `aget crawl`, which do not match current CLI help. | DOC-002 or a follow-up project-doc task should label this as historical/original goal and point current users to README/help. |
| `workpads/research/*` | historical | Crawl4AI and `agent-browser` mentions remain in archived research/migration evidence. | Allowed historical matches; do not rewrite unless they route active work incorrectly. |
| `workpads/post-migration/*` | parity-only | Crawl4AI and `agent-browser` mentions are scoped to historical dependency cleanup and parity tracking. | Allowed. |

Current flag vocabulary verified from help:

- `--envelope`, `--content-format`, `--output`, `--wait-for-selector`,
  `--backend-option`, `--allow-domain`, `--browser-profile`, and
  `--chrome-profile` are present on the relevant commands.
- `--json`, `--out`, `--format`, `--wait-for`, and `--extractor-option` are not
  current public flags in CLI help.

## DEP-001 / CLI-001 Closure: 2026-05-24

Decision:

- Delete compatibility adapters and mock-command fixtures instead of retaining
  them as dev-only tests.
- Treat source-project names as allowed only in this workpad's historical and
  parity sections.
- Use `aget.*` for current backend-option names.

Validation evidence:

```bash
cargo fmt --check
cargo test
env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" AGET_HOME="$(mktemp -d)" \
  target/debug/aget --envelope json get 'raw:<main><h1>No Command Path</h1></main>'
git diff --check
rg -n "Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND|AgentBrowser|agent_browser" \
  README.md skills/aget/SKILL.md .opencode src scripts tests workpads/post-migration -S
```

Allowed grep matches after closure are limited to historical/parity rows in
`workpads/post-migration/`.

## PAR-001 Inventory: 2026-05-24

Source snapshots verified with:

```bash
git -C references/repos/crawl4ai rev-parse HEAD
git -C references/repos/agent-browser rev-parse HEAD
```

Relevant upstream test discovery used:

```bash
rg --files references/repos/crawl4ai references/repos/agent-browser \
  | rg '(^|/)(test|tests|spec|specs|__tests__)|(_test|\.test\.|\.spec\.)'
rg -n "test_|def test|async def test|#\[test\]|async fn" \
  references/repos/crawl4ai/tests/regression/test_reg_core_crawl.py \
  references/repos/crawl4ai/tests/regression/test_reg_content.py \
  references/repos/crawl4ai/tests/test_raw_html_edge_cases.py \
  references/repos/crawl4ai/tests/test_issue_1484_css_selector.py \
  references/repos/crawl4ai/tests/test_table_gfm_compliance.py \
  references/repos/agent-browser/cli/src/native/e2e_tests.rs \
  references/repos/agent-browser/cli/src/native/parity_tests.rs \
  references/repos/agent-browser/cli/tests/doctor_cli.rs
```

Local coverage discovery used:

```bash
rg -n "fn .*\(|#\[test\]" \
  tests/mock_site_cli.rs tests/get_cli tests/cli/current_tab.rs \
  tests/aget_api src/session tests/session_cli tests/mock_site_browser \
  tests/mock_site_docs_contract
```

Out-of-scope upstream behavior for this matrix:

- Crawl4AI deep crawl, Docker/server API, cache database modes, hooks, network
  capture, LLM extraction, schema extraction, proxy/anti-bot, and broad batch
  APIs. These are not implemented `aget` features yet.
- agent-browser interactive actions such as click/type/drag/upload/tabs,
  Electron automation, HAR/vitals/React tree, credentials vault, daemon/socket
  lifecycle, and `doctor`. These are not current `aget` features; `doctor` is
  tracked under DR-001/DR-002.
- Screenshot output is not currently an `aget get` artifact contract. Add it to
  parity only if screenshot artifacts become implemented.

## PAR-002 Focused Validation: 2026-05-24

Focused tests used while adding deterministic parity coverage:

```bash
cargo fmt --check
cargo test --lib merge_login_session_replaces_stale_same_scope_state
cargo test --test get_cli get_raw_html_handles_edge_case_inputs
cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice
cargo test --test mock_site_cli mock_site_fetch_handles_redirect_output_shaping_and_waits
cargo test --test aget_api aget_session_failure_metadata_redacts_sensitive_backend_error
cargo test --test aget_api aget_start_login_session_injects_named_sessions_into_browser_backend
cargo test --lib session::browser_state::tests::filters_playwright_state_with_same_import_rules
```

Final PAR-002/PAR-003 gate:

```bash
cargo fmt --check
git diff --check
cargo test
```

## DR-001 Design: 2026-05-24

Design artifact:

- `workpads/post-migration/doctor-design.md`

Current implementation surfaces reviewed for DR-001:

- `src/session/store/mod.rs` and `src/session/store/permissions.rs` for
  `AGET_HOME`, layout, and permissions.
- `src/browser_cdp/chrome_process/launch.rs` for Chrome command discovery.
- `src/session/cmux/command.rs` for `AGET_CMUX_COMMAND` and cmux optionality.
- `.opencode/tools/aget.ts` for `AGET_OPENCODE_BIN` wrapper resolution.
- `src/main_envelope.rs` for structured JSON envelope shape.

## DR-002 Implementation Validation: 2026-05-24

Focused tests for the new doctor command:

```bash
cargo test --lib cli::tests::doctor
cargo test --test cli doctor
```

Final gate for the implementation pass:

```bash
cargo fmt --check
git diff --check
cargo test
```

Review-follow-up docs touched:

- `README.md`: provider-session bootstrap versus replay-scope guardrail,
  import/source-profile retention, and `aget doctor` command reference.
- `skills/aget/SKILL.md`: provider-session guidance and import/custom-profile
  retention guidance.

## REL-001 Release Plan: 2026-05-24

Design artifact:

- `workpads/post-migration/release-plan.md`

Current package metadata reviewed:

- `Cargo.toml`: package `name = "aget"`, `version = "0.1.0"`,
  `license = "MIT"`.
- REL-002 added top-level `LICENSE` and `CHANGELOG.md` before producing release
  archives.

Validation commands run for this planning pass:

```bash
cargo fmt --check
git diff --check
cargo run --quiet -- --help
cargo run --quiet -- doctor --help
tmpdir="$(mktemp -d)"
env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  AGET_HOME="$tmpdir/aget-home" \
  target/debug/aget --envelope json get 'raw:<main><h1>No Command Path</h1></main>'
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" target/debug/aget --envelope json doctor --quick
rm -rf "$tmpdir"
rg -n '(--json|--out\b|backend\.key|Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND)' \
  README.md skills/aget/SKILL.md .opencode/tools/aget.ts scripts workpads/post-migration -S
```

The grep matched only historical/parity workpad mentions for old dependency
names and the DOC-001 note that `--json`/`--out` are not current public flags.
`scripts/demo_real_cli.sh` was updated to the current `--envelope json` and
`--output` flags.

## REL-002 Artifact Production: 2026-05-24

Produced files:

```text
dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz
dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz.sha256
dist/SHA256SUMS
```

Final archive SHA-256:

```text
ea802299df748a61ccd20306482af4cf7018847decb790e0353fdc95560128d5  dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz
```

Packaged binary SHA-256:

```text
9c7b1d09b8496a4bdaffca16789faacd203446675e86a8607156c1ae85de4b8f  dist/aget-v0.1.0-aarch64-apple-darwin/aget
```

Artifact generation command:

```bash
cargo build --release
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
target=$(rustc -vV | sed -n 's/^host: //p')
name="aget-v${version}-${target}"
archive="${name}.tar.gz"
rm -rf dist
mkdir -p "dist/${name}"
install -m 755 target/release/aget "dist/${name}/aget"
install -m 644 README.md "dist/${name}/README.md"
install -m 644 LICENSE "dist/${name}/LICENSE"
find "dist/${name}" -exec touch -t 202605240000 {} +
COPYFILE_DISABLE=1 tar --format ustar --uid 0 --gid 0 --uname root --gname wheel -C dist -cf "dist/${name}.tar" "${name}"
gzip -n "dist/${name}.tar"
shasum -a 256 "dist/${archive}" > "dist/${archive}.sha256"
(cd dist && shasum -a 256 "$archive" > SHA256SUMS)
```

Reproducibility check:

```bash
before=$(shasum -a 256 dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz | awk '{print $1}')
# rerun the artifact generation command above
after=$(shasum -a 256 dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz | awk '{print $1}')
test "$before" = "$after"
```

Release smoke validation:

```bash
cargo fmt --check
git diff --check
cargo test
target/release/aget --help
target/release/aget get --help
target/release/aget current-tab --help
target/release/aget session --help
target/release/aget doctor --help
tmpdir="$(mktemp -d)"
env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json get 'raw:<main><h1>No Command Path</h1></main>'
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" target/release/aget --envelope json doctor --quick
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
cargo install --path . --locked --root "$tmpdir/install"
"$tmpdir/install/bin/aget" --version
AGET_HOME="$tmpdir/aget-home" "$tmpdir/install/bin/aget" --envelope json doctor --quick
rm -rf "$tmpdir"
```

Archive validation:

```bash
tar -tvzf dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz
cmp README.md dist/aget-v0.1.0-aarch64-apple-darwin/README.md
cmp LICENSE dist/aget-v0.1.0-aarch64-apple-darwin/LICENSE
dist/aget-v0.1.0-aarch64-apple-darwin/aget --version
rg -n 'Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND|AgentBrowser|agent_browser' \
  README.md skills/aget/SKILL.md .opencode/tools/aget.ts src tests scripts -S
```

Validation result:

- `cargo test`: 155 lib tests passed, 2 ignored; integration tests passed
  including CLI, API, get, mock-site, docs-contract, and session CLI suites.
- No-command-path smoke returned `ok: true` with `# No Command Path`.
- Release-binary `doctor --quick` returned `ok: true` with one non-failing
  OpenCode PATH warning.
- Installed temp-root binary reported `aget 0.1.0` and `doctor --quick`
  returned `ok: true`.
- Active source grep found no old Crawl4AI or `agent-browser` runtime surfaces
  in README, skill, OpenCode tool, `src`, `tests`, or scripts.
- After ART-002 changed the binary and README, the archive was regenerated with
  SHA-256 `ea802299df748a61ccd20306482af4cf7018847decb790e0353fdc95560128d5`.

## ART-001 Artifact Lifecycle Design: 2026-05-24

Design artifact:

- `workpads/post-migration/artifact-lifecycle-design.md`

Current implementation surfaces reviewed:

- `src/extraction/mod.rs`: creates `AGET_HOME/runs/<run-id>` and chooses
  caller-provided `--output` paths outside the run directory when requested.
- `src/extraction/pipeline/direct.rs`: direct/raw/file extraction uses the same
  run directory and caller-output behavior.
- `src/extraction/pipeline/finalization.rs`: writes content and metadata paths
  into `GetSuccess.artifacts`.
- `src/extraction/artifacts/metadata.rs`: current `metadata.json` fields for
  success and failure runs.
- `src/session/store/mod.rs`: `AGET_HOME` layout includes `runs`, `sessions`,
  `cache`, and `tmp`.
- `src/session/store/permissions.rs`: private directory/file mode expectations.
- `src/main_doctor.rs`: current read-only artifact checks and metadata sampling.

Contract probe:

```bash
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json get \
  'raw:<main><h1>Artifact Design</h1><p>Hello</p></main>' \
  --output "$tmpdir/caller-output.md" > "$tmpdir/result.json"
metadata=$(jq -r '.data.artifacts.metadata' "$tmpdir/result.json")
sed -n '1,220p' "$metadata"
find "$tmpdir" -maxdepth 4 -type f -print | sort
```

Finding:

- `metadata.json` is internal under `AGET_HOME/runs/<run-id>/metadata.json`.
- `artifacts.content` points to the caller-provided `--output` path when set.
- Therefore `delete`/`prune` must remove only the selected internal run
  directory and preserve content paths outside that directory.

Design validation:

```bash
cargo fmt --check
git diff --check
rg -n 'artifacts list|artifacts inspect|artifacts delete|artifacts prune|--older-than|--keep-last|--max-bytes|--dry-run|--yes|external' \
  workpads/post-migration/artifact-lifecycle-design.md
```

## ART-002 Artifact Lifecycle Implementation: 2026-05-24

Implemented surfaces:

- `src/cli/artifacts.rs`
- `src/main_artifacts.rs`
- `tests/cli/artifacts.rs`

Focused validation:

```bash
cargo fmt --check
cargo test --lib cli::tests::artifacts
cargo test --test cli artifacts
cargo test --test get_cli get_json_success_writes_run_artifacts_with_empty_state
```

Full validation:

```bash
cargo test
cargo build --release
tmpdir="$(mktemp -d)"
env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json get 'raw:<main><h1>No Command Path</h1></main>'
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" target/release/aget --envelope json doctor --quick
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" target/release/aget --envelope json get 'raw:<main><h1>Artifact Smoke</h1></main>' >/dev/null
AGET_HOME="$tmpdir/aget-home" target/release/aget --envelope json artifacts list
rm -rf "$tmpdir"
```

Release artifact refresh after ART-002:

```text
ea802299df748a61ccd20306482af4cf7018847decb790e0353fdc95560128d5  dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz
9c7b1d09b8496a4bdaffca16789faacd203446675e86a8607156c1ae85de4b8f  dist/aget-v0.1.0-aarch64-apple-darwin/aget
```

## BACKLOG-001 Batch/Map/Crawl Design: 2026-05-25

Design artifact:

- `workpads/post-migration/batch-map-crawl-design.md`

Current surfaces reviewed:

- `project.md`: original product goals for batch, map, crawl, and local/auth
  safety.
- `src/cli/get.rs`: current `get` flags to reuse across multi-URL commands.
- `src/extraction/owned/page/html.rs`: current extracted-page content and link
  processing boundary for future `map`.
- `workpads/post-migration/artifact-lifecycle-design.md`: internal artifact
  ownership and lifecycle constraints.
- `README.md`: current command reference and planned-work routing.

Validation:

```bash
cargo fmt --check
git diff --check
rg -n 'aget batch|aget map|aget crawl|--limit|--concurrency|same-origin|same-path|partial|manifest|MCP|server' \
  workpads/post-migration/batch-map-crawl-design.md README.md workpads/post-migration/tasks.md
```

## BATCH-001 Batch Implementation: 2026-05-25

Implemented surfaces:

- `src/cli/batch.rs`
- `src/main_batch.rs`
- `tests/cli/batch.rs`
- `src/extraction/artifacts/mod.rs`
- `src/session/playwright/state_file.rs`

Focused validation:

```bash
cargo fmt --check
cargo test --lib cli::tests::batch
cargo test --test cli batch
```

Full validation:

```bash
cargo test
cargo build --release
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json batch \
  'raw:<main><h1>Batch Smoke</h1></main>' \
  --output-dir "$tmpdir/batch"
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json get \
  'raw:<main><h1>No Command Path</h1></main>'
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json doctor --quick
rm -rf "$tmpdir"
```

Release artifact refresh after BATCH-001:

```text
b8d245b882a6f3ce3643edc777cf39e3e72cf1aea091300839d22401dec917b8  dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz
59905d22287e34e7ec162acb8066dfc13bf452f15c4393d1c5f5ac9d7699e589  dist/aget-v0.1.0-aarch64-apple-darwin/aget
```

## MAP-001 Map Implementation: 2026-05-25

Implemented surfaces:

- `src/cli/map.rs`
- `src/main_map.rs`
- `src/cli/tests/map.rs`
- `tests/cli/map.rs`
- `README.md`
- `CHANGELOG.md`

Focused validation:

```bash
cargo fmt --check
cargo test --lib cli::tests::map
cargo test --test cli map
```

Full validation:

```bash
cargo test
cargo build --release
tmpdir="$(mktemp -d)"
page="$tmpdir/page.html"
printf '%s\n' '<main><a href="/docs/a">A</a><a href="https://other.example/docs">External</a></main>' > "$page"
AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json map "file://$page" --any-path
rm -rf "$tmpdir"
```

Release artifact refresh after MAP-001:

```text
4c2e6721318c845cb379c95b119e76ee8554fa3547f182981c5d42e4078a373a  dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz
0e699ca349284625bb56be8bf3f4fa1a75cc481740fdd820e4563e4772a718b0  dist/aget-v0.1.0-aarch64-apple-darwin/aget
```

## CRAWL-001 Bounded Crawl Implementation: 2026-05-25

Implemented surfaces:

- `src/cli/crawl.rs`
- `src/main_crawl.rs`
- `src/cli/tests/crawl.rs`
- `tests/cli/crawl.rs`
- `README.md`
- `CHANGELOG.md`

Focused validation:

```bash
cargo fmt --check
cargo test --lib cli::tests::crawl
cargo test --test cli crawl
```

Full validation:

```bash
cargo test
cargo build --release
tmpdir="$(mktemp -d)"
page="$tmpdir/index.html"
printf '%s\n' '<main><a href="sub.html">Sub</a></main>' > "$page"
printf '%s\n' '<main><h1>Sub</h1></main>' > "$tmpdir/sub.html"
AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json crawl "file://$page" \
  --limit 2 \
  --any-path \
  --content-format html
rm -rf "$tmpdir"
```

Release artifact refresh after CRAWL-001:

```text
e95e4ddc16a3bf59128ec02ad92ae83ac73d8e25d74d78338391cf93bb2e832a  dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz
e6384d95635891b2c72c33b7d454e66d4f41d7859ccb292abfe3bc6f64da0e90  dist/aget-v0.1.0-aarch64-apple-darwin/aget
```

## REL-003 Release/Install Prep: 2026-05-26

Release notes draft:

- `workpads/post-migration/release-notes-v0.1.0.md`

Prepared GitHub Release URL:

- `https://github.com/nicolaslara/aget/releases/tag/v0.1.0`

Packaging update:

- The release archive now includes `skills/aget/SKILL.md` so the global Codex
  skill can be installed from the tarball without a separate source checkout.
- `Cargo.toml` now records `repository = "https://github.com/nicolaslara/aget"`
  for package/source-install metadata.
- README and skill docs include checkout and release-tarball skill install
  commands for `$CODEX_HOME/skills/aget`, with a Codex restart note.

Current archive SHA-256:

```text
b02a6f348a5adfbfd280b7b915cc485095dd4f67990c04d700871e8b37763182  aget-v0.1.0-aarch64-apple-darwin.tar.gz
```

Packaged binary SHA-256:

```text
e6384d95635891b2c72c33b7d454e66d4f41d7859ccb292abfe3bc6f64da0e90  dist/aget-v0.1.0-aarch64-apple-darwin/aget
```

Artifact generation command:

```bash
cargo build --release
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
target=$(rustc -vV | sed -n 's/^host: //p')
name="aget-v${version}-${target}"
archive="${name}.tar.gz"
rm -rf dist
mkdir -p "dist/${name}/skills/aget"
install -m 755 target/release/aget "dist/${name}/aget"
install -m 644 README.md "dist/${name}/README.md"
install -m 644 LICENSE "dist/${name}/LICENSE"
install -m 644 skills/aget/SKILL.md "dist/${name}/skills/aget/SKILL.md"
find "dist/${name}" -exec touch -t 202605240000 {} +
COPYFILE_DISABLE=1 tar --format ustar --uid 0 --gid 0 --uname root --gname wheel -C dist -cf "dist/${name}.tar" "${name}"
gzip -n "dist/${name}.tar"
(cd dist && shasum -a 256 "$archive" > "$archive.sha256")
(cd dist && shasum -a 256 "$archive" > SHA256SUMS)
```

Local archive validation:

```bash
tar -tzf dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz | sort
tmpdir="$(mktemp -d)"
tar -xzf dist/aget-v0.1.0-aarch64-apple-darwin.tar.gz -C "$tmpdir"
"$tmpdir/aget-v0.1.0-aarch64-apple-darwin/aget" --version
AGET_HOME="$tmpdir/aget-home" \
  "$tmpdir/aget-v0.1.0-aarch64-apple-darwin/aget" --envelope json doctor --quick
skill_tmp="$tmpdir/codex-home"
mkdir -p "$skill_tmp/skills"
cp -R "$tmpdir/aget-v0.1.0-aarch64-apple-darwin/skills/aget" "$skill_tmp/skills/aget"
test -f "$skill_tmp/skills/aget/SKILL.md"
rm -rf "$tmpdir"
(cd dist && shasum -a 256 -c aget-v0.1.0-aarch64-apple-darwin.tar.gz.sha256)
(cd dist && shasum -a 256 -c SHA256SUMS)
```

Validation result:

- Archive contents include `aget`, `README.md`, `LICENSE`, and
  `skills/aget/SKILL.md`.
- Tarball binary reported `aget 0.1.0`.
- Tarball `doctor --quick` returned `ok: true` with one non-failing OpenCode
  PATH warning.
- Temporary tarball skill install copied `SKILL.md` successfully.
- Per-archive checksum and `SHA256SUMS` both verified.

Release gate before publication:

```bash
cargo fmt --check
git diff --check
cargo test
rg -n 'Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND|AgentBrowser|agent_browser' \
  README.md skills/aget/SKILL.md .opencode/tools/aget.ts src tests scripts -S
tmpdir="$(mktemp -d)"
env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json get 'raw:<main><h1>No Command Path</h1></main>'
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
cargo install --path . --locked --root "$tmpdir/install"
"$tmpdir/install/bin/aget" --version
AGET_HOME="$tmpdir/aget-home" "$tmpdir/install/bin/aget" --envelope json doctor --quick
rm -rf "$tmpdir"
```

Validation result:

- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `cargo test`: passed; 159 lib tests passed with 2 ignored, integration suites
  passed, and doc tests passed.
- Stale dependency-surface grep returned no active matches.
- No-command-path smoke returned `ok: true` with `# No Command Path`.
- Temp-root `cargo install --path . --locked` installed `aget 0.1.0`; installed
  binary `doctor --quick` returned `ok: true`.

Source install validation after GitHub Release publication:

```bash
tmpdir="$(mktemp -d)"
cargo install --git https://github.com/nicolaslara/aget --tag v0.1.0 --locked --root "$tmpdir/install" aget
"$tmpdir/install/bin/aget" --version
AGET_HOME="$tmpdir/aget-home" "$tmpdir/install/bin/aget" --envelope json doctor --quick
rm -rf "$tmpdir"
```

Validation result:

- Initial smoke without the explicit `aget` package argument failed because the
  repository also contains the `aget-mock-tools` binary package.
- The corrected command with the explicit `aget` package installed `aget 0.1.0`
  from `https://github.com/nicolaslara/aget?tag=v0.1.0#b88ff890`.
- Installed source binary `doctor --quick` returned `ok: true`.

Published release verification:

```bash
gh release view v0.1.0 --json url,tagName,targetCommitish,assets,isDraft,isPrerelease,name
git ls-remote --tags origin v0.1.0
tmpdir="$(mktemp -d)"
gh release download v0.1.0 -D "$tmpdir/download"
(cd "$tmpdir/download" && shasum -a 256 -c aget-v0.1.0-aarch64-apple-darwin.tar.gz.sha256)
(cd "$tmpdir/download" && shasum -a 256 -c SHA256SUMS)
tar -xzf "$tmpdir/download/aget-v0.1.0-aarch64-apple-darwin.tar.gz" -C "$tmpdir"
"$tmpdir/aget-v0.1.0-aarch64-apple-darwin/aget" --version
AGET_HOME="$tmpdir/aget-home" \
  "$tmpdir/aget-v0.1.0-aarch64-apple-darwin/aget" --envelope json doctor --quick
rg -n "cargo install --git" "$tmpdir/aget-v0.1.0-aarch64-apple-darwin/README.md"
mkdir -p "$tmpdir/codex-home/skills"
cp -R "$tmpdir/aget-v0.1.0-aarch64-apple-darwin/skills/aget" "$tmpdir/codex-home/skills/aget"
test -f "$tmpdir/codex-home/skills/aget/SKILL.md"
rm -rf "$tmpdir"
tmpdir="$(mktemp -d)"
CARGO_HOME="$tmpdir/cargo-home" \
  cargo install --git https://github.com/nicolaslara/aget --tag v0.1.0 --locked --root "$tmpdir/install" aget
"$tmpdir/install/bin/aget" --version
AGET_HOME="$tmpdir/aget-home" "$tmpdir/install/bin/aget" --envelope json doctor --quick
rm -rf "$tmpdir"
```

Validation result:

- GitHub Release URL:
  `https://github.com/nicolaslara/aget/releases/tag/v0.1.0`.
- Release is published, not draft or prerelease.
- Release target and remote `v0.1.0` tag both resolve to
  `b88ff890f0eb114ffdef380b85cd6637cdc5ccaa`.
- Uploaded assets are:
  `aget-v0.1.0-aarch64-apple-darwin.tar.gz`,
  `aget-v0.1.0-aarch64-apple-darwin.tar.gz.sha256`, and `SHA256SUMS`.
- GitHub asset digest for the tarball is
  `sha256:b02a6f348a5adfbfd280b7b915cc485095dd4f67990c04d700871e8b37763182`.
- Downloaded checksums verified.
- Downloaded tarball binary reported `aget 0.1.0` and `doctor --quick`
  returned `ok: true`.
- Downloaded tarball README contains the corrected source install command:
  `cargo install --git https://github.com/nicolaslara/aget --tag v0.1.0 --locked aget`.
- Downloaded tarball skill copied successfully into a temporary
  `$CODEX_HOME/skills/aget`.
- Fresh `CARGO_HOME` source install from `v0.1.0` installed `aget 0.1.0` from
  commit `b88ff890` and `doctor --quick` returned `ok: true`.

## REL-008 Global Skill Install Helper: 2026-05-26

Implemented surface:

- `scripts/install-codex-skill.sh`

Validation commands:

```bash
bash -n scripts/install-codex-skill.sh
tmpdir="$(mktemp -d)"
scripts/install-codex-skill.sh --codex-home "$tmpdir/codex-copy"
test -f "$tmpdir/codex-copy/skills/aget/SKILL.md"
scripts/install-codex-skill.sh --codex-home "$tmpdir/codex-link" --symlink
test "$(readlink "$tmpdir/codex-link/skills/aget")" = "$(pwd)/skills/aget"
rm -rf "$tmpdir"
scripts/install-codex-skill.sh --codex-home "$HOME/.codex" --symlink --force
readlink "$HOME/.codex/skills/aget"
test -f "$HOME/.codex/skills/aget/SKILL.md"
```

Validation result:

- The helper shell syntax is valid.
- Temporary copy-mode install produced `skills/aget/SKILL.md`.
- Temporary symlink-mode install pointed at the checkout's `skills/aget`.
- Local global install now points
  `/Users/nicolas/.codex/skills/aget -> /Users/nicolas/devel/aget/skills/aget`.
- Codex must be restarted before a running session sees a newly installed or
  replaced global skill.

## AGENT-001 OpenCode/Skill Integration: 2026-05-26

Implemented surfaces:

- `.opencode/tools/aget.ts`
- `.opencode/lib/aget_args.ts`
- `.opencode/tests/aget_args.test.ts`
- `skills/aget/SKILL.md`
- `README.md`

New OpenCode wrappers:

- `aget_batch`
- `aget_map`
- `aget_crawl`
- `aget_artifacts_list`
- `aget_artifacts_inspect`
- `aget_doctor`

Deterministic wrapper validation:

```bash
cd .opencode && bun test tests/aget_args.test.ts
cd .opencode && bun -e 'import("./tools/aget.ts").then((m) => console.log(Object.keys(m).sort().join("\n")))'
git diff --check
cargo fmt --check
cargo test
rg -n 'Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND|AgentBrowser|agent_browser' \
  README.md skills/aget/SKILL.md .opencode/tools/aget.ts src tests scripts -S
```

Validation result:

- Batch builder covers explicit URL lists, repeated sessions, content format,
  concurrency, output directory, and fail-fast flag.
- Map builder covers artifact-first discovery, same-path widening, include
  filters, content-type filters, and max-link limits.
- Crawl builder covers required limit, max depth, concurrency, allow-domain,
  sessions, content format, and output directory.
- Artifact and doctor builders cover `artifacts list`, `artifacts inspect`, and
  selected `doctor --quick --check ...` diagnostics.
- `.opencode/tools/aget.ts` exports only actual OpenCode tool definitions, not
  test helpers.
- `git diff --check`, `cargo fmt --check`, and `cargo test` passed. The stale
  dependency-surface grep returned no active matches.

Direct wrapper smoke:

```bash
tmpdir="$(mktemp -d)"
# temporary Bun script imports .opencode/tools/aget.ts, points
# AGET_OPENCODE_BIN at target/release/aget, and executes doctor, batch, map,
# crawl, and artifacts_list against raw/file inputs under a temporary AGET_HOME.
SMOKE_TMP="$tmpdir" bun "$tmpdir/wrapper_smoke.ts"
rm -rf "$tmpdir"
```

Validation result:

- The OpenCode tool module exported only tool definitions:
  `artifacts_inspect`, `artifacts_list`, `batch`, `crawl`, `doctor`, `fetch`,
  `map`, `session_import_chrome`, `session_inspect`, and `session_list`.
- Direct wrapper execution returned valid envelopes with commands `batch`,
  `map`, `crawl`, and `artifacts.list`; `doctor --quick --check binary`
  returned `ok: true`.

Review note:

- A focused review subagent was spawned for AGENT-001 but did not return before
  shutdown. Local review found one integration risk: helper exports and tests
  under `.opencode/tools` could be mistaken for the tool surface. Resolution:
  move argument builders to `.opencode/lib/aget_args.ts`, move snapshots to
  `.opencode/tests/aget_args.test.ts`, and keep `.opencode/tools/aget.ts`
  exporting only actual tools.

## CACHE-001 Cache/Freshness/Usage Metadata: 2026-05-26

Implemented surfaces:

- `aget get|batch|map|crawl --fresh`
- `aget get|batch|map|crawl --cache-policy <auto|refresh|off>`
- `aget get|batch|map|crawl --cache-ttl <seconds>`
- `AGET_HOME/cache/<key>/metadata.json`
- `AGET_HOME/cache/<key>/content`
- `data.cache` and `data.usage` in `get` envelopes and artifact metadata
- per-item/source cache and usage metadata in `batch`, `map`, and `crawl`
  manifests where those commands fetch through `get`

Design decisions:

- Cache eligibility is limited to public unauthenticated HTTP(S) fetches.
  Session-backed, current-tab, `raw:`, and `file://` inputs do not read or
  write reusable cache entries.
- Cache keys include URL, output format, selectors/waits, and normalized backend
  options. Keys intentionally ignore `--output` and `--max-chars`.
- Cache entries store full untruncated extracted content. Each run still writes
  fresh run artifacts and applies its own `--max-chars`.
- Cache metadata statuses are `miss`, `hit`, `stale`, `refresh`, `disabled`,
  and `ineligible`.
- Usage metadata is approximate and intended for agent budgeting:
  `fetched_bytes`, `content_bytes`, `estimated_tokens`, and
  `estimated_tokens_saved`.

Focused validation:

```bash
cargo test --lib cache
cargo test --lib cli::tests
cargo test --test get_cli cache
cd .opencode && bun test tests/aget_args.test.ts
cargo run --quiet -- get --help | rg -- '--fresh|--cache-policy|--cache-ttl'
cargo run --quiet -- batch --help | rg -- '--fresh|--cache-policy|--cache-ttl'
cargo run --quiet -- map --help | rg -- '--fresh|--cache-policy|--cache-ttl'
cargo run --quiet -- crawl --help | rg -- '--fresh|--cache-policy|--cache-ttl'
```

Final validation gate:

```bash
cargo fmt --check
git diff --check
cargo test
rg -n 'Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND|AgentBrowser|agent_browser' \
  README.md skills/aget/SKILL.md .opencode/tools/aget.ts src tests scripts -S
cargo build
tmpdir="$(mktemp -d)"
env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  AGET_HOME="$tmpdir/aget-home" \
  target/debug/aget --envelope json get 'raw:<main><h1>No Command Path</h1></main>'
rm -rf "$tmpdir"
```

Validation result:

- Focused cache tests covered miss-to-hit behavior, stale refresh,
  `--fresh`, cache/usage metadata shape, exact cache-hit content, and
  session-backed ineligibility.
- CLI parser tests covered `--fresh`, `--cache-policy off`, `--cache-ttl`, and
  conflicting `--fresh` plus `--cache-policy`.
- OpenCode argument snapshots include cache controls for `batch`, `map`, and
  `crawl`.
- Help output for `get`, `batch`, `map`, and `crawl` includes `--fresh`,
  `--cache-policy`, and `--cache-ttl`.
- `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.
- Stale dependency-surface grep returned no active matches.
- No-command-path smoke returned `ok: true`; raw input reported
  `cache.status = ineligible` and included usage metadata.

Review note:

- Local review focused on cache privacy, content consistency, and cache
  robustness. Accepted fixes: write cache content without appending artifact
  newlines, normalize backend-option order in cache keys, expose cache/usage
  metadata through multi-URL manifests, and treat malformed cache entries as
  misses instead of blocking fresh fetches.

## SEARCH-001 Artifact Page Search: 2026-05-26

Implemented surfaces:

- `aget search-page --artifact <run-id> --query <text>`
- `--max-results <n>`
- `--context-chars <n>`
- `--allow-private-content`
- `--output <markdown|json>`
- OpenCode wrapper `aget_search_page`

Design decisions:

- Search is artifact-first and reads an existing successful `get` run. It does
  not fetch URLs, run browser automation, or call an LLM.
- Ranking is deterministic and local. It scores markdown-ish sections by
  heading phrase matches, body phrase matches, keyword counts, and structured
  lines such as links, lists, and table rows.
- Results include section IDs, heading context, snippets, scores, and character
  offsets. Full source artifact content is not embedded in the search envelope.
- Sensitive artifacts require `--allow-private-content` before snippets are
  emitted.

Focused validation:

```bash
cargo test --lib cli::tests::search_page
cargo test --test cli search_page
cd .opencode && bun test tests/aget_args.test.ts
cd .opencode && bun -e 'import("./tools/aget.ts").then((m) => console.log(Object.keys(m).sort().join("\n")))'
cargo run --quiet -- search-page --help | rg -- '--artifact|--query|--max-results|--allow-private-content'
```

Final validation gate:

```bash
cargo fmt --check
git diff --check
cargo test
rg -n 'Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND|AgentBrowser|agent_browser' \
  README.md skills/aget/SKILL.md .opencode/tools/aget.ts src tests scripts -S
```

Validation result:

- Parser and help tests cover the `search-page` CLI surface.
- CLI integration tests cover heading/keyword ranking, empty no-match output,
  and sensitive artifact consent before snippet emission.
- OpenCode argument snapshots cover `search-page`, and tool export smoke shows
  `search_page` as an actual wrapper.
- `cargo fmt --check`, `git diff --check`, and full `cargo test` passed.
- Stale dependency-surface grep returned no active matches.

Review note:

- Local review focused on privacy and artifact lifecycle compatibility. Accepted
  fixes: write `metadata.json` for search runs so artifact lifecycle commands
  can inspect/prune them, and include source URL/finality/sensitivity metadata
  without embedding full source content in the search envelope.

## EXTRACT-001 Structured Artifact Extraction: 2026-05-26

Implemented surfaces:

- `aget extract --artifact <run-id>`
- `aget extract --manifest <path>`
- `--field <headings|links|tables|definitions|metadata>`
- `--schema <path>` for selector, JSON path, and metadata path fields
- `--allow-private-content`
- `--output <json|markdown>`
- OpenCode wrapper `aget_extract`

Design decisions:

- Extraction is artifact-first. It reads existing successful page artifacts or
  successful batch/crawl manifest items; it does not refetch pages or call an
  LLM.
- Built-in primitives are deterministic: headings, links, tables, definition
  lists, and metadata. Schema fields add HTML selectors and JSON paths.
- Sensitive source artifacts require `--allow-private-content` before
  structured values are emitted.
- `extract --artifact` reads only internal run content and refuses
  caller-owned external `--output` paths.
- `extract --manifest` requires per-item metadata under `AGET_HOME/runs`,
  requires manifest content paths to match item metadata, and keeps content
  reads inside the manifest directory.

Focused validation:

```bash
cargo test --lib cli::tests::extract
cargo test --test cli extract
cd .opencode && bun test tests/aget_args.test.ts
cargo run --quiet -- extract --help | rg -- '--artifact|--manifest|--schema|--field|--allow-private-content'
```

Final validation gate:

```bash
cargo fmt --check
git diff --check
cargo test
rg -n 'Crawl4AI|crawl4ai|agent-browser|AGET_CRAWL4AI_COMMAND|AGET_AGENT_BROWSER_COMMAND|AgentBrowser|agent_browser' \
  README.md skills/aget/SKILL.md .opencode/tools/aget.ts src tests scripts -S
cargo run --quiet -- --help | rg 'extract|search-page'
cargo run --quiet -- extract --help | rg -- '--artifact|--manifest|--schema|--field|--allow-private-content'
```

Validation result:

- Parser tests cover artifact and manifest input shapes.
- CLI integration tests cover table extraction, selector extraction, batch
  manifest extraction, crawl manifest extraction, schema JSON and metadata
  path extraction, malformed schema/input errors, sensitive artifact consent,
  sensitive manifest-item consent, external artifact refusal, and mismatched
  manifest content-path refusal.
- OpenCode argument snapshots cover `extract`.
- `cargo fmt --check`, `git diff --check`, full `cargo test`, stale
  dependency-surface grep, and help smoke passed.

Review note:

- Focused review found that the first implementation allowed manifest fallback
  metadata and trusted external artifact content paths. Accepted fixes removed
  fallback metadata, required `AGET_HOME/runs` item metadata, required content
  path matching, bounded manifest content reads to the manifest directory, and
  made `extract --artifact` refuse caller-owned external `--output` paths.

## REL-004 CI And Release Automation: 2026-05-26

Implemented surfaces:

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `scripts/package-release.sh`

Primary source reviewed:

- GitHub-hosted runner labels:
  `https://docs.github.com/en/actions/reference/github-hosted-runners-reference`

Design decisions:

- CI runs on `ubuntu-24.04` for fmt, tests, release build, stale
  dependency-surface grep, no-command-path smoke, and doctor smoke.
- Multi-target package smoke uses hosted runner labels from the GitHub docs:
  `ubuntu-24.04`, `ubuntu-24.04-arm`, `macos-15`, and `macos-15-intel`.
- Release workflow runs on `v*` tag pushes or manual dispatch for an existing
  tag. It packages macOS ARM, macOS Intel, Linux x86_64, and Linux ARM64
  tarballs, verifies each packaged binary, regenerates aggregate `SHA256SUMS`,
  and creates or updates GitHub Release assets with `gh`.
- Windows artifacts remain deferred until REL-007 provides a real Windows
  smoke path.
- `scripts/package-release.sh` is the shared packaging contract for local and
  CI use. It packages `aget`, `README.md`, `LICENSE`, `skills/aget/SKILL.md`,
  and `scripts/install-codex-skill.sh`.

Local validation:

```bash
bash -n scripts/package-release.sh
ruby -e 'require "yaml"; ARGV.each { |path| YAML.load_file(path); puts path }' \
  .github/workflows/ci.yml .github/workflows/release.yml
tmpdir="$(mktemp -d)"
target="$(rustc -vV | sed -n 's/^host: //p')"
scripts/package-release.sh --target "$target" --out-dir "$tmpdir/dist"
name="aget-v$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)-$target"
tar -xzf "$tmpdir/dist/$name.tar.gz" -C "$tmpdir"
"$tmpdir/$name/aget" --version
AGET_HOME="$tmpdir/aget-home" "$tmpdir/$name/aget" --envelope json doctor --quick
test -f "$tmpdir/$name/README.md"
test -f "$tmpdir/$name/LICENSE"
test -f "$tmpdir/$name/skills/aget/SKILL.md"
test -x "$tmpdir/$name/scripts/install-codex-skill.sh"
(cd "$tmpdir/dist" && shasum -a 256 -c "$name.tar.gz.sha256" && shasum -a 256 -c SHA256SUMS)
rm -rf "$tmpdir"
```

Validation result:

- Shell syntax check passed.
- Workflow YAML parsed successfully.
- Host-target package smoke produced
  `aget-v0.1.0-aarch64-apple-darwin.tar.gz` in a temporary directory.
- Packaged binary reported `aget 0.1.0`.
- Packaged `doctor --quick` returned `ok: true` with one non-failing OpenCode
  PATH warning.
- Packaged README, LICENSE, skill, and install helper were present.
- Per-archive checksum and aggregate `SHA256SUMS` verified.
- `cargo fmt --check`, `git diff --check`, and full `cargo test` passed after
  the workflow/script/docs updates.
- Stale dependency-surface grep returned no active matches.
- No-command-path smoke returned `ok: true` with `# No Command Path`.
- Remote GitHub Actions status is pending until these workflow files are pushed
  and run on GitHub.

## REL-005 Homebrew Tap/Formula Path: 2026-05-26

Draft formula:

- `workpads/post-migration/homebrew/aget.rb`

Primary sources reviewed:

- Homebrew Formula Cookbook:
  `https://docs.brew.sh/Formula-Cookbook`
- Homebrew tap maintenance guide:
  `https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap`

Decision:

- Use a dedicated tap, planned as `nicolaslara/homebrew-aget`.
- Do not submit to Homebrew core yet. The project is pre-1.0, still evolving
  CLI surfaces quickly, and the current published binary release only has a
  macOS ARM artifact.
- Defer publishing the tap until a release has macOS ARM and macOS Intel
  artifacts generated by the REL-004 release workflow.
- Keep `cargo install --git ... aget` and release tarball install as the
  supported source/binary paths until the tap is published.

Current v0.1.0 formula draft:

- Uses the published macOS ARM release artifact:
  `https://github.com/nicolaslara/aget/releases/download/v0.1.0/aget-v0.1.0-aarch64-apple-darwin.tar.gz`
- Uses SHA-256:
  `b02a6f348a5adfbfd280b7b915cc485095dd4f67990c04d700871e8b37763182`
- Installs `aget`, README, LICENSE, `skills/aget`, and
  `scripts/install-codex-skill.sh`.
- Includes a Homebrew `test do` smoke for `aget --version` and
  `aget --envelope json doctor --quick`.
- Declares `depends_on :macos` and `depends_on arch: :arm64` because the
  current published artifact is macOS ARM only.

Future tap commands:

```bash
brew tap nicolaslara/aget https://github.com/nicolaslara/homebrew-aget
brew install nicolaslara/aget/aget
aget --version
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" aget --envelope json doctor --quick
rm -rf "$tmpdir"
```

Local validation:

```bash
ruby -c workpads/post-migration/homebrew/aget.rb
```

Validation result:

- Formula syntax is valid Ruby.
- `brew audit --formula --strict --new workpads/post-migration/homebrew/aget.rb`
  was attempted, but this Homebrew version rejects path-based audit with:
  `Calling brew audit [path ...] is disabled! Use brew audit [name ...] instead.`
  Developer mode was turned back off afterward with `brew developer off`.
