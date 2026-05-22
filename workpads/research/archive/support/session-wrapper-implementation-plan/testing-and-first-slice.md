# Session Wrapper Implementation Plan: Testing And First Build Slice

## Testing Strategy

### Unit Tests

Run on every change:

- CLI alias parsing.
- Session model serialization/deserialization.
- Domain and origin allowlist filtering.
- Cookie normalization from cmux and agent-browser shapes.
- Session composition and conflict detection.
- Metadata generation.
- Redaction behavior.
- Timeout parsing.

### Integration Tests

Run locally and in CI where dependencies exist:

- Local HTTP test server for public/authenticated/cookie echo pages.
- Empty fetch has no cookies.
- Session replay sends expected cookies.
- Temp state cleanup on success and failure.
- Missing backend errors are structured.
- `AGET_HOME` isolates test state.

### E2E Tests

Use a local browser-backed test app:

- Public page extraction.
- JS-rendered page extraction.
- App login cookie import.
- OAuth-like provider plus app session composition.
- Provider pruning.
- Sensitive output metadata.

cmux e2e tests should be optional/skipped when cmux is unavailable.

agent-browser/Chrome import e2e should start as manual or ignored tests because it depends on local browser state.

## First Build Slice

Start with this exact slice:

1. Rust CLI skeleton.
2. Session model/store with `AGET_HOME` test override.
3. Playwright state writer from zero/one sessions.
4. Crawl4AI helper and `aget get` with empty session.
5. Local integration test proving cookie replay via a hand-written session file.

Only after that add cmux import.

This avoids entangling session storage, backend orchestration, and cmux quirks before the core replay path is tested.
