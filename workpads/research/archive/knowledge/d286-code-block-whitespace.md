# D286: Code Block Whitespace

Decision: owned markdown normalization now preserves raw lines inside fenced code blocks.

Source inspection:

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` uses `CustomHTML2Text` for raw markdown with code-marking behavior enabled through the active options path.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `CustomHTML2Text` emits raw data while inside `<pre>` and ignores nested `<code>` tags there by default, preserving code-block text instead of applying inline-code formatting.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: cleaned-HTML empty-element pruning skips descendants of `<pre>` or `<code>` so whitespace-only code descendants remain significant.

Implementation boundary:

- The owned renderer still builds fenced code blocks from raw `<pre>` text.
- Markdown normalization now bypasses trailing-space trimming and blank-line collapsing while inside a fenced code block.
- Normal markdown whitespace cleanup outside code fences is unchanged.
- This is a narrow markdown fidelity slice; it does not add language detection, syntax highlighting, or broader readability filtering.

Validation:

- `cargo test aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli -- --nocapture`
- `cargo test extraction::owned::page --lib`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
