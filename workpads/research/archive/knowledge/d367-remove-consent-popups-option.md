# D367: Remove Consent Popups Option

## Decision

Owned extraction now accepts Crawl4AI's `crawl4ai.remove_consent_popups` backend option for generic cookie/GDPR consent cleanup.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/async_configs.py` defines `CrawlerRunConfig.remove_consent_popups` as a default-false option for GDPR/cookie consent popups.
- `crawl4ai/async_crawler_strategy.py` runs consent-popup cleanup before generic overlay cleanup.
- `crawl4ai/js_snippet/remove_consent_popups.js` includes generic cookie consent, banner, notice, law, popup, overlay, GDPR, and cookie iframe selectors.

## Implementation

- `OwnedExtractorOptions` validates and stores `crawl4ai.remove_consent_popups`.
- Owned HTML cleanup applies a consent-specific selector set before generic overlay removal.
- The owned implementation removes generic consent elements only; it does not click accept buttons, set cookies, or add site-specific paywall/login handling.
- `crawl4ai.remove_overlay_elements=false` remains separate, so callers can remove consent noise without removing every modal/dialog.
- The command/mock validation surface, README, and OpenCode tool text list the new supported option.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo test aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test get_command_backend_accepts_scan_full_page_options`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

## Follow-Up

- I19d remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
