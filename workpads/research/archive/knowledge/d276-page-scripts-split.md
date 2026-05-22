# D276: Browser CDP Page-Scripts Split

## Decision

- Split `src/browser_cdp/page_scripts.rs` into behavior-owned modules under `src/browser_cdp/page_scripts/`.
- Kept `src/browser_cdp/page_scripts/mod.rs` as the stable route that re-exports the existing helper names at the same `browser_cdp` visibility.
- Moved storage scripts, selector/full-page readiness scripts, overlay cleanup, iframe processing, and shadow DOM scripts into separate files.

## Boundary

- This was a mechanical decomposition only. It did not change generated JavaScript strings, CDP evaluation order, or public browser/extraction behavior.
- `workpads/research/tasks.md` was not compacted; it only received the I19dg task record.

## Validation

- `cargo test browser_cdp::tests::scripts`
- `cargo test browser_cdp::tests::chrome::cdp_client`
