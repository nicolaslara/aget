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
