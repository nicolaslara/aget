---
name: aget
description: Use the local aget CLI for agent-friendly URL fetching, local markdown extraction, and explicit user-authorized session replay. Use when a task needs page content through `aget`, authenticated/gated-page fetches, OAuth/session import from the user's real browser, session composition, cmux or Chrome session import, or safe local-only web context workflows.
disable-model-invocation: true
---

# aget

Use `aget` as a generic local fetcher. It returns page content and extraction outcomes; the agent decides whether returned content means login, subscription, stale auth, or a narrower selector is needed.

## Safety Rules

- Only process content the user is authorized to access.
- Never collect, type, script, store, or ask the user to reveal credentials.
- Do not try to bypass paywalls, access controls, anti-bot systems, or site policy.
- Do not use ambient browser auth silently. Authenticated fetches require an explicit named session.
- Do not use `current-tab` unless the user explicitly approves reading the selected local browser tab and provides or approves the CDP port.
- For OAuth-backed sites, prefer importing a user-authorized real browser session over opening an automation-controlled login browser.
- Treat session files, storage-state temp files, screenshots, authenticated markdown, and envelope content as private local data.
- Prefer `--output <path>` for large or sensitive content so the agent can read only the needed artifact.

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

## Current Tab

Use current-tab only after explicit user approval, because it can read authenticated/private content already open in the local browser. The user must provide or approve the Chrome DevTools debugging port:

```bash
aget --envelope json current-tab --cdp-port 9222 --allow-private-content --output /tmp/current-tab.md
```

Do not scan ports or profiles. Do not use `--inline-content always` unless the user explicitly wants the selected tab content embedded in the envelope.

## Gated Page Flow

Example starting point with HelloInterview as a user-authorized representative gated site:

```bash
aget --envelope json get "https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals" --content-format markdown --output /tmp/hi.md
```

If the returned content looks like a login/subscription wall, tell the user what you observed and ask whether they want to create a local session. If they agree:

```bash
aget --envelope json session import browser --browser chrome --browser-profile Default --name hellointerview --allow-domain hellointerview.com --allow-domain www.hellointerview.com
```

Then verify the imported session unlocks the page:

```bash
aget --envelope json get "https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals" --session hellointerview --content-format markdown --output /tmp/hi-auth.md
```

Generalize the same pattern to any user-authorized gated site, such as `ft.com`, `nytimes.com`, private docs, dashboards, or account pages. Session names are caller-chosen labels, not built-in site handlers. For OAuth-backed sites, ask the user whether they are already logged in through a real browser and prefer importing that browser profile:

```bash
aget --envelope json session import browser --browser chrome --browser-profile Default --name news --allow-domain nytimes.com --allow-domain www.nytimes.com
aget --envelope json get "https://www.nytimes.com/account" --session news --output /tmp/news-account.md
```

If import returns `requires_user_action` because the profile is locked, ask the user to quit the relevant browser/profile and retry. If import succeeds but the follow-up fetch still shows a login wall, explain that the existing browser profile is not logged in for the target site; ask the user to sign in through their normal browser, then import again.

## Combining Sessions

Some sites require provider cookies during login but not later fetches. Keep provider/app sessions explicit.

During fetch:

```bash
aget --envelope json get "https://docs.example.com/account" --session provider --session app --output /tmp/account.md
```

Persist a reusable composition:

```bash
aget --envelope json session compose workdocs --session provider --session app
aget --envelope json get "https://docs.example.com/account" --session workdocs --output /tmp/account.md
```

If session composition reports a conflict, do not guess which secret wins. Ask the user which session should be replaced, deleted, or retried.

## Combining Sessions While Logging In

`aget session login start` does not currently accept `--session`. Do not pretend provider/app session composition is built into the login command.

`aget session login start` is a fallback for non-OAuth or controlled test flows. It opens an AgetBrowser-managed profile and may be rejected by OAuth providers. Do not use it as the first authenticated path for OAuth-backed sites when a real browser session can be imported.

If a login flow appears to require provider cookies beyond the relying-party session:

- Explain the limitation to the user.
- Ask whether they want to import or create each required session explicitly.
- Use request-time composition or `aget session compose` after those sessions exist.
- Prefer the narrowest relying-party session for later fetches if it works without provider cookies.

## Importing Existing Local Auth

cmux cookie import is explicit and domain-scoped:

```bash
aget --envelope json session import cmux --surface "surface:1" --name workdocs --allow-domain docs.example.com
```

Chrome import uses `aget`'s owned local Chrome/CDP import path:

```bash
aget --envelope json session import browser --browser chrome --browser-profile Default --name workdocs --allow-domain docs.example.com
```

Before running either import command, ask the user to approve the specific local surface/profile and domains. These commands can read credential-equivalent local browser state.

If Chrome import returns `requires_user_action`, do not close the user's browser. Relay the message and let the user decide whether to quit Chrome and retry.

## Interpreting Results

- `ok: true` with login-wall-looking content is still a successful generic fetch. Decide next action from the content and user goal.
- `backend_unavailable` means an optional local dependency or explicitly configured compatibility backend such as Chrome, cmux, Crawl4AI, or agent-browser is missing.
- `requires_user_action` means the user must do something local, such as complete login or unlock/quit a profile.
- For extraction tuning beyond the core flows above, inspect the project README instead of inventing flags or site-specific workarounds.
