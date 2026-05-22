# D278: Crawl4AI Base URL Option

Decision: the owned extractor supports `crawl4ai.base_url` for raw/local HTML link resolution.

Source inspection:

- `references/repos/crawl4ai/crawl4ai/async_configs.py` defines `CrawlerRunConfig.base_url` as the base URL for markdown link resolution, especially for raw HTML.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py` preserves `config.base_url` as the redirected URL for raw/local content instead of falling back to the raw HTML string.
- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py` selects `base_url` from config, then redirected URL, then original URL, and allows an HTML `<base href>` tag to override that value before markdown generation.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` passes the selected base URL into `CustomHTML2Text(baseurl=...)`, which resolves relative links and images.

Implementation boundary:

- `crawl4ai.base_url` is accepted as a non-empty absolute URL in owned extraction options.
- The configured base URL feeds the existing owned markdown/link-cleanup base calculation.
- HTML `<base href>` keeps precedence over the configured base URL, matching Crawl4AI's later base-tag override.
- The option is aligned across owned extraction, `scripts/crawl4ai_extract.py`, mock-backend validation, README option documentation, and unsupported-option error text.
- This does not change final URL reporting or session scope. It only affects relative URL resolution in extracted content.

Validation:

- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo test --test mock_site_cli aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
