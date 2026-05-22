# D358: Preserve Tags Markdown Option

## Decision

Owned markdown extraction supports `crawl4ai.preserve_tags` for callers that need specific HTML subtrees preserved as raw HTML instead of converted to markdown.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`, `crawl4ai/html2text/__init__.py`.
- `CustomHTML2Text.update_params` accepts `preserve_tags` and stores it as a set.
- `CustomHTML2Text.handle_tag` checks preserved tags before normal tag conversion; it collects nested tag/text content and emits the preserved HTML block when the preserved tag closes.

## Implementation

- `crawl4ai.preserve_tags` is parsed as a comma-separated list of plain HTML tag names.
- Owned markdown rendering emits configured preserved tag subtrees as raw HTML blocks.
- Default behavior is unchanged when the option is absent.
- The Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text include the new option.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo fmt --check`
- `cargo test aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_forwards_supported_output_options_and_records_limits --test get_cli`

## Follow-Up

- I19d remains open for fuller Crawl4AI-quality markdown/readability and richer rendered-page readiness.
