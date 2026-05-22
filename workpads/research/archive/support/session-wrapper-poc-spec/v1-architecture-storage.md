## V1 Architecture, Dependencies, And Storage

### Architecture

V1 is a CLI wrapper around existing backends.

```text
aget CLI
  session store
  session filter/composer
  temp Playwright state writer
  backend adapters
    cmux adapter
    agent-browser adapter
    Crawl4AI adapter
```

V1 should be boring and explicit. It should prove the UX and session model before replacing underlying utilities.

### Language

Use Rust if the project direction is still Rust, but do not block the PoC on Rust purity. A small CLI in Rust that shells out to existing tools is enough. A Python/Node prototype is acceptable only if clearly marked throwaway.

Recommended v1 shape:

- Rust CLI for product surface, filesystem layout, filtering, and process orchestration.
- Python helper script for Crawl4AI extraction if direct SDK use is simplest.
- External commands for `agent-browser` and `cmux`.

### Dependencies

Runtime tools:

- `npx -y agent-browser` for Chrome profile/session import.
- `uv run --with crawl4ai` or a bundled Python environment for Crawl4AI extraction.
- `cmux` optional, detected dynamically.

V1 should degrade cleanly:

- If cmux is unavailable, hide or error only on `session import cmux` and `current --from cmux`.
- If `agent-browser` is unavailable, error only on Chrome/agent-browser imports.
- If Crawl4AI is unavailable, fetch commands requiring rendered markdown should explain setup.

### Storage Layout

Default local storage:

```text
~/.aget/
  sessions/
    hellointerview.json
    google.json
  runs/
    <run-id>/
      content.md
      metadata.json
  tmp/
```

Session files should not live in the project repo.

V1 session file shape:

```json
{
  "version": 1,
  "name": "hellointerview",
  "source": {
    "type": "cmux",
    "surface": "surface:8"
  },
  "created_at": "2026-05-06T00:00:00Z",
  "updated_at": "2026-05-06T00:00:00Z",
  "sensitive": true,
  "allowed_cookie_domains": ["www.hellointerview.com"],
  "allowed_storage_origins": ["https://www.hellointerview.com"],
  "cookies": [
    {
      "name": "hi.session-token-2",
      "value": "...",
      "domain": "www.hellointerview.com",
      "path": "/",
      "expires": 1780671846,
      "httpOnly": true,
      "secure": true,
      "sameSite": "Lax",
      "source_session": "hellointerview"
    }
  ],
  "origins": [
    {
      "origin": "https://www.hellointerview.com",
      "localStorage": []
    }
  ]
}
```

V1 can store plaintext locally with restrictive permissions if needed for speed, but the CLI must mark this as a PoC limitation. Encryption at rest should be a near-term hardening task.
