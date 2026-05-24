---
name: aget
description: Use the local aget CLI for agent-friendly URL fetching, local markdown extraction, and explicit user-authorized session replay. Use when a task needs page content through `aget`, authenticated/gated-page fetches, OAuth/session import from the user's real browser, session composition, cmux or Chrome session import, or safe local-only web context workflows.
disable-model-invocation: true
---

# aget

Use `aget` as a generic local fetcher. It returns page content and extraction outcomes; the agent decides whether returned content means login, subscription, stale auth, or a narrower selector is needed.

Prefer a release binary when available:

```bash
target/release/aget --help
```

Otherwise use the project binary through Cargo:

```bash
cargo run --quiet -- --help
```

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

### 5. Content Format And Extraction

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

For `get` and `current-tab`, artifact paths are in `data.artifacts`, selected sessions are in `data.sessions`, and extracted page content is in `data.content` only when `--inline-content` includes it. The default `--inline-content auto` omits `data.content` for session-backed/sensitive fetches and current-tab output; read the local artifact path instead, or use `--inline-content always` only when the user explicitly wants authenticated content embedded in the envelope.

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

This avoids repeated provider login while keeping the provider credential ceremony outside the agent. `aget get` enforces replay scope, so an `oauth` session for `accounts.example.com` must not be passed to `https://example.com/...` fetches unless its saved scope actually matches that request host.

`aget session login start --session <provider>` injects only explicitly named local sessions into the controlled login browser profile. The user still completes any provider prompts, passwords, passkeys, and one-time-code steps; `login finish` saves only the target relying-party session unless the user later asks to compose same-scope sessions. Provider-session injection is supported only by the owned browser backend and only with the default aget-owned login profile; omit `--profile`, and unset `AGET_AGENT_BROWSER_COMMAND` if a compatibility backend is selected. If injected sessions conflict on cookie or storage values, retry with narrower or corrected sessions instead of choosing a secret silently.

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

Before running either import command, ask the user to approve the specific local surface/profile and domains. These commands can read credential-equivalent local browser state. Prefer named Chrome profiles such as `--browser-profile Default`; explicit `--profile-path` is an advanced path and may launch that local profile directory directly, so use it only with a disposable or explicitly approved profile path.

If Chrome import returns `requires_user_action`, do not close the user's browser. Relay the message and let the user decide whether to quit Chrome and retry.

## Interpreting Results

- `ok: true` with login-wall-looking content is still a successful generic fetch. Decide next action from the content and user goal.
- `backend_unavailable` means an optional local dependency or explicitly configured compatibility backend such as Chrome, cmux, Crawl4AI, or agent-browser is missing.
- `requires_user_action` means the user must do something local, such as complete login or unlock/quit a profile.
- For extraction tuning beyond the core flows above, inspect the project README instead of inventing flags or site-specific workarounds.
