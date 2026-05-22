# D344: Image readiness timeout boundary

## Decision

Owned CDP rendering preserves Crawl4AI's timeout boundary for image readiness:
`crawl4ai.wait_for_timeout` applies to explicit `wait_for` conditions, while
`crawl4ai.wait_for_images` uses its own short bounded image-completion wait.

## Source Evidence

- Crawl4AI snapshot: `references/repos/crawl4ai` at
  `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/async_configs.py` documents `wait_for_timeout` as a timeout for
  the `wait_for` condition and says it falls back to `page_timeout` when unset.
- `crawl4ai/async_crawler_strategy.py` applies `wait_for_timeout` only when
  `config.wait_for` is present.
- The same strategy checks `config.wait_for_images` separately with
  `csp_compliant_wait(..., timeout=1000)`, warning when images do not complete
  within that image-specific timeout.

## Owned Boundary

- The owned renderer continues to pass `wait_for_timeout.unwrap_or(page_timeout)`
  only to selector readiness.
- The owned image-readiness helper keeps its independent one-second deadline.
- New mock CDP coverage proves an explicit one-millisecond `wait_for_timeout`
  does not truncate image readiness; the renderer waits for a later successful
  image-completion response and emits no image timeout warning.

## Validation

- `cargo test browser_cdp_render_attached_page_keeps_wait_for_timeout_out_of_image_wait`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
