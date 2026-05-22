# D290: Owned Extractor Options Parse Split

Decision: owned extractor option parsing helpers now live in a child module.

Implementation boundary:

- `src/extraction/owned/options.rs` still owns `OwnedExtractorOptions`, defaults, supported option dispatch, and `validate_owned_extraction_options`.
- `src/extraction/owned/options/parse.rs` owns private parsing helpers for URLs, lists, booleans, durations, waits, selector lists, and CSS-only wait validation.
- Supported `crawl4ai.*` option names, default values, and error text are unchanged.
- The JavaScript-wait safety boundary is unchanged: `--wait-for-selector` still rejects `js:` and JavaScript-like expressions before backend execution.
- `workpads/research/tasks.md` was not compacted.

Validation:

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_real_helper_rejects_javascript_wait_before_crawl4ai_import --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
