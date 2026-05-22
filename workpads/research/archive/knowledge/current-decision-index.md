# Current Decision Index

This archive file is the compact router for decision history that used to live in top-level `knowledge.md`.
Open the referenced child indexes for older per-decision rows, then open the per-decision archive files for exact evidence, validation commands, and source paths.

## Current Routing

| Need | Open |
| --- | --- |
| D168-D231 individual decision rows | `decision-index/d168-d231.md` |
| D232-D292 individual decision rows | `decision-index/d232-d292.md` |
| D293-D346 individual decision rows | `decision-index/d293-d346.md` |
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
| D168-D356 | `decision-index/` plus per-decision files | Module/test/support decomposition plus owned extractor/browser parity improvements, agent-facing option docs, support-file routing refreshes, login-session injection, markdown options, CDP diagnostics, and transport/readiness coverage. |

## Current Open Migration Gaps

- I19d remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
- I19e remains open for broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and still-fuller startup/error classification.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.
