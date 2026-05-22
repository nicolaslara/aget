# Research Knowledge

This compact file records only the current routing context needed for active work.
Detailed decision history lives under `workpads/research/archive/knowledge/`; start with `archive/knowledge/current-decision-index.md` when you need older evidence.

## Current Project Direction

`aget` is a local-first, auth-aware URL-to-agent-context tool. The active implementation direction is a Rust CLI/library with these boundaries:

- `src/aget/`: public facade and orchestration across extractor, browser automation/fallback, and session-store backends.
- `src/extraction/` plus `src/aget_extractor.rs`: local Crawl4AI-like extraction engine and backend wrapper.
- `src/browser_cdp/`, `src/aget_browser.rs`, `src/aget_browser/`, and browser-facing session modules: local agent-browser-like Chrome/CDP behavior and backend wrapper.
- `src/session/`: local session model, store, composition, imports, login lifecycle, and compatibility helpers.

Safety boundary remains unchanged: `aget` is a generic fetcher for content the user is authorized to access.
Authenticated state and content stay local by default; site-specific paywall/login reasoning belongs to the caller or an agent-facing skill, not the binary.

## Active Work State

- Active workpad: `workpads/research/tasks.md`.
- Main migration umbrella: I19 on branch `dep-migration-homegrown-backends`.
- Completed migration phase detail is archived in `archive/knowledge/d308-workpad-support-routing.md`.
- Still open: I19d, I19e, and I19h.
- I19d remains open for full Crawl4AI-quality readability/markdown and still-richer rendered-page readiness. Recent source-backed markdown option work is D293-D307 plus D309-D311; current-state detail routes through D308.
- I19e remains open for broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and startup/error classification. Current wait/startup detail routes through D284 and D287.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.
- Current support-file rule: do not compact `workpads/research/tasks.md`. Keep top-level support files as compact routers and move dense completed-history detail to referenced archive files; see D308.

## Decision Routing

| Need | Open |
| --- | --- |
| Recent decision ledger and archive map | `archive/knowledge/current-decision-index.md` |
| Foundation, PoC, and early architecture decisions | `archive/knowledge/d001-d048-foundation-and-poc.md` |
| Migration setup, parity, default switch, and first audit | `archive/knowledge/d049-d076-migration-setup-and-defaults.md` |
| Owned backend feature slices | `archive/knowledge/d077-d122-owned-backend-slices.md` |
| Original module-decomposition sequence | `archive/knowledge/d123-d150-module-decomposition.md` |
| Auth/browser import and browser-choice design | `archive/knowledge/d151-d157-auth-and-browser-import.md` |
| Recent migration follow-ups and source-backed extraction improvements | `archive/knowledge/d158-d167-recent-migration-followups.md` |
| Latest individual decisions | `archive/knowledge/d168-workpad-knowledge-compaction.md` through `archive/knowledge/d314-owned-option-apply-split.md` |

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
