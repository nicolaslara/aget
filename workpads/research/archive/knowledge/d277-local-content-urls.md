# D277: Crawl4AI Local-Content URLs

Decision: the owned static extractor supports explicit `raw:`, `raw://`, and `file://` inputs for Crawl4AI compatibility.

Source inspection:

- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py` documents `http://`, `https://`, `file://`, and `raw://` crawl inputs.
- For local content, Crawl4AI strips `file://`, `raw://`, or `raw:`, reads the local file or raw HTML, returns status `200`, and only routes through the browser path when browser-specific options require it.
- Crawl4AI strips raw prefixes directly instead of URL-parsing raw content because raw HTML/CSS may contain `#`.

Implementation boundary:

- `owned_fetch` now recognizes `raw:`, `raw://`, and `file://` before HTTP URL parsing.
- Local-content responses are marked non-renderable, so HTTP script auto-render and rendered wait retry stay limited to HTTP(S) network fetches.
- Cookie headers are only composed for HTTP(S) requests. Session replay still requires a valid HTTP(S)-scoped request host, so named sessions are rejected for local-content inputs before extraction.
- `file://` uses the same explicit-path stripping boundary as Crawl4AI; no implicit profile/session state is applied.

Validation:

- `cargo test owned_fetch`
- `cargo test parses_get_local_content_input`
- `cargo test replay_scope_rejects_session_state_for_local_content_inputs`
- `cargo test --test mock_site_cli aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
