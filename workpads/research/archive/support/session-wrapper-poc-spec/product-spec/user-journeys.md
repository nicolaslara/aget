# Session Wrapper Product Spec: Primary User Journeys

## Public Fetch

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

## Import From cmux Browser Pane

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

## Import From Existing Chrome Login

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

## Fetch With One Session

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

## Fetch With Combined Sessions

```bash
aget get https://www.hellointerview.com/login \
  --session google \
  --session hellointerview
```

Expected behavior:

- Combines sessions only for this run.
- Shows a warning if provider domains are included.
- Does not persist the combination unless `session compose` is used.

## OAuth Cleanup

After a login flow:

```bash
aget session test hellointerview https://www.hellointerview.com/dashboard
aget session prune hellointerview --drop-provider-domains
```

Expected behavior:

- Verifies whether the relying-party session works without provider cookies.
- Removes provider domains only when explicitly requested.
