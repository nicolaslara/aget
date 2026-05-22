# D369: Cache Mode Compatibility Options

## Decision

Owned extraction now accepts Crawl4AI cache-mode compatibility options only when they preserve the current uncached owned-extractor behavior.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/cache_context.py` defines `CacheMode` values: `enabled`, `disabled`, `read_only`, `write_only`, and `bypass`.
- `crawl4ai/async_configs.py` defaults `CrawlerRunConfig.cache_mode` to `CacheMode.BYPASS`.
- `crawl4ai/async_configs.py` documents legacy boolean cache flags and marks setting non-default legacy flags as deprecated in favor of `cache_mode`.
- `crawl4ai/async_webcrawler.py` creates `CacheContext` from `config.cache_mode`; cache read/write behavior is meaningful only when a cache store exists.

## Implementation

- Owned extraction accepts `crawl4ai.cache=bypass`, `crawl4ai.cache=disabled`, `crawl4ai.cache_mode=bypass`, and `crawl4ai.cache_mode=disabled` as validated no-cache compatibility options.
- Owned extraction rejects `enabled`, `read_only`, and `write_only` because `aget` does not yet have an owned cache store for extraction results.
- The `crawl4ai.cache` spelling is kept as a compatibility alias because existing `aget` CLI/output coverage already records that key.
- The optional Crawl4AI command compatibility helper accepts both `cache` and `cache_mode`, validates the Crawl4AI cache-mode value set, maps `cache` to `cache_mode`, and converts strings into the installed `CacheMode` enum before constructing `CrawlerRunConfig`.
- Command mock validation, README, and OpenCode tool text list the supported cache options and the owned-path cache limitation.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo test aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/crawl4ai_extract.py scripts/aget_crawl4ai_compat/options.py`
- Python helper smoke for `cache`/`cache_mode` enum conversion and invalid-mode rejection.
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

## Follow-Up

- I19d remains open for fuller Crawl4AI-quality readability/markdown, richer rendered-page readiness, and any future owned cache store work.
