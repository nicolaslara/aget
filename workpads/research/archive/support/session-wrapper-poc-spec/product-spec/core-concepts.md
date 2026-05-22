# Session Wrapper Product Spec: Core Concepts

## Empty Session

The default fetch context contains no imported cookies or storage.

```bash
aget get https://example.com
```

This must not silently use Chrome, cmux, OS browser cookies, prior OAuth sessions, or any saved `aget` session.

## Session

A session is a named local bundle of credential-equivalent browser state scoped by explicit domains and origins.

Examples:

```bash
aget session list
aget session inspect hellointerview
aget session delete hellointerview
```

Each session records:

- Name.
- Allowed cookie domains.
- Allowed storage origins.
- Cookie metadata, with values hidden by default in inspection.
- localStorage/sessionStorage entries when imported.
- Source: `cmux`, `agent-browser`, `chrome-profile`, `manual`, or `composed`.
- Creation and last-used timestamps.
- Sensitivity flags.
- Optional expiry hints.

## Session Combination

Requests may use multiple sessions.

```bash
aget get https://www.hellointerview.com/dashboard \
  --session google \
  --session hellointerview
```

`aget` combines the selected sessions into a temporary replay state for the extraction backend. It does not mutate the source sessions unless explicitly requested.

Conflicts are explicit:

- Same cookie name/domain/path from multiple sessions is a conflict.
- Same storage origin/key from multiple sessions is a conflict.
- Default behavior should fail with a clear message.
- Later versions may support `--prefer-session <name>`.

## Composed Session

A composed session is a named session created from other sessions.

```bash
aget session compose hi-oauth \
  --session google \
  --session hellointerview
```

This is useful for OAuth provider plus relying-party flows. Composition should preserve source provenance for each cookie/origin.

## Provider Sessions

Provider sessions, such as `google` or `facebook`, are useful during login but often should not be used for normal crawling after the relying-party app session has been established.

The UX should make this distinction visible:

```bash
aget session inspect hi-oauth
```

Example output:

```text
Session: hi-oauth
Includes:
- google: accounts.google.com, .google.com
- hellointerview: www.hellointerview.com

Warning: includes OAuth provider session material.
Recommended after login: aget session test hellointerview <url>, then prune provider domains if no longer needed.
```
