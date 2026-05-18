---
name: aget
description: Use the local aget CLI for agent-friendly URL fetching, local markdown extraction, and explicit user-authorized session replay. Use when a task needs page content through `aget`, authenticated/gated-page fetches, `aget session login`, session composition, cmux or Chrome session import, or safe local-only web context workflows.
disable-model-invocation: true
---

# aget

Use `aget` as a generic local fetcher. It returns page content and extraction outcomes; the agent decides whether returned content means login, subscription, stale auth, or a narrower selector is needed.

## Safety Rules

- Only process content the user is authorized to access.
- Never collect, type, script, store, or ask the user to reveal credentials.
- Do not try to bypass paywalls, access controls, anti-bot systems, or site policy.
- Do not use ambient browser auth silently. Authenticated fetches require an explicit named session.
- Treat session files, storage-state temp files, screenshots, authenticated markdown, and envelope content as private local data.
- Prefer `--out <path>` for large or sensitive content so the agent can read only the needed artifact.

## Structured Output

Prefer `--envelope`; `--json` is a compatibility alias.

Success shape:

```json
{"ok": true, "command": "get", "data": {}, "warnings": [], "timing_ms": {"total": 0}}
```

Error shape:

```json
{"ok": false, "command": "get", "error": {"code": "requires_user_action", "message": "..."}}
```

For `get`, extracted page content is in `data.content`, artifact paths are in `data.artifacts`, and selected sessions are in `data.sessions`.

## Basic Fetch

Use an empty session first unless the user already chose a named session:

```bash
aget --envelope get "https://example.com/docs" --format markdown
```

For long output:

```bash
aget --envelope get "https://example.com/docs" --format markdown --out /tmp/aget-page.md --max-chars 12000
```

## Gated Page Flow

Example starting point with HelloInterview as a user-authorized representative gated site:

```bash
aget --envelope get "https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals" --format markdown --out /tmp/hi.md
```

If the returned content looks like a login/subscription wall, tell the user what you observed and ask whether they want to create a local session. If they agree:

```bash
aget --envelope session login start hellointerview --url "https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals"
```

The user completes login manually in the opened browser. Then finish:

```bash
aget --envelope session login finish hellointerview
aget --envelope get "https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals" --session hellointerview --format markdown --out /tmp/hi-auth.md
```

Generalize the same pattern to any user-authorized gated site, such as `ft.com`, `nytimes.com`, private docs, dashboards, or account pages. Session names are caller-chosen labels, not built-in site handlers:

```bash
aget --envelope session login start news --url "https://www.nytimes.com/account"
aget --envelope session login finish news
aget --envelope get "https://www.nytimes.com/account" --session news --out /tmp/news-account.md
```

## Combining Sessions

Some sites require provider cookies during login but not later fetches. Keep provider/app sessions explicit.

During fetch:

```bash
aget --envelope get "https://docs.example.com/account" --session provider --session app --out /tmp/account.md
```

Persist a reusable composition:

```bash
aget --envelope session compose workdocs --session provider --session app
aget --envelope get "https://docs.example.com/account" --session workdocs --out /tmp/account.md
```

If session composition reports a conflict, do not guess which secret wins. Ask the user which session should be replaced, deleted, or retried.

## Combining Sessions While Logging In

`aget session login start` does not currently accept `--session`. Do not pretend provider/app session composition is built into the login command.

If a login flow appears to require provider cookies beyond the relying-party session:

- Explain the limitation to the user.
- Ask whether they want to import or create each required session explicitly.
- Use request-time composition or `aget session compose` after those sessions exist.
- Prefer the narrowest relying-party session for later fetches if it works without provider cookies.

## Importing Existing Local Auth

cmux cookie import is explicit and domain-scoped:

```bash
aget --envelope session import cmux --surface "surface:1" --name workdocs --domain docs.example.com
```

Chrome import uses `agent-browser` as a scoped acquisition backend:

```bash
aget --envelope session import chrome --profile Default --name workdocs --domain docs.example.com
```

Before running either import command, ask the user to approve the specific local surface/profile and domains. These commands can read credential-equivalent local browser state.

If Chrome import returns `requires_user_action`, do not close the user's browser. Relay the message and let the user decide whether to quit Chrome and retry.

## Interpreting Results

- `ok: true` with login-wall-looking content is still a successful generic fetch. Decide next action from the content and user goal.
- `backend_unavailable` means an optional backend such as Crawl4AI, cmux, or agent-browser is missing.
- `requires_user_action` means the user must do something local, such as complete login or unlock/quit a profile.
- For extraction tuning beyond the core flows above, inspect the project README instead of inventing flags or site-specific workarounds.
