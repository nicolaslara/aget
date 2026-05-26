---
name: aget
description: Use the local aget CLI for agent-friendly URL fetching, local markdown extraction, and explicit user-authorized session replay. Use when a task needs page content through `aget`, authenticated/gated-page fetches, OAuth/session import from the user's real browser, session composition, cmux or Chrome session import, or safe local-only web context workflows.
disable-model-invocation: true
---

# aget

Use `aget` as a generic local fetcher. It returns page content and extraction outcomes; the agent decides whether returned content means login, subscription, stale auth, or a narrower selector is needed.

Prefer an installed release or source binary on `PATH`:

```bash
aget --help
```

For unreleased checkout work, use the project binary through Cargo:

```bash
cargo run --quiet -- --help
```

## Global Skill Install

Install this skill into Codex from a checkout:

```bash
scripts/install-codex-skill.sh --symlink --force
```

Install from an unpacked release tarball:

```bash
CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
mkdir -p "$CODEX_HOME/skills"
rm -rf "$CODEX_HOME/skills/aget"
cp -R aget-v0.1.0-aarch64-apple-darwin/skills/aget "$CODEX_HOME/skills/aget"
```

The helper defaults to copying the skill. Use `--symlink` from a development
checkout when local skill edits should be reflected after restarting Codex.
Restart Codex after installing or replacing the global skill.

## Safety Rules

- Only process content the user is authorized to access.
- Never collect, type, script, store, or ask the user to reveal credentials.
- Do not use ambient browser auth silently. Authenticated fetches require an explicit named session.
- Always ask before importing/copying session state from the user's real browser, including the browser/profile or surface and the allowed domains.
- Do not use `current-tab` unless the user explicitly approves reading the selected local browser tab and provides or approves the CDP port.
- For OAuth-backed sites, prefer a reusable provider session such as `oauth` when available, so login attempts can reuse provider cookies without exposing credentials to the agent.
- Treat session files, storage-state temp files, screenshots, authenticated markdown, and envelope content as private local data.
- Prefer `--output <path>` for large or sensitive content so the agent can read only the needed artifact.

## Decision Workflow

Before running `aget`, choose these deliberately.

### 0. Command Choice

- **One known URL**: use `aget get`.
- **Several known URLs**: use `aget batch` with an explicit finite URL list or
  newline-delimited file. This is for comparison/fetch fan-out, not discovery.
- **Need candidate links from one page**: use `aget map`. Prefer
  `aget map --artifact <run-id>` when a page was already fetched, especially
  for private or large content.
- **Need a bounded section of a site**: use `aget crawl <url> --limit <n>`.
  Keep same-origin/same-path defaults unless the user explicitly approves a
  wider scope.
- **Need relevant sections inside one fetched page**: use
  `aget search-page --artifact <run-id> --query <text>` after fetching the page.
  Use `--allow-private-content` only when the user has approved snippets from a
  sensitive artifact.
- **Need structured data from existing artifacts**: use `aget extract` with
  `--artifact <run-id>` or `--manifest <path>`. Prefer built-in `--field`
  values for headings, links, tables, definitions, and metadata; use a schema
  file for HTML selectors or JSON paths.
- **Need debugging provenance without page content**: add `--capture-trace` and
  inspect the local `debug-trace.json` path. The trace records options,
  warnings, timing, cache, usage, and errors; it does not include extracted page
  content.
- **Need visual evidence for a browser-rendered page**: add
  `--capture-screenshot` only after the user approves capturing the page
  visually. Screenshots are local run artifacts and may contain private content.
- **Need prior run metadata or local files**: use `aget artifacts list` and
  `aget artifacts inspect <run-id>` before reading large or sensitive files.
- **Tool or environment looks broken**: use `aget doctor --quick` before
  changing sessions or browser setup.

For sensitive pages, prefer an artifact-first flow: fetch with `--output` or
default run artifacts, inspect metadata with `artifacts inspect`, then read only
the local artifact paths needed for the task. Do not widen map/crawl scope on
authenticated content without explicit user approval.

### 1. Auth Mode

- **Public/unknown page**: start without `--session`.
- **User named a session**: use that exact `--session <name>`.
- **Page appears private after a public fetch**: explain the observed content and ask whether to use/create a local session.
- **Existing local auth preferred**: ask for approval, then import the explicitly approved browser profile or cmux surface with scoped `--allow-domain`.
- **OAuth/private content login needed**: if a provider session such as `oauth` exists, inject it into `session login start` for the target login flow. Do not pass provider sessions to unrelated target-site fetches; replay scope intentionally rejects sessions outside the request host. If no provider session exists, ask whether the user wants to create/import one.

Do not infer access state from command success alone. `aget` is a generic fetcher: `ok: true` means extraction of the returned page succeeded, not that the user reached the intended content. A login wall, private-resource placeholder, subscription prompt, empty app shell, or generic error page can all be successful extractions. The caller agent must inspect content against the user goal or use caller-supplied verification predicates where available.

### 2. Output Strategy

- **Small, non-sensitive answer needed in chat**: no `--output` is acceptable.
- **Large page, private page, or follow-up analysis expected**: use `--output /tmp/...` or `/private/tmp/...` and inspect the file selectively.
- **Authenticated/session-backed content**: prefer `--output`; avoid embedding private page content in tool envelopes.
- **Exact content URL known**: prefer the direct document/file URL over a landing page for cleaner extraction.

### 3. Envelope Strategy

- **Human/debug exploration**: plain output is fine.
- **Agent/tool control flow**: use `--envelope json`.
- **Sensitive session-backed output**: keep `--inline-content auto` or set `--inline-content never`; read artifact paths instead.
- **Only embed content intentionally**: use `--inline-content always` only when the user explicitly wants the content in the JSON response.

### 4. Content Size

- **Need complete artifact**: omit `--max-chars`.
- **Need bounded context for quick inspection**: use `--max-chars`, commonly `4000`, `12000`, or another task-appropriate limit.
- **When writing to a file for later analysis**: avoid `--max-chars` unless the user asked for a sample or bounded extract.

### 5. Cache And Freshness

- **Stable public HTTP(S) page**: keep default cache behavior and inspect `data.cache.status` plus `data.usage`.
- **Pricing, changelog, time-sensitive docs, or suspected stale result**: use `--fresh` or `--cache-policy refresh`.
- **Do not want reuse for a public fetch**: use `--cache-policy off`.
- **Need a shorter or longer freshness window**: use `--cache-ttl <seconds>`.
- **Session-backed, current-tab, raw, and file inputs**: expect cache status `ineligible` or `disabled`; do not try to force reusable cache entries for private content.

Cache hits still create fresh run artifacts. Cache entries store untruncated extracted content, so a later run can apply its own `--max-chars`.

### 6. Content Format And Extraction

- Default to `--content-format markdown`.
- Use `--content-format text` for grep-like text extraction.
- Use `--content-format html` for debugging extractor output or selectors.
- Use `--selector`, `--exclude-selector`, and `--wait-for-selector` when the user needs a section, app content, or rendered page readiness.
- Use `--backend-option` only for known supported options; do not invent site-specific workarounds.

## Structured Output

Use `--envelope json` for stable agent/tool output.

Success shape:

```json
{"ok": true, "schema_version": "aget.envelope.v1", "command": "get", "data": {}, "warnings": [], "timing_ms": {"total": 0}}
```

Error shape:

```json
{"ok": false, "schema_version": "aget.envelope.v1", "command": "get", "error": {"code": "requires_user_action", "message": "..."}}
```

For `get` and `current-tab`, artifact paths are in `data.artifacts`, selected sessions are in `data.sessions`, cache state is in `data.cache`, rough byte/token accounting is in `data.usage`, and extracted page content is in `data.content` only when `--inline-content` includes it. The default `--inline-content auto` omits `data.content` for session-backed/sensitive fetches and current-tab output; read the local artifact path instead, or use `--inline-content always` only when the user explicitly wants authenticated content embedded in the envelope.

Debug artifacts appear under `data.artifacts.debug` only when explicitly
requested. Prefer `--capture-trace` before `--capture-screenshot`; traces avoid
page content, while screenshots can expose visible private data.

For `search-page` and `extract`, outputs come from existing local artifacts.
Sensitive source artifacts require `--allow-private-content` before snippets or
structured values are emitted. Use `extract --field tables --field links` for
common deterministic fields, or `extract --schema schema.json` for selector and
JSON-path fields. `extract --artifact` reads internal run content only; it
refuses caller-owned external `--output` paths.

## Basic Fetch

Use an empty session first unless the user already chose a named session:

```bash
aget --envelope json get "https://example.com/docs" --content-format markdown
```

For long output:

```bash
aget --envelope json get "https://example.com/docs" --content-format markdown --output /tmp/aget-page.md --max-chars 12000
```

For a release binary:

```bash
target/release/aget get "https://example.com/docs" --output /tmp/aget-page.md
```

## Current Tab

Use current-tab only after explicit user approval, because it can read authenticated/private content already open in the local browser. The user must provide or approve the Chrome DevTools debugging port:

```bash
aget --envelope json current-tab --cdp-port 9222 --allow-private-content --output /tmp/current-tab.md
```

Do not scan ports or profiles. Do not use `--inline-content always` unless the user explicitly wants the selected tab content embedded in the envelope.

## Gated Page Flow

Example starting point for a user-authorized gated page:

```bash
aget --envelope json get "https://example.com/private/page" --content-format markdown --output /tmp/aget-check.md
```

If the returned content looks like a login/subscription wall, tell the user what you observed and ask whether they want to create a local session. If they agree:

```bash
aget --envelope json session import browser --browser chrome --browser-profile Default --name target --allow-domain example.com
```

Then verify the imported session unlocks the page:

```bash
aget --envelope json get "https://example.com/private/page" --session target --content-format markdown --output /tmp/aget-auth.md
```

Generalize the same pattern to any user-authorized gated site, private docs, dashboards, or account pages. Session names are caller-chosen labels, not built-in site handlers. For OAuth-backed login flows, prefer a reusable provider session such as `oauth` when the user has one:

```bash
aget --envelope json session login start target --url "https://example.com/login" --session oauth
aget --envelope json session login finish target
aget --envelope json get "https://example.com/account" --session target --output /tmp/aget-account.md
```

If import returns `requires_user_action` because the profile is locked, ask the user to quit the relevant browser/profile and retry. If import succeeds but the follow-up fetch still shows a login wall, explain that the existing browser profile is not logged in for the target site; ask the user to sign in through their normal browser, then import again.

## OAuth Provider Sessions

For OAuth-backed workflows, a useful pattern is to keep a reusable provider session named `oauth` or a provider-specific name such as `google`, `github`, or `okta`. This session should contain only user-authorized provider cookies/storage imported or created through `aget`; the agent never sees or handles passwords, passkeys, or one-time codes.

Use a generic `oauth` session when the user wants one bucket for shared login-provider state:

```bash
aget --envelope json session import browser --browser chrome --browser-profile Default --name oauth --allow-domain accounts.example.com
```

Use a provider-specific name when the distinction matters:

```bash
aget --envelope json session import browser --browser chrome --browser-profile Default --name google --allow-domain accounts.google.com
```

Inject provider sessions only into the login browser profile for the relying-party flow:

```bash
aget --envelope json session login start target --url "https://example.com/login" --session oauth
aget --envelope json session login finish target
aget --envelope json get "https://example.com/account" --session target --output /tmp/aget-account.md
```

This avoids repeated provider login while keeping the provider credential ceremony outside the agent. Provider sessions are bootstrap inputs for `session login start`, not fetch credentials for target content. `aget get` enforces replay scope as a guardrail, but scope acceptance is not permission to use provider credentials as target-site authorization.

`aget session login start --session <provider>` injects only explicitly named local sessions into the controlled login browser profile. The user still completes any provider prompts, passwords, passkeys, and one-time-code steps; `login finish` saves only the target relying-party session unless the user later asks to compose same-scope sessions. Provider-session injection uses the current Chrome/CDP login path and the default aget-owned login profile; omit `--profile` when injecting sessions. Fetch target content with the session saved by `login finish`, not with the provider session. If injected sessions conflict on cookie or storage values, retry with narrower or corrected sessions instead of choosing a secret silently.

## Access Verification

`aget` does not currently have a universal "needs login" detector. Keep that responsibility in the caller agent.

Use one of these verification styles:

- **Content review**: fetch the page, read a small bounded sample or artifact, and compare it to the user's requested content.
- **Positive predicate**: when using `session authorize`, pass `--must-contain <text>` for generic text that should appear only when the intended page is reached.
- **Negative predicate**: pass `--must-not-contain <text>` for generic placeholders such as a known login prompt when that is safe and non-site-specific enough for the task.
- **Artifact-first review**: for sensitive pages, write to `--output` and inspect only enough of the artifact to decide whether auth worked.

Example unknown-access flow:

```bash
aget get "https://example.com/private/page" --output /tmp/aget-check.md --max-chars 4000
```

If the content does not match the requested page, ask the user whether to create or use a local session. Import and verify with a caller-chosen session name and domain scope:

```bash
aget session authorize target \
  --url "https://example.com/private/page" \
  --browser chrome \
  --browser-profile Default \
  --allow-domain example.com \
  --must-contain "expected page text" \
  --output /tmp/aget-auth-check.md
```

If import exports no usable auth state or verification still shows the wrong content, ask the user whether to complete a controlled login flow:

```bash
aget session login start target --url "https://example.com/private/page" --session oauth
# user completes login in the opened browser
aget session login finish target
aget get "https://example.com/private/page" --session target --output /tmp/aget-private.md
```

Omit `--session oauth` when the user has not explicitly chosen a provider session to inject.

Avoid encoding site-specific login-wall strings into the skill or binary. Site-specific interpretation belongs in the caller's task context.

## Combining Sessions

Some sites require provider cookies during login but not later fetches. Inject provider sessions during login; combine sessions during fetch only when every selected session's saved scope matches the request host.

During fetch with same-scope sessions:

```bash
aget --envelope json get "https://docs.example.com/account" --session docs-base --session docs-extra --output /tmp/account.md
```

Persist a reusable composition:

```bash
aget --envelope json session compose target --session docs-base --session docs-extra
aget --envelope json get "https://docs.example.com/account" --session target --output /tmp/account.md
```

If session composition reports a conflict, do not guess which secret wins. Ask the user which session should be replaced, deleted, or retried.

## Combining Sessions While Logging In

`aget session login start` can inject one or more existing local sessions into the login browser profile:

```bash
aget --envelope json session login start target --url "https://example.com/login" --session oauth --session okta
```

Use this only for sessions the user explicitly named. Do not combine this with `--profile`; injection uses the default aget-owned login profile so provider state can be cleaned up after finish or cancel. Injection does not broaden later replay scope: the target session saved by `login finish` remains scoped to the login URL's allowed domains, and provider sessions remain separate unless the user later runs same-scope `aget session compose`.

## Importing Existing Local Auth

cmux cookie import is explicit and domain-scoped:

```bash
aget --envelope json session import cmux --surface "surface:1" --name target --allow-domain docs.example.com
```

Chrome import uses `aget`'s owned local Chrome/CDP import path:

```bash
aget --envelope json session import browser --browser chrome --browser-profile Default --name target --allow-domain docs.example.com
```

Before running either import command, ask the user to approve the specific local surface/profile and domains. These commands can read credential-equivalent local browser state. Prefer named Chrome profiles such as `--browser-profile Default`; explicit `--profile-path` is an advanced path and may launch that local profile directory directly, so use it only with a disposable or explicitly approved profile path. Import stores only scoped exported cookies/storage in the named `aget` session; it does not copy, retain, or manage the whole source browser profile. For login flows, the default aget-owned temporary profile is cleaned up after finish or cancel; a caller-provided custom profile path is caller-owned and is not deleted or pruned by `aget`.

If Chrome import returns `requires_user_action`, do not close the user's browser. Relay the message and let the user decide whether to quit Chrome and retry.

## Interpreting Results

- `ok: true` with login-wall-looking content is still a successful generic fetch. Decide next action from the content and user goal.
- `backend_unavailable` means a required local component for the selected command is missing or unavailable, such as Chrome/CDP for browser-backed flows or cmux for `session import cmux`.
- `requires_user_action` means the user must do something local, such as complete login or unlock/quit a profile.
- For extraction tuning beyond the core flows above, inspect the project README instead of inventing flags or site-specific workarounds.
