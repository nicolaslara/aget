# Session Wrapper Implementation Plan: Decisions Before Coding

- V1 may persist plaintext sessions as a PoC compromise, but only outside the repo, with explicit credential-equivalent warnings, `0700` directories, `0600` session/temp-state files, and a tracked follow-up for encryption at rest.
- `aget get` should write run artifacts under `~/.aget/runs/<run-id>/` by default and print a concise summary; `--json` returns the stable machine-readable shape; `--out` can write content to a caller-chosen path.
- Crawl4AI should be invoked through `uv run --with crawl4ai` for the PoC to avoid committing to a managed Python environment before the wrapper direction is validated.
- Default timeouts should start with command=60s, navigation=30s, extraction=45s, backend_startup=20s, all configurable through CLI/config.
- Token estimation should start with a simple approximate heuristic recorded as approximate; a tokenizer dependency is deferred until output-shaping behavior is otherwise stable.
