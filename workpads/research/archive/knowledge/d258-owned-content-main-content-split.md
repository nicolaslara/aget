# D258: Owned Content Main-Content Split

## Decision

Split owned content main-content candidate discovery and scoring out of `src/extraction/owned/content.rs`.

## Boundary

- `src/extraction/owned/content.rs` keeps owned content extraction composition, selector fallback, target-element collection, cleanup sequencing, selected-element lookup, output assembly, and markdown base URL handling.
- `src/extraction/owned/content/main_content.rs` owns default main-content candidate selection and scoring, including Crawl4AI-like pruning density, tag weights, label bonuses and penalties, class/id noise penalties, excluded ancestor checks, and word-threshold gating.

## Compatibility

The split is mechanical. `extract_owned_content` and `markdown_base_url` remain the internal owned-extraction entrypoints, and selected root handling still uses `ego_tree::NodeId` so the existing cleanup pipeline can operate on the same document tree.

## Validation

- `cargo test --test mock_site_cli aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
