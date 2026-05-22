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
| D250 | `d250-mock-site-browser-test-split.md` | Mechanical split of mock-site browser integration tests into behavior-owned modules. |
| D251 | `d251-extraction-pipeline-helper-split.md` | Mechanical split of extraction pipeline finalization helpers from public extraction orchestration. |
| D252 | `d252-module-decomposition-archive-split.md` | Split the historical D123-D150 module-decomposition archive into smaller routed section files. |
| D253 | `d253-session-cli-root-test-split.md` | Mechanical split of remaining session CLI root tests into behavior-owned modules. |
| D254 | `d254-mock-site-support-split.md` | Mechanical split of shared mock-site support server helpers into protocol and routes modules. |
| D255 | `d255-owned-extraction-page-pipeline-split.md` | Mechanical split of owned extraction page pipeline helpers from backend adapter entrypoints. |
| D256 | `d256-main-session-import-login-split.md` | Mechanical split of binary session import and login command handlers from the session dispatcher. |
| D257 | `d257-browser-cdp-diagnostics-split.md` | Mechanical split of browser CDP startup diagnostics helpers from endpoint discovery. |
| D258 | `d258-owned-content-main-content-split.md` | Mechanical split of owned content main-content selection/scoring helpers from extraction composition. |
| D259 | `d259-agent-browser-fallback-command-split.md` | Mechanical split of agent-browser fallback command adapter helpers from fallback orchestration. |
| D260 | `d260-browser-cdp-readiness-split.md` | Mechanical split of browser CDP runtime readiness/evaluation helpers from navigation. |
| D261 | `d261-chrome-profile-discovery-split.md` | Mechanical split of Chrome profile discovery/name-resolution helpers from profile preparation/copying. |
| D262 | `d262-chrome-process-launch-split.md` | Mechanical split of Chrome process launch command helpers from process lifecycle/retry orchestration. |
| D263 | `d263-session-wrapper-v1-slice-split.md` | Support-file split of the historical V1 thin-wrapper implementation spec into smaller routed archive files without compacting tasks. |
| D264 | `d264-session-wrapper-plan-split.md` | Support-file split of the historical session-wrapper implementation plan into smaller routed archive files without compacting tasks. |
| D265 | `d265-session-wrapper-product-spec-split.md` | Support-file split of the historical session-wrapper product spec into smaller routed archive files without compacting tasks. |
| D266 | `d266-internal-link-cleanup.md` | Crawl4AI `exclude_internal_links` compatibility option for owned cleanup and command helper forwarding. |
| D267 | `d267-linked-heading-markdown.md` | Crawl4AI `CustomHTML2Text` linked-heading markdown behavior for anchors wrapping heading elements. |
| D268 | `d268-r12-architecture-archive-split.md` | Support-file split of the historical R12 architecture proposal into smaller routed archive files without compacting tasks. |
| D269 | `d269-d092-d108-archive-split.md` | Support-file split of the historical D92-D108 markdown/browser decision bundle into smaller routed archive files without compacting tasks. |
| D270 | `d270-d064-d076-archive-split.md` | Support-file split of the historical D64-D76 browser/default-switch decision bundle into smaller routed archive files without compacting tasks. |
| D271 | `d271-engine-refactor-plan-split.md` | Support-file split of the historical Aget engine refactor plan into smaller routed files without compacting tasks. |
| D272 | `d272-d077-d091-archive-split.md` | Support-file split of the historical D77-D91 owned extractor/browser decision bundle into smaller routed files without compacting tasks. |
| D273 | `d273-d109-d122-archive-split.md` | Support-file split of the historical D109-D122 selector/overlay/shadow decision bundle into smaller routed files without compacting tasks. |
| D274 | `d274-oauth-login-design-split.md` | Support-file split of the OAuth-safe browser login design into smaller routed files without compacting tasks. |
| D275 | `d275-process-iframes.md` | Crawl4AI `process_iframes` option for owned CDP rendering and compatibility option forwarding. |
| D276 | `d276-page-scripts-split.md` | Mechanical split of browser CDP page-script helpers into behavior-owned modules. |
| D277 | `d277-local-content-urls.md` | Crawl4AI `raw:`, `raw://`, and `file://` local-content URL compatibility in owned static extraction. |
| D278 | `d278-base-url-option.md` | Crawl4AI `base_url` option for raw/local HTML link resolution in owned extraction. |
| D279 | `d279-main-content-negative-labels.md` | Crawl4AI-negative class/id labels exclude owned default main-content candidates. |
| D280 | `d280-d055-d063-archive-split.md` | Support-file split of the historical D55-D63 owned-extractor foundation bundle without compacting tasks. |
| D281 | `d281-page-metadata.md` | Crawl4AI-style generic page metadata propagation from owned and compatibility extraction. |
| D282 | `d282-fallback-page-metadata.md` | Owned browser fallback preserves extraction page metadata while command fallback stays empty. |
| D283 | `d283-cdp-networkidle-reset-timeout.md` | agent-browser-style owned CDP networkidle reset and timeout coverage. |
| D284 | `d284-cdp-navigation-timeout-messages.md` | agent-browser-style owned CDP lifecycle and networkidle timeout messages. |
| D285 | `d285-agent-facing-option-docs.md` | Agent-facing OpenCode tool option docs aligned with the owned extractor compatibility surface. |
| D286 | `d286-code-block-whitespace.md` | Crawl4AI-style owned markdown code-block whitespace preservation. |
| D287 | `d287-chrome-early-exit-message.md` | agent-browser-style owned Chrome early-exit startup message with exit code. |
| D288 | `d288-browser-cdp-discovery-test-split.md` | Mechanical split of browser CDP discovery tests into diagnostics, port-file, and endpoint behavior modules. |
| D289 | `d289-mock-site-docs-contract-split.md` | Mechanical split of mock-site docs-contract tests into behavior-focused modules. |
| D290 | `d290-owned-extractor-options-parse-split.md` | Mechanical split of owned extractor option parsing helpers into a child module. |
| D291 | `d291-owned-page-rendering-split.md` | Mechanical split of owned page rendering request and script/readiness helpers into child modules. |
| D292 | `d292-aget-facade-sessions-split.md` | Mechanical split of Aget facade session/import/login orchestration into a child module. |
| D293 | `d293-skip-internal-links.md` | Crawl4AI `skip_internal_links` markdown option for owned markdown and command compatibility. |
| D294 | `d294-include-sup-sub.md` | Crawl4AI `include_sup_sub` markdown option for owned markdown and command compatibility. |
| D295 | `d295-ignore-links.md` | Crawl4AI `ignore_links` markdown option for owned markdown and command compatibility. |
| D296 | `d296-ignore-images.md` | Crawl4AI `ignore_images` markdown option for owned markdown and command compatibility. |
| D297 | `d297-ignore-emphasis.md` | Crawl4AI `ignore_emphasis` markdown option for owned markdown and command compatibility. |
| D298 | `d298-protect-links.md` | Crawl4AI `protect_links` markdown option for owned markdown and command compatibility. |
| D299 | `d299-escape-snob.md` | Crawl4AI `escape_snob` markdown option for owned markdown and command compatibility. |
| D300 | `d300-ignore-mailto-links.md` | Crawl4AI `ignore_mailto_links` markdown option for owned markdown and command compatibility. |
| D301 | `d301-ignore-tables.md` | Crawl4AI `ignore_tables` markdown option for owned markdown and command compatibility. |
| D302 | `d302-bypass-tables.md` | Crawl4AI `bypass_tables` markdown option for owned markdown and command compatibility. |
| D303 | `d303-use-automatic-links.md` | Crawl4AI `use_automatic_links` markdown option for owned markdown and command compatibility. |
| D304 | `d304-images-to-alt.md` | Crawl4AI `images_to_alt` markdown option for owned markdown and command compatibility. |
| D305 | `d305-images-as-html.md` | Crawl4AI `images_as_html` markdown option for owned markdown and command compatibility. |
| D306 | `d306-images-with-size.md` | Crawl4AI `images_with_size` markdown option for owned markdown and command compatibility. |
| D307 | `d307-default-image-alt.md` | Crawl4AI `default_image_alt` markdown option for owned markdown and command compatibility. |
| D308 | `d308-workpad-support-routing.md` | Workpad support-file routing refresh that keeps `tasks.md` intact and moves dense current-history detail to archive. |
| D309 | `d309-quote-markers.md` | Crawl4AI `open_quote` and `close_quote` markdown options for owned markdown and command compatibility. |
| D310 | `d310-ul-item-mark.md` | Crawl4AI `ul_item_mark` markdown option for owned markdown and command compatibility. |
| D311 | `d311-emphasis-markers.md` | Crawl4AI `emphasis_mark` and `strong_mark` markdown options for owned markdown and command compatibility. |
| D312 | `d312-mock-site-markdown-test-split.md` | Mechanical split of mock-site extractor markdown assertions into behavior-owned child modules. |
| D313 | `d313-markdown-block-renderer-split.md` | Mechanical split of markdown block/list/code/quote rendering helpers into a child module. |
| D314 | `d314-owned-option-apply-split.md` | Mechanical split of owned extractor backend-option application into a child module. |
| D315 | `d315-playwright-tests-split.md` | Mechanical split of Playwright session composition tests into behavior-owned child modules. |
| D316 | `d316-mock-site-session-test-split.md` | Mechanical split of mock-site session integration tests into behavior-owned child modules. |
| D317 | `d317-crawl4ai-adapter-script-split.md` | Mechanical split of the Crawl4AI compatibility adapter script into behavior-owned helper modules. |
| D318 | `d318-cli-session-parser-test-split.md` | Mechanical split of CLI session parser unit tests into command-surface child modules. |
| D319 | `d319-attached-page-cdp-test-split.md` | Mechanical split of attached-page CDP client tests into behavior-owned child modules. |

## Full Archive Routing

| Range | Archive | Contents |
| --- | --- | --- |
| D1-D20 | `d001-d020-foundation-research.md` | Product premise, prior art, auth/session research, benchmark findings, and first PoC direction. |
| R12 | `r12-mvp-architecture-proposal.md` index to section files | MVP architecture proposal, CLI/API boundary, session model, security notes, implementation plan, and review recommendations. |
| D21-D34 | `d021-d034-poc-session-api.md` | Early implementation, session replay/import, output shaping, auth ownership, login bootstrap, and response API decisions. |
| D35-D48 | `d035-d048-poc-hardening-backends.md` | Generic extraction follow-up, agent integration, hardening, mocked tests, facade, and backend-pluggability decisions. |
| D49-D54 | `d049-d054-migration-setup-and-parity.md` | OAuth workflow assessment, API cleanup, dependency migration setup, backend abstractions, and parity matrix. |
| D55-D63 | `d055-d063-owned-extractor-foundation.md` index to section files | First owned extractor, transport, selector, markdown, CDP, and table-rendering slices. |
| D64-D76 | `d064-d076-browser-default-switch.md` index to section files | Browser/profile import, login lifecycle, main-content selection, owned default switch, command-adapter demotion, and first final-audit evidence. |
| D77-D91 | `d077-d091-owned-extractor-options-cleanup.md` index to section files | Owned extractor options, CDP rendering controls, cleaned-HTML cleanup, and early markdown tags. |
| D92-D108 | `d092-d108-markdown-browser-slices.md` index to section files | Markdown fidelity improvements plus Chrome/CDP discovery and startup fallback slices. |
| D109-D122 | `d109-d122-selector-overlay-shadow.md` index to section files | Hard breaks, selector behavior, Chrome retry diagnostics, overlay cleanup, linked images, and shadow DOM flattening. |
| D123-D150 | `d123-d150-module-decomposition.md` index to section files | Original extraction/CDP/test module decomposition through I19i completion. |
| D151-D157 | `d151-d157-auth-and-browser-import.md` | OAuth-safe authorization API/CLI and conservative browser-neutral Chrome import surface. |
| D158-D167 | `d158-d167-recent-migration-followups.md` | Recent source-backed extraction improvements and follow-up decomposition slices. |
| D168-D176 | Per-decision files | Knowledge/reference compaction, engine-refactor planning, wrapper introduction, naming cleanup, and direct engine coverage. |
| D177-D319 | Per-decision files | Module/test/support decomposition plus the latest owned extractor/browser parity improvements, agent-facing option docs, and support-file routing refreshes. |

## Current Open Migration Gaps

- I19d remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
- I19e remains open for broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and still-fuller startup/error classification.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.
