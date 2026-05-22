# D373: Crawl4AI Explicit `user_agent` Option

Date: 2026-05-23

## Decision

Owned extraction accepts `crawl4ai.user_agent` for explicit user-agent strings and applies it to both owned static HTTP requests and owned CDP-rendered page navigation.

This slice does not implement random user-agent generation, `user_agent_mode`, client hints synthesis, or arbitrary custom headers. Those are broader request-identity and fingerprinting surfaces that need separate product and privacy review.

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py` documents `CrawlerRunConfig.user_agent`, defaults it to `None`, stores it on the run config, and serializes it through `dump()`.
- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py` calls `crawler_strategy.update_user_agent(config.user_agent)` before crawling when a run-level user agent is supplied.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py` copies explicit run-level `user_agent` into browser config for non-persistent contexts and pushes updated `User-Agent` headers to the page before navigation.
- `references/repos/crawl4ai/crawl4ai/browser_manager.py` applies configured user agents to Playwright contexts and extra HTTP headers.

## Implementation Notes

- `src/extraction/owned/options.rs` stores an optional explicit user agent.
- `src/extraction/owned/options/apply.rs` accepts `crawl4ai.user_agent` and treats blank values as no override.
- `src/extraction/http.rs` uses the configured value for static HTTP `User-Agent`, falling back to `aget/0.1`.
- `src/browser_cdp/render.rs` applies the configured value with `Network.setUserAgentOverride` after enabling page/network domains and before state-loading or target navigation.
- The optional Crawl4AI compatibility helper forwards `user_agent` to real Crawl4AI when the installed version accepts it.

## Validation

Passed:

- `cargo test browser_cdp_sets_user_agent_override_for_page_session`
- `cargo test aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/crawl4ai_extract.py scripts/aget_crawl4ai_compat/options.py`
- Python helper parse smoke for `crawl4ai.user_agent=aget-test/2.0`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`
