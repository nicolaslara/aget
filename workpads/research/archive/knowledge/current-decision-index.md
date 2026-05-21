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
| D202 | `d202-cdp-networkidle-load-event.md` | agent-browser-inspired network-idle load-event readiness. |
| D203 | `d203-owned-comment-cleanup.md` | Crawl4AI-inspired HTML comment cleanup for owned extraction. |
| D204 | `d204-aget-extractor-parity-test-split.md` | Mechanical AgetExtractor parity test helper split by behavior. |
| D205 | `d205-browser-cdp-chrome-test-split.md` | Mechanical browser CDP Chrome test split by behavior. |
| D206 | `d206-attached-page-render.md` | agent-browser-inspired attached-page CDP renderer for current-tab groundwork. |
| D207 | `d207-aget-browser-attached-page-seam.md` | AgetBrowser engine seam for attached-page CDP rendering. |
| D208 | `d208-aget-browser-cdp-endpoint-discovery.md` | AgetBrowser engine seam for explicit local CDP endpoint discovery. |
| D209 | `d209-aget-browser-current-tab-engine.md` | AgetBrowser engine seam composing explicit-port CDP discovery and attached-page rendering. |
| D210 | `d210-consent-gated-current-tab.md` | Public consent-gated current-tab CLI/API on owned browser and extraction paths. |
| D211 | `d211-current-tab-compat-env.md` | Current-tab stays on the owned browser/CDP engine when compatibility command env is set. |
| D212 | `d212-cli-parser-test-split.md` | Mechanical CLI parser unit-test split by command surface. |
| D213 | `d213-cli-integration-test-split.md` | Mechanical CLI integration-test split by behavior. |
| D214 | `d214-chrome-startup-stderr-diagnostics.md` | agent-browser-inspired labeled generic Chrome startup stderr diagnostics. |
| D215 | `d215-real-chrome-import-smoke.md` | Stronger opt-in real Chrome import smoke assertions for persisted scoped auth. |
| D216 | `d216-aget-browser-current-tab-split.md` | Mechanical AgetBrowser current-tab seam and internal test split. |
| D217 | `d217-css-wait-prefix-normalization.md` | Crawl4AI-style explicit `css:` wait-prefix normalization for owned selector waits. |
| D218 | `d218-scan-full-page-readiness.md` | Crawl4AI-style bounded `scan_full_page` rendered readiness for owned CDP capture. |
| D219 | `d219-scan-options-command-compat.md` | Crawl4AI command compatibility allowlist alignment for owned scan options. |
| D220 | `d220-fragment-link-markdown.md` | Crawl4AI `CustomHTML2Text` fragment-link markdown parity. |
| D221 | `d221-ordered-list-start-cleanup-boundary.md` | Crawl4AI default cleaned-HTML boundary for ordered-list `start` attributes. |
| D222 | `d222-executable-script-render-detection.md` | Crawl4AI-like executable script detection for owned render escalation. |
| D223 | `d223-remove-forms-cleanup.md` | Crawl4AI `remove_forms` cleanup option for owned extraction and command compatibility. |
| D224 | `d224-browser-cdp-client-test-split.md` | Mechanical split of mock browser CDP client tests by behavior. |
| D225 | `d225-session-store-split.md` | Mechanical split of session store layout, permissions, cleanup, and tests. |
| D226 | `d226-keep-data-attributes-cleanup.md` | Crawl4AI `keep_data_attributes` cleanup option for owned cleaned HTML. |
| D227 | `d227-exclude-all-images-cleanup.md` | Crawl4AI `exclude_all_images` cleanup option for owned extraction. |
| D228 | `d228-external-link-image-cleanup.md` | Crawl4AI external link and image cleanup options for owned extraction. |
| D229 | `d229-exclude-domains-cleanup.md` | Crawl4AI `exclude_domains` cleanup option for owned extraction. |
| D230 | `d230-social-media-link-cleanup.md` | Crawl4AI `exclude_social_media_links` cleanup option for owned extraction. |
| D231 | `d231-custom-social-media-domains.md` | Crawl4AI `exclude_social_media_domains` custom link-domain cleanup option for owned extraction. |
| D232 | `d232-archive-knowledge-bundle-split.md` | Split the largest historical knowledge bundles into smaller routed archive files without compacting tasks. |
| D233 | `d233-main-content-chrome-ancestor-skip.md` | Crawl4AI pruning-filter-inspired page-chrome ancestor skip for default main-content candidates. |
| D234 | `d234-remove-overlay-elements-option.md` | Explicit `crawl4ai.remove_overlay_elements` option support while preserving aget's default overlay cleanup. |
| D235 | `d235-html-clean-module-split.md` | Mechanical split of owned HTML cleanup helpers into behavior-owned modules. |
| D236 | `d236-main-content-word-threshold.md` | Crawl4AI pruning-filter-inspired word-count threshold for default main-content candidates. |
| D237 | `d237-only-text-cleaned-html.md` | Crawl4AI `only_text` inline-tag replacement for owned cleaned HTML. |
| D238 | `d238-owned-text-block-boundaries.md` | Crawl4AI compatibility-helper-style block boundaries for owned text output. |
| D239 | `d239-empty-pruning-threshold-boundary.md` | Keep Crawl4AI `word_count_threshold` out of cleaned-HTML empty-leaf pruning. |
| D240 | `d240-link-label-code-markdown.md` | Crawl4AI `CustomHTML2Text` link-label handling for inline code in owned markdown. |
| D241 | `d241-windows-chrome-process-lifecycle.md` | agent-browser-inspired Windows Chrome process-group launch and pid cleanup hooks. |
| D242 | `d242-chrome-stderr-tail-diagnostics.md` | agent-browser-inspired last-five-line generic Chrome startup stderr diagnostics. |
| D243 | `d243-session-wrapper-spec-archive-split.md` | Support-file split of historical session-wrapper PoC spec without compacting tasks. |
| D244 | `d244-chrome-launch-stability-flags.md` | agent-browser-inspired generic Chrome launch stability/noise-control flags. |
| D245 | `d245-headed-chrome-window-size.md` | agent-browser-inspired headed Chrome launch window-size boundary. |
| D246 | `d246-chrome-process-test-split.md` | Mechanical split of Chrome process launch-command tests from production launcher code. |
| D247 | `d247-current-tab-facade-split.md` | Mechanical split of current-tab facade options and orchestration from `src/aget/mod.rs`. |
| D248 | `d248-aget-extractor-route-fixture-split.md` | Mechanical split of the AgetExtractor mock-site route fixture into behavior-owned route modules. |
| D249 | `d249-session-authorize-test-split.md` | Mechanical split of session authorization CLI tests into behavior-owned scenario modules. |

## Full Archive Routing

| Range | Archive | Contents |
| --- | --- | --- |
| D1-D20 | `d001-d020-foundation-research.md` | Product premise, prior art, auth/session research, benchmark findings, and first PoC direction. |
| R12 | `r12-mvp-architecture-proposal.md` | MVP architecture proposal, CLI/API boundary, session model, security notes, implementation plan, and review recommendations. |
| D21-D34 | `d021-d034-poc-session-api.md` | Early implementation, session replay/import, output shaping, auth ownership, login bootstrap, and response API decisions. |
| D35-D48 | `d035-d048-poc-hardening-backends.md` | Generic extraction follow-up, agent integration, hardening, mocked tests, facade, and backend-pluggability decisions. |
| D49-D54 | `d049-d054-migration-setup-and-parity.md` | OAuth workflow assessment, API cleanup, dependency migration setup, backend abstractions, and parity matrix. |
| D55-D63 | `d055-d063-owned-extractor-foundation.md` | First owned extractor, transport, selector, markdown, CDP, and table-rendering slices. |
| D64-D76 | `d064-d076-browser-default-switch.md` | Browser/profile import, login lifecycle, main-content selection, owned default switch, command-adapter demotion, and first final-audit evidence. |
| D77-D91 | `d077-d091-owned-extractor-options-cleanup.md` | Owned extractor options, CDP rendering controls, cleaned-HTML cleanup, and early markdown tags. |
| D92-D108 | `d092-d108-markdown-browser-slices.md` | Markdown fidelity improvements plus Chrome/CDP discovery and startup fallback slices. |
| D109-D122 | `d109-d122-selector-overlay-shadow.md` | Hard breaks, selector behavior, Chrome retry diagnostics, overlay cleanup, linked images, and shadow DOM flattening. |
| D123-D150 | `d123-d150-module-decomposition.md` | Original extraction/CDP/test module decomposition through I19i completion. |
| D151-D157 | `d151-d157-auth-and-browser-import.md` | OAuth-safe authorization API/CLI and conservative browser-neutral Chrome import surface. |
| D158-D167 | `d158-d167-recent-migration-followups.md` | Recent source-backed extraction improvements and follow-up decomposition slices. |
| D168-D176 | Per-decision files | Knowledge/reference compaction, engine-refactor planning, wrapper introduction, naming cleanup, and direct engine coverage. |
| D177-D249 | Per-decision files | Module/test/support decomposition plus the latest owned extractor/browser parity improvements. |

## Current Open Migration Gaps

- I19d remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
- I19e remains open for broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and still-fuller startup/error classification.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.
