# D419: HTML Cleanup URL Helper Split

Date: 2026-05-23

## Decision

Split the owned HTML cleanup URL helper from a single `urls.rs` file into a
route module plus focused helpers:

- `src/extraction/html_clean/urls/mod.rs`: public cleanup entrypoints used by
  the HTML cleanup pipeline.
- `src/extraction/html_clean/urls/elements.rs`: selector-driven DOM element
  removal for external, internal, and excluded-domain URL-bearing nodes.
- `src/extraction/html_clean/urls/domains.rs`: Crawl4AI-like URL/domain
  normalization, special URL classification, and excluded-domain option
  normalization.

## Rationale

The old file mixed three separate concerns: exported cleanup operations,
tree-mutation mechanics, and Crawl4AI-like domain matching. Keeping the exported
function names at the route boundary preserves the existing extraction pipeline
while letting future Crawl4AI-parity work inspect or edit domain matching
without loading DOM mutation code.

## Validation

- `cargo test --lib extraction::html_clean`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`
- Line-count check for the split modules:
  - `urls/mod.rs`: 71 lines
  - `urls/domains.rs`: 94 lines
  - `urls/elements.rs`: 79 lines
