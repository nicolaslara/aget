# Research Knowledge

This compact file records only the current routing context needed for active work. Detailed decision history lives under `workpads/research/archive/knowledge/`; start with `archive/knowledge/current-decision-index.md` when you need older evidence.

## Current Project Direction

`aget` is a local-first, auth-aware URL-to-agent-context tool. The active implementation direction is a Rust CLI/library with these boundaries:

- `src/aget/`: public facade and orchestration across extractor, browser automation/fallback, and session-store backends.
- `src/extraction/` plus `src/aget_extractor.rs`: local Crawl4AI-like extraction engine and backend wrapper.
- `src/browser_cdp/`, `src/aget_browser.rs`, `src/aget_browser/`, and browser-facing session modules: local agent-browser-like Chrome/CDP behavior and backend wrapper.
- `src/session/`: local session model, store, composition, imports, login lifecycle, and compatibility helpers.

Safety boundary remains unchanged: `aget` is a generic fetcher for content the user is authorized to access. Authenticated state and content stay local by default. Site-specific paywall/login reasoning belongs to the caller or an agent-facing skill, not the binary.

## Active Work State

- Active workpad: `workpads/research/tasks.md`.
- Main migration umbrella: I19 on branch `dep-migration-homegrown-backends`.
- Completed migration phases include source clones/inventory, backend abstraction audit, parity tests, default owned runtime switch, PoC command-default demotion, engine naming cleanup, module/test/support decomposition, and follow-up cleanup-module splitting.
- Still open: I19d, I19e, and I19h.
- I19d remains open for full Crawl4AI-quality readability/markdown and still-richer rendered-page readiness after D281 added source-backed generic page metadata propagation, D282 preserved it across owned browser fallback extraction, D285 aligned agent-facing option docs with the owned extractor surface, and D286 preserved code-block whitespace during markdown normalization.
- I19e remains open for broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and still-fuller startup/error classification after D284 added source-backed lifecycle/networkidle timeout messages.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.
- Current support-file rule: do not compact `workpads/research/tasks.md`; compact supporting files by moving dense detail to referenced archive files. The largest historical knowledge bundles now route through split archive indexes after D232, the historical session-wrapper PoC spec and implementation plan route through smaller support slices after D243/D263-D265, and the R12/D55-D63/D64-D76/D77-D91/D92-D108/D109-D122/engine-refactor/OAuth-design bundles route through smaller support slices after D268-D280. The current CDP page-script helpers route through behavior-owned modules after D276, Crawl4AI local-content URL compatibility routes through D277, raw/local base-URL option behavior routes through D278, main-content negative class/id exclusion routes through D279, page metadata propagation routes through D281, fallback metadata propagation routes through D282, networkidle reset/timeout parity coverage routes through D283, CDP navigation wait timeout messages route through D284, agent-facing option doc alignment routes through D285, and code-block whitespace preservation routes through D286.

## Decision Routing

| Need | Open |
| --- | --- |
| Recent decision ledger and archive map | `archive/knowledge/current-decision-index.md` |
| Foundation, PoC, and early architecture decisions | `archive/knowledge/d001-d048-foundation-and-poc.md` index to `d001-d020-foundation-research.md`, `r12-mvp-architecture-proposal.md` index to section files, `d021-d034-poc-session-api.md`, and `d035-d048-poc-hardening-backends.md` |
| Migration setup, parity, default switch, and first audit | `archive/knowledge/d049-d076-migration-setup-and-defaults.md` index to `d049-d054-migration-setup-and-parity.md`, `d055-d063-owned-extractor-foundation.md` index to section files, and `d064-d076-browser-default-switch.md` index to section files |
| Owned backend feature slices | `archive/knowledge/d077-d122-owned-backend-slices.md` index to `d077-d091-owned-extractor-options-cleanup.md` index to section files, `d092-d108-markdown-browser-slices.md` index to section files, and `d109-d122-selector-overlay-shadow.md` index to section files |
| Original module-decomposition sequence | `archive/knowledge/d123-d150-module-decomposition.md` index to `d123-d150-module-decomposition/d123-d132-core-production-splits.md`, `d123-d150-module-decomposition/d133-d141-orchestration-and-test-target-splits.md`, and `d123-d150-module-decomposition/d142-d150-integration-test-splits.md` |
| Auth/browser import and browser-choice design | `archive/knowledge/d151-d157-auth-and-browser-import.md` |
| Recent migration follow-ups and source-backed extraction improvements | `archive/knowledge/d158-d167-recent-migration-followups.md` |
| Latest individual decisions | `archive/knowledge/d168-workpad-knowledge-compaction.md` through `archive/knowledge/d286-code-block-whitespace.md` |

## Historical Support Docs

| Old entrypoint | Full archived content |
| --- | --- |
| `session-wrapper-poc-spec.md` | `archive/support/session-wrapper-poc-spec.md` index to section files |
| `session-wrapper-implementation-plan.md` | `archive/support/session-wrapper-implementation-plan.md` |
| `aget-engine-refactor-plan.md` | `aget-engine-refactor-plan.md` index to section files |
| `oauth-safe-browser-login-design.md` | `oauth-safe-browser-login-design.md` index to section files |

## Current Verification Expectations

- For behavior changes, inspect the relevant external source snapshot under `references/repos/crawl4ai` or `references/repos/agent-browser` before porting and record the source-backed decision in the appropriate workpad file.
- Run focused tests for touched behavior plus `cargo fmt --check` and, at stable points, `cargo test`.
- Before starting another task after a stable point, make an explicit commit decision. The user wants iterative stable commits on this branch.
- `CLAUDE_REVIEW.md` is an untracked review artifact in this worktree; do not rewrite or commit it unless explicitly asked.

## Open Questions

- Should current-tab grow optional profile/port discovery after the explicit-port consented command has enough real-browser smoke evidence?
- Can pure Rust browser automation provide reliable persistent profiles and CDP attach across platforms?
- Which remaining readability/markdown features are worth porting before final I19 review?
- How should `aget` represent and redact authenticated/private content before returning it to an agent?
