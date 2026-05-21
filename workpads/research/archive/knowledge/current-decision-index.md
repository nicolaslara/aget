# Current Decision Index

This archive file keeps the denser decision routing that used to live in top-level `knowledge.md`. Open the per-decision archive files for exact evidence, validation commands, and source paths.

## Recent Decision Routing

| Decision | Archive | Use |
| --- | --- | --- |
| D158-D167 | `d158-d167-recent-migration-followups.md` | Source-backed extraction improvements and follow-up decomposition slices. |
| D168 | `d168-workpad-knowledge-compaction.md` | First workpad knowledge compaction pass. |
| D169 | `d169-engine-refactor-plan.md` | `AgetExtractor`/`AgetBrowser` boundary plan. |
| D170-D176 | `d170-aget-browser-wrapper.md` through `d176-direct-extractor-option-test.md` | Engine wrapper introduction, naming cleanup, and direct engine coverage. |
| D177-D186 | `d177-markdown-module-split.md` through `d186-main-session-module-split.md` | Mechanical production-code module splits. |
| D187-D195 | `d187-mock-backend-module-split.md` through `d195-browser-cdp-test-split.md` | Mechanical test/support module splits. |
| D196 | `d196-main-content-density-scoring.md` | Crawl4AI `PruningContentFilter`-inspired main-content scoring. |
| D197 | `d197-support-file-compaction.md` | Top-level support-file compaction without task compaction. |
| D198 | `d198-class-id-noise-scoring.md` | Crawl4AI-inspired class/id noise penalty for main-content candidates. |
| D199 | `d199-direct-page-cdp-session.md` | agent-browser-inspired direct page CDP session support. |
| D200 | `d200-cdp-target-discovery-auto-attach.md` | agent-browser-inspired target discovery and auto-attach parity. |
| D201 | `d201-cdp-same-document-navigation.md` | agent-browser-inspired same-document CDP navigation handling. |

## Full Archive Routing

| Range | Archive | Contents |
| --- | --- | --- |
| D1-D48 | `d001-d048-foundation-and-poc.md` | Product premise, prior art, benchmarks, R12 MVP architecture, early implementation and backend-pluggability decisions. |
| D49-D76 | `d049-d076-migration-setup-and-defaults.md` | OAuth workflow assessment, I19 source inventory, backend abstractions, parity matrix, owned default switch, command-adapter demotion, first final-audit evidence. |
| D77-D122 | `d077-d122-owned-backend-slices.md` | Crawl4AI and agent-browser feature slices for owned options, markdown behavior, CDP rendering/readiness, Chrome startup/discovery, overlays, linked images, shadow DOM. |
| D123-D150 | `d123-d150-module-decomposition.md` | Original extraction/CDP/test module decomposition through I19i completion. |
| D151-D157 | `d151-d157-auth-and-browser-import.md` | OAuth-safe authorization API/CLI and conservative browser-neutral Chrome import surface. |
| D158-D167 | `d158-d167-recent-migration-followups.md` | Recent source-backed extraction improvements and follow-up decomposition slices. |
| D168-D176 | Per-decision files | Knowledge/reference compaction, engine-refactor planning, wrapper introduction, naming cleanup, and direct engine coverage. |
| D177-D201 | Per-decision files | Module/test decomposition plus the latest owned extractor/browser parity improvements. |

## Current Open Migration Gaps

- I19d remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
- I19e remains open for current-tab attach, broader rendered JavaScript parity, real logged-in profile/keychain smoke coverage, cross-platform close/process lifecycle parity, and fuller startup/error classification.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.
