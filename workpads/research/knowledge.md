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
- I19/I19d/I19e/I19h are complete as replacement-grade migration work. Final closure evidence is in `experiments/2026-05-24-final-migration-release-and-browser-smokes.md`.
- I19d decision: the local `AgetExtractor` replacement is good enough for current agent usefulness. Remaining Crawl4AI-quality/readability/rendered-page deltas are accepted as normal backlog unless they change concrete agent outcomes.
- I19e decision: the local `AgetBrowser` replacement is good enough for current browser/session usage. Verified support is intentionally Chrome/CDP-focused; broader browser/profile behavior is documented as unsupported/deferred.
- I19h decision: final review passes completed for test adequacy, architecture cohesion, security/privacy, and docs/user workflow clarity. Material findings were fixed or explicitly deferred before closure.
- Recently completed: I23 added `aget session login start --session <provider>` support so reusable OAuth/provider sessions such as `oauth`, `google`, or `github` can be injected into controlled login profiles without exposing credentials to the agent. See D325.
- Current support-file rule: do not compact `workpads/research/tasks.md`. Keep top-level support files as compact routers and move dense completed-history detail to referenced archive files; see D308, D347, D357, D364, and D376-D420.
- Latest mechanical cleanup split: I19gzzt moved HTML cleanup attribute helpers from one `attributes.rs` file into `src/extraction/html_clean/attributes/` route modules for pruning, image-source cleanup, comments, empty-element pruning, and only-text replacement. Behavior stayed behind the same `html_clean` re-exported helper names. Verification passed with `cargo fmt --check`, `cargo test html_clean`, and line-count checks.
- Release-binary private GitHub session smoke evidence is recorded in `experiments/2026-05-22-release-binary-github-private-repo-smoke.md`: public fetch worked, unauthenticated private repo fetch returned GitHub's generic private/404 page, user-driven `session login` saved a reusable `github` session, and that session fetched both `nicolaslara/zodl-desktop` and `nicolaslara/ai`.

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
| Latest individual decisions | `archive/knowledge/current-decision-index.md` and per-decision files through `archive/knowledge/d420-cmux-session-import-helper-split.md` |
| Release-binary private GitHub session smoke | `experiments/2026-05-22-release-binary-github-private-repo-smoke.md` |
| Final migration release/browser smokes | `experiments/2026-05-24-final-migration-release-and-browser-smokes.md` |

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
