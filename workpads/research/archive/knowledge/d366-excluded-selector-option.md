# D366: Excluded Selector Option

## Decision

Owned extraction now accepts Crawl4AI's `crawl4ai.excluded_selector` backend option and removes matching elements before content extraction.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/async_configs.py` defines `CrawlerRunConfig.excluded_selector` as a CSS selector to exclude from processing and stores missing values as an empty string.
- `crawl4ai/content_scraping_strategy.py` removes `remove_forms`, `excluded_tags`, then `excluded_selector` matches from the parsed body and tolerates selector errors.

## Implementation

- `OwnedExtractorOptions` stores repeated non-empty `crawl4ai.excluded_selector` values.
- The owned cleanup pipeline removes configured backend excluded selectors after form/tag cleanup and after the existing top-level `--exclude-selector`.
- Invalid selector tolerance is preserved through the existing `remove_selected_elements` helper.
- The command/mock validation surface, README, and OpenCode tool text list the new supported option.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- Focused owned extractor selector coverage.
- Command-backend option validation coverage.
- Standard check set before commit.

## Follow-Up

- I19d remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
