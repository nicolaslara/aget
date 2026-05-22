# D282: Fallback Page Metadata Propagation

Decision: owned browser fallback extraction now preserves generic page metadata instead of dropping it at the fallback result boundary.

Source inspection:

- D281 source inspection remains the metadata boundary: Crawl4AI extracts generic page metadata before content filtering and cleanup and exposes it separately as `CrawlResult.metadata`.
- The command-backed `agent-browser` fallback compatibility path requests body HTML/text only, so it intentionally returns an empty metadata map.

Implementation boundary:

- `BrowserFallbackResult` now carries `page_metadata` alongside content, warnings, final URL, and extractor name.
- `run_owned_browser_fallback` forwards the metadata already produced by the owned extraction pipeline.
- Session-backed fallback finalization now copies `fallback.page_metadata` into `GetSuccess`.
- Command-backed `agent-browser` fallback and test doubles return an empty map to keep compatibility behavior stable.

Validation:

- `cargo fmt --check`
- `cargo test --test mock_site_browser aget_browser_fallback_replays_cookie_backed_session_without_agent_browser`
- `cargo test --test get_cli get_session_backend_failure_uses_agent_browser_fallback_with_composed_state`
- `cargo test`
- `git diff --check`
