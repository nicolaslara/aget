# TinyFish-Inspired Future Features

Date: 2026-05-26

## Context

TinyFish is a hosted web-agent platform with separate Search, Fetch, Browser, and Agent surfaces. `aget` is intentionally different: it is local-first, CLI-first, and built around explicit user-approved local sessions. Still, TinyFish's product shape suggests useful future features and documentation patterns for `aget`.

This document records ideas to consider later. It is not a commitment to add hosted services, stealth/anti-bot behavior, proxy routing, MCP, or password-vault sync.

## Useful Ideas To Mimic

### 1. Clear Capability Split

TinyFish separates its web stack into distinct surfaces: known-URL fetch, search, browser control, and goal-driven agent automation.

`aget` already has a local equivalent split:

- `aget get`: known URL to agent-ready content.
- `aget batch`: many known URLs.
- `aget map`: discover links from a page or artifact.
- `aget crawl`: bounded same-origin/path traversal.
- `aget current-tab`: explicitly approved local browser tab extraction.
- `aget session`: local auth/session state.

Future idea: document this as the core decision tree, and only add a future `interact` or `agent` command if local multi-step browser workflows become a real need.

### 2. Agent-Facing Routing Docs

TinyFish has docs aimed directly at coding agents, plus an `llms.txt` index.

Future idea:

- Add repo-root `llms.txt`.
- Add `docs/for-agents.md`.
- Include a compact decision tree for `get`, `batch`, `map`, `crawl`, `session`, and `current-tab`.
- Keep private-content and session-safety guidance explicit.

### 3. Run Lifecycle Language

TinyFish exposes run lifecycle states such as pending, running, completed, failed, and cancelled.

`aget` already stores run artifacts under `AGET_HOME/runs`, and has artifact lifecycle commands. Future idea:

- Decide whether `artifacts` should stay the public noun or whether `runs` should become an alias or replacement.
- If keeping `artifacts`, document that artifacts are the persisted record of local runs.
- Consider run status metadata for long-running `batch` and `crawl` operations.

### 4. Partial Failure Shape

TinyFish Fetch treats multi-URL failures as per-URL errors alongside successful results.

`aget batch` and `crawl` should continue leaning into this:

- deterministic manifest files;
- per-item success, skipped, and error states;
- stable non-zero exit behavior when supported inputs fail;
- machine-readable JSON envelopes for agents.

Future idea: make the manifest schema a documented contract.

### 5. Streaming Or Progress Events

TinyFish has sync, async, and SSE-style automation modes.

`aget` should not add a hosted server just to copy this, but a local CLI equivalent may be useful:

- `--events jsonl` for machine-readable progress;
- `--progress jsonl` for `batch` and `crawl`;
- stable event names such as `started`, `item_started`, `item_completed`, `item_failed`, `complete`.

This would help agents monitor long local crawls without parsing human output.

### 6. Stable Structured Extraction

TinyFish Fetch distinguishes output formats and presents JSON as a structured document tree.

`aget` already supports `--content-format json`, but future work should decide whether that output is a stable public document-tree contract.

Future idea:

- Define a versioned `aget.document.v1` JSON tree.
- Include headings, paragraphs, lists, tables, links, images, code blocks, and metadata.
- Keep markdown as the default agent-context format.

### 7. Local Browser Presets

TinyFish has browser profile choices. Some of their positioning is hosted/anti-bot-specific and does not fit `aget`, but the preset concept is useful.

Future local presets could configure extraction/browser behavior without anti-bot claims:

- `default`: current balanced behavior.
- `rendered`: stronger JavaScript/rendered-page settings.
- `low-resource`: fewer images, lower timeouts, less scrolling.
- `debug`: richer artifacts and diagnostics.

Avoid stealth/proxy/bypass positioning in core `aget`.

### 8. Local Credential And Session Model

TinyFish has hosted vault credentials. `aget` should not copy that model as a service.

The local-first equivalent is:

- explicit named provider sessions such as `oauth`, `google`, `github`, or `okta`;
- `aget session login start target --session oauth` for provider-session injection;
- explicit browser import only after user approval;
- clear session retention and deletion through `aget session inspect/delete`.

Future idea: improve docs and UX around provider-session setup and cleanup.

## Ideas Not To Mimic Directly

Do not add these unless the project direction changes explicitly:

- hosted browser fleet;
- proxy/geographic routing as a bypass feature;
- anti-bot or stealth positioning;
- password-manager vault sync to a service;
- MCP/server layer as part of the current CLI-first productization track;
- full goal-driven remote agent automation before local CLI workflows prove a need.

## Candidate Backlog Tasks

- `DOC-AGENT-001`: Add `llms.txt` and `docs/for-agents.md`.
- `RUNS-001`: Decide whether `artifacts` should become or alias `runs`.
- `EVENTS-001`: Add `--events jsonl` or `--progress jsonl` for batch/crawl progress.
- `JSON-001`: Define stable `--content-format json` document-tree output.
- `PRESETS-001`: Define local extraction/browser presets without anti-bot claims.
- `SESSION-UX-001`: Improve provider-session setup, inspection, cleanup, and examples.
- `INSTALL-001`: Continue release/install polish, including GitHub Release automation and install docs.

## Source Notes

Reviewed TinyFish public docs on 2026-05-26:

- https://docs.tinyfish.ai/
- https://docs.tinyfish.ai/fetch-api
- https://docs.tinyfish.ai/key-concepts/endpoints
- https://docs.tinyfish.ai/key-concepts/runs
- https://docs.tinyfish.ai/key-concepts/browser-profiles
- https://docs.tinyfish.ai/for-coding-agents
- https://docs.tinyfish.ai/vault-setup
- https://docs.tinyfish.ai/llms.txt
