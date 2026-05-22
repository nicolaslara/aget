# D285: Agent-Facing Option Documentation

Decision: the OpenCode `aget` tool description now mirrors the owned Crawl4AI compatibility option surface exposed by the Rust extractor.

Source inspection:

- `src/extraction/owned/options.rs`: owned parser accepts `crawl4ai.base_url`, `crawl4ai.exclude_internal_links`, `crawl4ai.process_iframes`, and the rest of the current compatibility option set.
- `README.md`: CLI documentation already lists the current owned `AgetExtractor` options.
- `.opencode/tools/aget.ts`: the `backend_options` schema description was missing `crawl4ai.base_url`, `crawl4ai.exclude_internal_links`, and `crawl4ai.process_iframes`.
- `.cursor/skills/aget/SKILL.md`: the project skill routes extraction tuning to the README instead of duplicating the backend option list.
- `tests/support/bin/aget_mock_backend/config.rs`: mock backend validation allowlist already includes the current option set.

Implementation boundary:

- Updated only the OpenCode tool schema description for `backend_options`.
- Kept README unchanged because it already listed the current owned option set.
- Kept the project skill unchanged because it deliberately delegates detailed extraction tuning to README.
- This is an agent-integration documentation alignment slice; it does not change runtime option parsing or extraction behavior.

Validation:

- `rg -n "crawl4ai\\.base_url|crawl4ai\\.exclude_internal_links|crawl4ai\\.process_iframes" .opencode/tools/aget.ts README.md src/extraction/owned/options.rs tests/support/bin/aget_mock_backend/config.rs`
- `rg -n "AgetExtractor currently supports|extraction tuning beyond" .opencode/tools/aget.ts README.md .cursor/skills/aget/SKILL.md`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
