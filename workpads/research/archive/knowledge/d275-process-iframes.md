# D275: Crawl4AI Iframe Processing Option

## Source Inspection

- Inspected `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`.
- `AsyncCrawlerStrategy.process_iframes` assigns iframe IDs, gets each iframe content frame, waits for frame load, extracts `document.body.innerHTML`, parses it, and replaces accessible iframe elements with `div.extracted-iframe-content-*` before final HTML capture.
- Crawl4AI treats inaccessible or failing iframes as warnings/errors in logs and continues with the remaining page.
- Crawl4AI includes `process_iframes` among options that require a browser-rendered path.

## Decision

- Added owned support for `crawl4ai.process_iframes`, defaulting to `false`.
- When enabled, owned extraction forces the CDP-rendered path and runs iframe replacement before rendered overlay cleanup and final HTML capture.
- Accessible iframe body HTML is copied into `div.extracted-iframe-content-*`; inaccessible iframe counts produce extraction warnings instead of hard failures.
- CDP script/evaluation failures also become warnings so the main page can still be extracted.
- The compatibility Crawl4AI helper and mock backend option validation now accept `process_iframes`.

## Boundary

- This is generic iframe body extraction only. It does not bypass browser same-origin restrictions, access controls, or site policy boundaries.
- Main-content selection still runs after rendered HTML capture, so a dense iframe body can become the selected content and the outer page shell may be omitted.

## Validation

- `cargo test iframe_process_expression`
- `cargo test --test mock_site_cli aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo test --test mock_site_browser aget_extractor_backend_processes_accessible_iframes_with_chrome -- --ignored`
