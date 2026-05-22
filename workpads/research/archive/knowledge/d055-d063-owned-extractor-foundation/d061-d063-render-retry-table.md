### D61: I19d uses owned CDP rendering for localStorage-backed primary extraction

The D60 renderer exposed a follow-up correctness gap: once `OwnedExtractorBackend` becomes default, a localStorage-backed request could otherwise return a static app shell successfully and never invoke browser fallback. I19d now routes owned primary extraction through the same temporary Chrome/CDP renderer whenever composed session state contains localStorage origins. Cookie-only extraction stays on the static HTTP path.

This still does not make every JavaScript-heavy cookie-backed page render through Chrome; there is no reliable generic signal for that yet. The new rule only covers the explicit structured-state case where static HTTP cannot replay localStorage at all.

Validation:

- `cargo test --test mock_site_cli owned_extractor_backend_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_browser_fallback_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The local Chrome smoke test proves rendered DOM extraction for the primary owned backend on this machine; broader rendered-JavaScript default policy remains an open I19d/I19f decision.

### D62: I19d retries owned extraction through CDP when CSS waits need rendered DOM

I19d now covers a second narrow rendered-JavaScript case without adding public API: when the owned static HTTP path cannot find a requested CSS `--wait-for-selector`, it retries the same extraction through the owned Chrome/CDP renderer. This mirrors the current Crawl4AI contract for CSS waits while preserving the existing safety boundary: JavaScript wait expressions are still rejected, and the only user input evaluated in Chrome is a JSON-quoted CSS selector passed to `document.querySelector(...)`.

This is deliberately not a blanket browser-rendering default. Static pages with matching selectors still stay on the faster HTTP path, and JavaScript-heavy pages without a wait selector remain a future policy/default decision for I19f.

Validation:

- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The ignored Chrome smoke test proves the delayed-DOM wait path locally, and the normal suite keeps static wait-selector behavior covered without requiring Chrome.

### D63: I19d adds first owned markdown table rendering

Before this slice, the owned markdown renderer collapsed HTML tables into plain text. Crawl4AI's markdown behavior and tests treat tables as part of the markdown-quality target, so the owned renderer now emits GitHub-flavored markdown tables for static table structures. Header rows are detected from `<th>` cells; tables without explicit headers use the first row as the markdown header. Cell content goes through the same inline renderer as normal text, so links are still resolved against the page base URL, and pipe characters inside cells are escaped.

This is not a full markdown/readability replacement yet. Remaining quality gaps still include captions, complex row/column spans, Crawl4AI-style citations/references, fit markdown, and broader main-content cleanup.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The static parity test now covers a table with links and literal pipe characters, but complex table semantics remain an explicit follow-up.

