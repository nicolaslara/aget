# Changelog

## 0.1.0 - 2026-05-24

- Added the owned `aget get`, `aget current-tab`, `aget session`, and `aget doctor`
  CLI surfaces with JSON envelopes for agent/tool callers.
- Added `aget artifacts list/inspect/delete/prune` for local run-artifact
  lifecycle management.
- Added `aget batch` for bounded concurrent URL-list fetches with per-URL
  artifacts, deterministic duplicate skips, and partial-failure manifests.
- Added `aget map` for one-page link discovery from a URL or successful
  internal artifact run without recursively fetching discovered links.
- Added `aget crawl` for bounded same-origin/path traversal with required
  limits, per-page artifacts, and partial-failure manifests.
- Added `aget search-page` and `aget extract` for deterministic artifact-first
  narrowing and structured extraction.
- Added local owned extraction and Chrome/CDP-backed browser/session handling.
- Added explicit session replay scope checks, sensitive-output handling, and
  run artifacts under `AGET_HOME`.
- Removed active Crawl4AI and `agent-browser` runtime dependency surfaces; they
  remain only as historical/parity references.
- Added release planning, parity coverage, and post-migration workpad routing.
