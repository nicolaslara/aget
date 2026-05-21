# D222: Executable Script Render Detection

Date: 2026-05-21

Source inspected:

- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`: normal `http://` and `https://` crawls route through `_crawl_web` and call Playwright `page.goto`, so browser rendering is the default Crawl4AI web path rather than a static-HTML heuristic.
- `src/extraction/owned/mod.rs`: owned extraction uses a fast static path and escalates to CDP rendering only when session storage, image waits, wait retries, or script-bearing HTML indicate rendering is needed.

Decision:

- Broaden the owned script-bearing heuristic to recognize common executable JavaScript MIME types:
  - no `type` or an empty `type`;
  - `module`;
  - `text/javascript`;
  - `application/javascript`;
  - `text/ecmascript`;
  - `application/ecmascript`;
  - `text/jscript`.
- Normalize MIME types before matching so `text/javascript; charset=utf-8` triggers rendering.
- Keep non-executable script data types such as JSON-LD, JSON, import maps, and speculation rules on the static path.

Boundary:

- This improves Crawl4AI-like readiness without making every public page launch Chrome by default.
- It does not execute user-provided JavaScript waits; the v1 authenticated-session safety boundary remains unchanged.

Validation:

- `cargo test extraction::owned::tests::script_detection -- --nocapture`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
