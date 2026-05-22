# D314: Owned Extractor Option Application Split

Date: 2026-05-22

## Decision

Split the per-key owned extractor backend-option application logic out of `src/extraction/owned/options.rs` into `src/extraction/owned/options/apply.rs`.

## Boundary

- `src/extraction/owned/options.rs` remains the caller-facing options route and owns the `OwnedExtractorOptions` data model, defaults, CSS-wait precheck, Crawl4AI namespace check, and final validator entrypoint.
- `src/extraction/owned/options/apply.rs` owns the long per-key option match, typed parsing calls, field assignment, and supported-option error text.
- Existing parsing helpers remain in `src/extraction/owned/options/parse.rs`.

## Safety And Compatibility Notes

- This is a mechanical production-code decomposition only; owned extraction runtime behavior is unchanged.
- Unsupported-option error text remains unchanged.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The options route dropped from 325 lines to 135 lines; the option application helper is 189 lines.

## Validation

- `cargo fmt`
- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
