# D363: Crawl4AI `ignore_anchors` Alias

## Decision

Owned extraction now accepts `crawl4ai.ignore_anchors` as a compatibility alias for `crawl4ai.ignore_links`.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/html2text/config.py` names the default anchor-suppression flag `IGNORE_ANCHORS`.
- `crawl4ai/html2text/__init__.py` initializes runtime `ignore_links` from `config.IGNORE_ANCHORS`.
- `crawl4ai/html2text/cli.py` exposes `--ignore-links` with `default=config.IGNORE_ANCHORS`.

## Implementation

- `crawl4ai.ignore_anchors=true` maps to the existing owned `ignore_links` renderer behavior.
- The Crawl4AI compatibility helper accepts `ignore_anchors` and forwards it as runtime `ignore_links`, because `CustomHTML2Text` consumes `ignore_links`.
- Mock backend validation, unsupported-option diagnostics, README, and OpenCode option text include the alias.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo test --test mock_site_cli aget_extractor`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo fmt --check`
- `python3 -m py_compile scripts/crawl4ai_extract.py scripts/aget_crawl4ai_compat/*.py`

## Follow-Up

- I19d remains open for fuller Crawl4AI-quality markdown/readability and richer rendered-page readiness.
