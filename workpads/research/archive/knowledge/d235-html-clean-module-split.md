# D235: HTML Cleanup Module Split

## Decision

- Split the largest remaining source module, `src/extraction/html_clean.rs`, into `src/extraction/html_clean/`.
- Keep `mod.rs` as the stable cleanup facade for selector parsing, generic selected-element removal, excluded-tag cleanup, and overlay cleanup.
- Move attribute, comment, base64-image, and empty-element cleanup into `attributes.rs`.
- Move external URL, excluded-domain URL, and social-link cleanup into `urls.rs`.
- Do not compact `workpads/research/tasks.md`; it remains the executable task ledger.

## Rationale

- The prior extraction/CDP monoliths are already split, but the current size scan showed HTML cleanup as the largest tracked source module after `Cargo.lock` and the task ledger.
- Future Crawl4AI option ports often touch only one cleanup subdomain, so this split reduces the amount of unrelated code agents need to load.
- The split is mechanical and preserves caller-facing extraction module paths.

## Validation

- `cargo fmt --check`
- `cargo test extraction::html_clean::tests -- --nocapture`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
