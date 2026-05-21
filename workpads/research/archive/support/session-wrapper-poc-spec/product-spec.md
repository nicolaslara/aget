# aget Session Wrapper PoC Spec

## Product Spec

### Summary

`aget` is a local-first session and extraction tool for agents. Given a URL, visible browser surface, or saved session, it produces clean agent-ready markdown while keeping authenticated content and credential-equivalent session material local by default.

The PoC should be a thin wrapper over existing tools:

- `agent-browser` for Chrome profile snapshotting and browser state export.
- Crawl4AI for browser-rendered markdown extraction from Playwright-compatible state.
- cmux as an optional integration for users already browsing in cmux panes.

The product value is not the extraction backend itself. The product value is safe, explicit, composable local session management for agent web access.

### Problem

Hosted URL-to-markdown tools are excellent for public pages but are the wrong default for authenticated/private content. Browser automation tools can interact with logged-in pages but usually expose raw browser state, lack markdown-quality output, or force awkward profile/login workflows.

Users need a local tool that can:

- Start with no ambient auth by default.
- Create independent named sessions.
- Import only approved session material from a browser or browser surface.
- Combine sessions intentionally for a request.
- Fetch authenticated pages locally as clean markdown.
- Avoid sending cookies, local storage, private HTML, screenshots, or extracted content to hosted services.

### Goals

- Provide an empty-session default: `aget get <url>` should use no saved cookies unless explicitly requested.
- Provide `aget <url>` as a shorthand alias for `aget get <url>`.
- Support independent named sessions: `google`, `facebook`, `hellointerview`, `github-work`, `github-personal`.
- Support combining sessions per request: `aget get <url> --session google --session hellointerview`.
- Support creating a new session from a combination: `aget session compose hi-oauth --session google --session hellointerview`.
- Support OAuth-style workflows where provider state is useful during login but can be excluded later.
- Support users with and without cmux.
- Support agent-safe non-interactive operation by default.
- Support bounded runtime with configurable timeouts.
- Allow passing extraction options through to the backend without baking every extractor feature into the top-level command model.
- Treat all session state as credential-equivalent bearer material.
- Keep v1 as a thin wrapper with a clear migration path to owned v2 internals.

### Non-Goals For PoC

- Do not implement a custom browser engine or crawler.
- Do not implement multi-step site action APIs such as search, add-to-cart, posting, checkout, or account mutation.
- Do not bypass paywalls, access controls, anti-bot systems, or site policy.
- Do not automate credential entry.
- Do not send authenticated content or session state to hosted extraction services.
- Do not depend on cmux as a required runtime.
- Do not require users to use their primary browser profile by default.

### Core Concepts

#### Empty Session

The default fetch context contains no imported cookies or storage.

```bash
aget get https://example.com
```

This must not silently use Chrome, cmux, OS browser cookies, prior OAuth sessions, or any saved `aget` session.

#### Session

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

#### Session Combination

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

#### Composed Session

A composed session is a named session created from other sessions.

```bash
aget session compose hi-oauth \
  --session google \
  --session hellointerview
```

This is useful for OAuth provider plus relying-party flows. Composition should preserve source provenance for each cookie/origin.

#### Provider Sessions

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

### Primary User Journeys

#### Public Fetch

```bash
aget get https://example.com
```

Equivalent shorthand:

```bash
aget https://example.com
```

Expected behavior:

- Uses empty session.
- Fetches/render-extracts locally.
- Outputs markdown.
- Records no auth state.

#### Import From cmux Browser Pane

For users already browsing in cmux:

```bash
aget session import cmux \
  --surface surface:8 \
  --name hellointerview \
  --domain www.hellointerview.com
```

Expected behavior:

- Reads cookies from the cmux browser surface.
- Uses explicit `--domain` filters, not URL-only filtering.
- Post-filters returned cookies against the allowlist.
- Optionally reads local/session storage for explicit origins.
- Saves a scoped local session.
- Does not call `cmux browser state save` by default.

#### Import From Existing Chrome Login

For users without cmux:

```bash
aget session import chrome \
  --profile Default \
  --name hellointerview \
  --domain www.hellointerview.com
```

Expected behavior in v1:

- Uses `agent-browser --profile Default` as the bridge.
- Exports decrypted state with `agent-browser state save` into a temporary file.
- Filters that state to the requested domains/origins.
- Stores only the scoped session in `aget` storage.
- Deletes the raw broad temporary state immediately.
- If Chrome must be quit for a clean snapshot, stops and asks the user rather than closing Chrome.

#### Fetch With One Session

```bash
aget get https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals \
  --session hellointerview
```

Expected behavior:

- Builds a temporary Playwright state file from the selected session.
- Runs Crawl4AI locally with that state.
- Deletes the temporary state file.
- Writes authenticated markdown to local output.
- Marks output as sensitive.

Extractor options can reduce or reshape output:

```bash
aget get https://www.hellointerview.com/... \
  --session hellointerview \
  --format markdown \
  --selector main \
  --max-chars 8000 \
  --wait-for "Video Content"
```

#### Fetch With Combined Sessions

```bash
aget get https://www.hellointerview.com/login \
  --session google \
  --session hellointerview
```

Expected behavior:

- Combines sessions only for this run.
- Shows a warning if provider domains are included.
- Does not persist the combination unless `session compose` is used.

#### OAuth Cleanup

After a login flow:

```bash
aget session test hellointerview https://www.hellointerview.com/dashboard
aget session prune hellointerview --drop-provider-domains
```

Expected behavior:

- Verifies whether the relying-party session works without provider cookies.
- Removes provider domains only when explicitly requested.

### UX Principles

- No ambient auth: auth is always selected explicitly.
- No surprise browser lifecycle changes.
- No broad session exports by default.
- Non-interactive by default so agents can call commands reliably.
- Any command that may require user action must fail with a clear machine-readable reason unless `--interactive` or a dedicated interactive subcommand is used.
- No cookie values in normal logs.
- Sensitive temporary files are created in OS temp storage and deleted immediately.
- Authenticated outputs are sensitive too.
- Every run should explain which sessions were used and which domains were allowed.
- Every imported session should be inspectable without revealing values.
- Timeouts should be explicit and configurable; no command should hang indefinitely.

### Security And Privacy Requirements

- Treat cookies, localStorage, sessionStorage, and exported browser state as credential-equivalent.
- Store persisted session material outside project repos by default.
- Use restrictive filesystem permissions for session files.
- Avoid writing raw broad state files except as short-lived temp files.
- Do not include session values in command output unless `--show-secrets` is explicitly passed.
- Do not commit session files or authenticated outputs.
- Keep hosted services out of authenticated fetch paths.
- Make provider-session inclusion explicit.
- Always post-filter imported cookies/storage against allowlists, even if the backend claims to filter.

