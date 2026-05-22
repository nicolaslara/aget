# Session Wrapper Implementation Plan: Objective, Stack, And Repository Shape

## Objective

Build the v1 PoC as a thin wrapper that proves the `aget` product model:

- Empty session by default.
- Explicit named sessions.
- Per-request session composition via repeated `--session` flags.
- Optional cmux import for users in cmux.
- Chrome/profile import through `agent-browser` for users without cmux.
- Crawl4AI-backed local extraction using temporary Playwright storage state.
- Non-interactive, timeout-bounded, agent-safe command behavior.

## Implementation Principle

The first implementation should optimize for correctness and testability over completeness. The smallest useful product slice is:

```bash
aget <url>
aget get <url>
aget session import cmux --surface <surface> --name <name> --domain <domain>
aget get <url> --session <name>
aget session inspect <name>
```

Chrome import, session composition, and additional extractor options should follow once the local session store and Crawl4AI replay path are tested.

## Proposed Stack

Use Rust for the CLI and session/state logic.

Use external tools for v1 backend work:

- cmux CLI for optional browser-surface cookie import.
- `agent-browser` CLI for Chrome profile/session state acquisition.
- Python/Crawl4AI helper for rendered extraction.

Rust should own:

- CLI parsing.
- Session file format.
- Cookie/storage filtering.
- Session composition.
- Temporary state creation/deletion.
- Command timeout/process orchestration.
- Output metadata.
- Testable pure logic.

## Repository Shape

Expected first implementation layout:

```text
Cargo.toml
src/
  main.rs
  cli.rs
  commands/
    get.rs
    session.rs
  session/
    mod.rs
    store.rs
    model.rs
    compose.rs
    filter.rs
    playwright.rs
  backends/
    mod.rs
    crawl4ai.rs
    cmux.rs
    agent_browser.rs
  output/
    mod.rs
    metadata.rs
  process.rs
  error.rs
scripts/
  crawl4ai_extract.py
tests/
  fixtures/
  integration/
```

The exact names can change, but keep pure session logic separate from shelling out to backends.
