# D279: Main-Content Negative Labels

Decision: owned default main-content selection rejects candidates whose class/id contains Crawl4AI-negative labels.

Source inspection:

- `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py` defines generic negative patterns for navigation, footer/header/sidebar, ads, comments, promo, adverts, social, and share labels.
- The relevant-content candidate path combines class and id text and excludes elements when those negative patterns appear.
- `PruningContentFilter` also keeps a class/id weight using the same negative labels, but that is scoring-only and can still leave a noisy candidate in contention.

Implementation boundary:

- Owned default main-content candidate discovery now skips elements whose own `class` or `id` contains the Crawl4AI-negative label set.
- Existing page-chrome ancestor tag exclusion and explicit `crawl4ai.word_count_threshold` behavior are unchanged.
- Descendants are still independently eligible if they have their own content labels and are not inside excluded page-chrome tags.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
