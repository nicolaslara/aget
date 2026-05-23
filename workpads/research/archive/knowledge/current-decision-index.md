# Current Decision Index

This archive file is the compact router for decision history that used to live in top-level `knowledge.md`.
Open the referenced child indexes for older per-decision rows, then open the per-decision archive files for exact evidence, validation commands, and source paths.

## Current Routing

| Need | Open |
| --- | --- |
| D168-D231 individual decision rows | `decision-index/d168-d231.md` |
| D232-D292 individual decision rows | `decision-index/d232-d292.md` |
| D293-D346 individual decision rows | `decision-index/d293-d346.md` |
| D347-D399 individual decision rows | Recent decision routing below |
| Foundation and early migration bundles | Full archive routing below |

## Recent Decision Routing

| Decision | Archive | Use |
| --- | --- | --- |
| D340 | `d340-runtime-evaluate-exceptions.md` | CDP `Runtime.evaluate` exception details surfaced as stable owned extraction failures. |
| D341 | `d341-cdp-readiness-evaluate-exceptions.md` | CDP readiness/preprocessing `Runtime.evaluate` exception details surfaced consistently. |
| D342 | `d342-cdp-storage-evaluate-exceptions.md` | CDP storage load/export `Runtime.evaluate` exception details surfaced consistently. |
| D343 | `d343-blank-navigation-error-text.md` | CDP blank-response storage navigation surfaces `Page.navigate` `errorText`. |
| D344 | `d344-image-readiness-timeout-boundary.md` | Crawl4AI image readiness keeps its own short timeout boundary separate from `wait_for_timeout`. |
| D345 | `d345-google-doc-markdown-option.md` | Crawl4AI `google_doc` markdown option for source-compatible styled inline emphasis. |
| D346 | `d346-binary-cdp-response-frames.md` | agent-browser-style binary CDP WebSocket response frames accepted by owned transport. |
| D347 | `d347-current-decision-index-split.md` | Support-file split that keeps the current decision index as a compact router. |
| D348 | `d348-google-list-indent-option.md` | Crawl4AI `google_list_indent` markdown option for Google Docs-style list indentation. |
| D349 | `d349-invalid-binary-cdp-frames.md` | agent-browser-style invalid binary CDP WebSocket response frames skipped by owned transport. |
| D350 | `d350-malformed-cdp-frames.md` | agent-browser-style malformed text and UTF-8 binary CDP WebSocket frames skipped by owned transport. |
| D351 | `d351-cdp-websocket-size-limits.md` | agent-browser-style unlimited CDP WebSocket message and frame sizes for owned transport. |
| D352 | `d352-cdp-websocket-keepalive.md` | agent-browser-style CDP WebSocket keepalive pings while waiting for command responses. |
| D353 | `d353-cdp-dialog-auto-handling.md` | agent-browser-style automatic `alert`/`beforeunload` dialog acceptance while leaving `confirm`/`prompt` explicit. |
| D354 | `d354-cdp-transport-test-split.md` | Mechanical split of CDP WebSocket transport tests out of setup/attach-domain tests. |
| D355 | `d355-unlabeled-density-main-content.md` | Crawl4AI-style density scoring lets strong unlabeled content containers win main-content extraction. |
| D356 | `d356-body-fallback-page-chrome-cleanup.md` | Crawl4AI-style page-chrome tag removal when automatic main-content extraction falls back to body/root. |
| D357 | `d357-d021-d034-archive-split.md` | Support-file split that keeps D21-D34 as a compact router to smaller archive sections. |
| D358 | `d358-preserve-tags-markdown-option.md` | Crawl4AI `preserve_tags` markdown option for preserving configured subtrees as raw HTML blocks. |
| D359 | `d359-handle-code-in-pre-option.md` | Crawl4AI `handle_code_in_pre` markdown option for preserving code markers inside fenced pre blocks. |
| D360 | `d360-refresh-local-chrome-smokes.md` | Local Chrome ignored smoke expectations refreshed to match owned text block-boundary output. |
| D361 | `d361-metadata-title-fallback-parity.md` | Crawl4AI-compatible metadata title fallback coverage for missing `<title>` pages. |
| D362 | `d362-twitter-title-fallback-parity.md` | Crawl4AI-compatible metadata title fallback coverage for Twitter-only title pages. |
| D363 | `d363-ignore-anchors-alias.md` | Crawl4AI/html2text `ignore_anchors` compatibility alias for owned `ignore_links` markdown behavior. |
| D364 | `d364-large-file-ergonomics-audit.md` | Current tracked size audit: no new split needed and `tasks.md` stays uncompacted. |
| D365 | `d365-frame-tree-storage-export.md` | agent-browser-style frame-tree storage origin export within explicit allow-domain scope. |
| D366 | `d366-excluded-selector-option.md` | Crawl4AI `excluded_selector` cleanup option for owned extraction selector removal. |
| D367 | `d367-remove-consent-popups-option.md` | Crawl4AI `remove_consent_popups` cleanup option for generic cookie/GDPR consent removal. |
| D368 | `d368-css-selector-option.md` | Crawl4AI `css_selector` compatibility option for owned extraction content selection. |
| D369 | `d369-cache-mode-compat-options.md` | Crawl4AI cache-mode compatibility options for the owned uncached extraction boundary. |
| D370 | `d370-keep-attrs-option.md` | Crawl4AI `keep_attrs` cleanup option for preserving explicitly named attributes in owned cleaned HTML. |
| D371 | `d371-prettiify-option.md` | Crawl4AI `prettiify` cleanup option for formatting owned cleaned HTML output only. |
| D372 | `d372-process-in-browser-local-content.md` | Crawl4AI `process_in_browser` local-content routing through owned CDP rendering. |
| D373 | `d373-user-agent-option.md` | Crawl4AI explicit `user_agent` option for owned static HTTP and CDP-rendered page requests. |
| D374 | `d374-options-waits-test-split.md` | Mechanical split of oversized AgetExtractor options/waits parity helper by behavior. |
| D375 | `d375-locale-timezone-options.md` | Crawl4AI explicit `locale` and `timezone_id` browser-context options for owned CDP rendering. |
| D376 | `d376-owned-page-module-split.md` | Mechanical split of owned page orchestration, cleaned HTML extraction, and local-input routing. |
| D377 | `d377-attached-page-capture-test-split.md` | Mechanical split of attached-page capture CDP tests by behavior. |
| D378 | `d378-markdown-normalize-split.md` | Mechanical split of markdown normalization, wrapping, escaping, and URL helper boundaries. |
| D379 | `d379-markdown-table-split.md` | Mechanical split of markdown table rendering, row collection, bypass, ignored-table, and padding helpers. |
| D380 | `d380-markdown-writer-split.md` | Mechanical split of markdown writer text, reference-link, and abbreviation helpers. |
| D381 | `d381-markdown-dispatch-split.md` | Mechanical split of markdown node traversal and tag-dispatch helpers. |
| D382 | `d382-owned-option-apply-split.md` | Mechanical split of owned extractor backend-option application helpers. |
| D383 | `d383-owned-content-split.md` | Mechanical split of owned content selection, cleanup, and rendering helpers. |
| D384 | `d384-inline-markdown-assertion-split.md` | Mechanical split of inline markdown parity assertion helpers. |
| D385 | `d385-markdown-fixture-route-split.md` | Mechanical split of markdown mock-site fixture route helpers. |
| D386 | `d386-get-validation-test-split.md` | Mechanical split of get CLI validation test helpers. |
| D387 | `d387-extractor-option-validation-split.md` | Mechanical split of AgetExtractor option validation helpers. |
| D388 | `d388-aget-api-support-split.md` | Mechanical split of Aget API support helpers. |
| D389 | `d389-markdown-link-image-assertion-split.md` | Mechanical split of markdown link/image assertion helpers. |
| D390 | `d390-cleanup-assertion-split.md` | Mechanical split of owned extractor cleanup assertion helpers. |
| D391 | `d391-browser-render-capture-split.md` | Mechanical split of browser CDP render capture helpers. |
| D392 | `d392-get-failure-test-split.md` | Mechanical split of get CLI failure tests. |
| D393 | `d393-aget-backend-facade-split.md` | Mechanical split of Aget facade backend helpers. |
| D394 | `d394-get-session-replay-test-split.md` | Mechanical split of get CLI session replay tests. |
| D395 | `d395-get-session-fallback-test-split.md` | Mechanical split of get CLI session fallback tests. |
| D396 | `d396-markdown-inline-renderer-split.md` | Mechanical split of markdown inline renderer helpers. |
| D397 | `d397-session-login-start-test-split.md` | Mechanical split of session login start tests. |
| D398 | `d398-mock-backend-config-split.md` | Mechanical split of mock backend config helpers. |
| D399 | `d399-browser-cdp-client-page-split.md` | Mechanical split of browser CDP client page/runtime helpers. |

## Full Archive Routing

| Range | Archive | Contents |
| --- | --- | --- |
| D1-D20 | `d001-d020-foundation-research.md` | Product premise, prior art, auth/session research, benchmark findings, and first PoC direction. |
| R12 | `r12-mvp-architecture-proposal.md` index to section files | MVP architecture proposal, CLI/API boundary, session model, security notes, implementation plan, and review recommendations. |
| D21-D34 | `d021-d034-poc-session-api.md` index to section files | Early implementation, session replay/import, output shaping, auth ownership, login bootstrap, and response API decisions. |
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
| D168-D399 | `decision-index/` plus per-decision files | Module/test/support decomposition plus owned extractor/browser parity improvements, agent-facing option docs, support-file routing refreshes, login-session injection, markdown options, metadata coverage, CDP diagnostics, transport/readiness coverage, ignored local Chrome smoke refreshes, support-file archive splits, the current no-task-compaction size audit, frame-tree storage export, excluded selector cleanup, consent-popup cleanup, CSS selector option support, cache-mode compatibility options, `keep_attrs` cleanup support, `prettiify` cleaned-HTML formatting, `process_in_browser` local-content routing, explicit request identity via `user_agent`/`locale`/`timezone_id`, the AgetExtractor options/waits parity helper split, the owned page module split, the attached-page capture test split, the markdown normalize helper split, the markdown table renderer split, the markdown writer helper split, the markdown render-dispatch split, the owned option-application split, the owned content extraction split, the inline markdown assertion split, the markdown fixture route split, the get CLI validation test split, the AgetExtractor option validation split, the Aget API support helper split, the markdown link/image assertion split, the cleanup assertion split, the browser render capture split, the get failure test split, the Aget backend facade split, the get session replay test split, the get session fallback test split, the markdown inline renderer split, the session login start test split, the mock backend config split, and the browser CDP client page/runtime split. |

## Current Open Migration Gaps

- I19d remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
- I19e remains open for broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and still-fuller startup/error classification.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.
