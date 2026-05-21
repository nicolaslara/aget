# D204: AgetExtractor Parity Test Helper Split

Date: 2026-05-21

Decision:

- Keep `tests/mock_site_cli.rs` and the public `aget_extractor_backend_covers_static_http_parity_slice` integration test as the stable coverage entrypoint.
- Split the large `tests/mock_site_cli/aget_extractor.rs` test body into behavior-focused helper modules under `tests/mock_site_cli/aget_extractor/`.
- Preserve all existing fixture routes and assertions; this is a mechanical test-local decomposition, not a behavior change.

Resulting helper modules:

- `basic_formats.rs`: public fetch, session replay, text/html/json format basics.
- `markdown.rs`: structural markdown rendering parity.
- `cleanup.rs`: cleaned HTML and cleanup markdown behavior.
- `main_content.rs`: default main-content and overlay scoring behavior.
- `selectors.rs`: selector miss/invalid/multiple and target/excluded tag options.
- `options_waits.rs`: backend option parsing, redirect final URL, and wait selector behavior.

Boundaries:

- `workpads/research/tasks.md` remains the detailed executable backlog and was not compacted.
- The split keeps the single test setup so coverage still exercises one shared mock site and saved session, matching the previous integration shape.

Validation:

- `cargo fmt --check`
- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`
