# D317: Crawl4AI Adapter Script Split

Date: 2026-05-22

## Decision

Split `scripts/crawl4ai_extract.py` into smaller behavior-owned helper modules while preserving the existing executable compatibility adapter entrypoint.

## Boundary

- `scripts/crawl4ai_extract.py` remains the executable route and owns argument parsing, Crawl4AI imports/config construction, crawler execution, response shaping, and process exit status.
- `scripts/aget_crawl4ai_compat/options.py` owns Crawl4AI backend-option allowlisting, typed option parsing, CSS-only wait validation, and installed-Crawl4AI signature compatibility checks.
- `scripts/aget_crawl4ai_compat/content.py` owns result content selection for markdown/html/text/json and the fallback HTML-to-text parser.
- `scripts/aget_crawl4ai_compat/files.py` owns private `0600` text writes for output and metadata artifacts.
- `scripts/aget_crawl4ai_compat/__init__.py` marks the helper package.

## Safety And Compatibility Notes

- This is a mechanical adapter decomposition only; command-line arguments, JSON response shape, metadata writes, validation messages, and Crawl4AI invocation behavior are intended to remain unchanged.
- The script still imports helper modules from the adjacent `scripts/` directory, so existing `python scripts/crawl4ai_extract.py ...` and README `uv run ... python scripts/crawl4ai_extract.py` usage continue to work.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The executable script route dropped from 314 lines to 132 lines; helper modules are currently 122 lines or less.

## Validation

- `python3 -m py_compile scripts/crawl4ai_extract.py scripts/aget_crawl4ai_compat/*.py`
- `python3 scripts/crawl4ai_extract.py ... --extractor-option crawl4ai.magic=value`
- `python3 scripts/crawl4ai_extract.py ... --wait-for 'js:return document.body'`
- `cargo test --test get_cli validation`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
