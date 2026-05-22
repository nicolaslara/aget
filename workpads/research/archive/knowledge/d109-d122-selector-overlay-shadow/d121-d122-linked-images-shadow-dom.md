# D121-D122: Linked Images And Shadow DOM

### D121: I19d preserves linked image markdown

The next markdown-quality slice ports a Crawl4AI/html2text image-inside-link behavior. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `HTML2Text.handle_tag` opens a link label before an `<img>` child and emits image markdown before the anchor close renders the outer link target. With inline links enabled by default, an anchor wrapping a single image becomes linked-image markdown such as `[![alt](image)](href)`.

`OwnedExtractorBackend` now detects the narrow single-image-anchor case and renders it as linked-image markdown instead of escaping the image markdown into the link label text. Mixed text/image anchors continue through the existing generic link path until a broader html2text inline-label model is justified.

Validation:

- `cargo fmt --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `git diff --check`
- `cargo test`

Confidence: High for single-image anchors. The behavior is source-backed and covered by the static markdown parity fixture; broader mixed inline link-image formatting remains a separate markdown-quality follow-up.

### D122: I19d supports optional shadow DOM flattening in owned CDP rendering

The next rendered-readiness slice ports Crawl4AI's opt-in shadow DOM flattening behavior. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `CrawlerRunConfig.flatten_shadow_dom` defaults to false; `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI injects an `attachShadow` override before page work and evaluates `js_snippet/flatten_shadow_dom.js` instead of normal page content capture when the option is enabled; and `references/repos/crawl4ai/crawl4ai/js_snippet/flatten_shadow_dom.js`, which serializes shadow roots, resolves slots, skips shadow-scoped styles, and falls back to normal capture when flattening returns no content.

`OwnedExtractorBackend` now accepts `crawl4ai.flatten_shadow_dom=true` and passes it to the owned CDP renderer. The renderer injects an `attachShadow` override before navigation so newly-created closed roots become open in the controlled, temporary browser, then uses a shadow-aware serializer before HTML capture. The compatibility helper also accepts the same namespaced option and forwards it to Crawl4AI when the installed version supports it. The option remains false by default to match Crawl4AI and avoid changing ordinary rendered fetches.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test shadow_dom_flatten_expression_resolves_slots_and_skips_styles`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_flattens_shadow_dom_with_chrome -- --ignored`
- `cargo test`

Confidence: Medium-high. The behavior is source-backed and covered by a deterministic script-contract test plus a local Chrome smoke that verifies projected shadow DOM text reaches extraction. It remains opt-in, and broader rendered-readiness behaviors such as virtual scrolling, app-specific readiness, and full Crawl4AI readability quality remain separate I19d work.
