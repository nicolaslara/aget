# Knowledge Archive D169: Engine Refactor Plan

### D169: Plan `AgetExtractor` and `AgetBrowser` engine boundaries

After the original oversized extraction/CDP module split, the local replacements still use migration-era names such as `OwnedExtractorBackend` and `OwnedBrowserAutomationBackend`. Those names describe how the code was migrated away from command dependencies, not the product boundary that should remain after the migration settles.

`workpads/research/aget-engine-refactor-plan.md` now records the planned next refactor: introduce explicit local engines named `AgetExtractor` and `AgetBrowser`, keep `Aget` as the public facade for named sessions, persistence, artifacts, JSON envelopes, authorization workflow, and CLI policy, and keep command-backed Crawl4AI/`agent-browser` adapters as separate compatibility surfaces.

Task I19j tracks this work. The planned refactor should stay mechanical and separately committable: introduce wrappers first, move adapters to call them, then rename live backend types and tests after behavior is stable. Public CLI/API behavior and existing backend trait contracts should be preserved unless a follow-up task explicitly approves behavior change.

Validation:

- `git diff --check`

Confidence: High for the planning boundary. This is documentation/task planning only; implementation details still need code-level validation as I19j slices land.
