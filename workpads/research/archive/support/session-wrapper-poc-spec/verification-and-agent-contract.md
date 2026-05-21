### Verification Plan

V1 must include at least these tests or scripts:

#### Unit Tests

- CLI parsing: `aget <url>` is equivalent to `aget get <url>`.
- Session file load/save redacts values in normal inspection.
- Cookie domain allowlist filtering accepts only permitted domains.
- Storage origin allowlist filtering accepts only permitted origins.
- Multiple sessions compose deterministically.
- Duplicate identical cookies deduplicate.
- Duplicate conflicting cookies fail.
- Provider-domain pruning removes only configured domains.
- Extractor options normalize into backend config.
- Timeout parsing and propagation works.

#### Integration Tests

- Empty fetch does not include test cookie.
- cmux domain-scoped import captures only the requested domain cookie.
- cmux imported cookie replays through Crawl4AI.
- Multiple sessions compose into one Playwright state.
- Conflict detection rejects duplicate cookie conflicts.
- Temp state files are deleted after extraction.
- `session inspect` redacts values by default.
- `aget get --format markdown|html|text|json` returns the expected shape where supported.
- `aget get --max-chars` truncates deterministically and records truncation metadata.
- Missing optional backend returns a structured actionable error.
- Non-interactive commands fail fast with `requires_user_action` when user action is needed.

Manual verification:

- Import an authenticated site session from Chrome using `agent-browser`.
- Fetch the authenticated page via Crawl4AI.
- Confirm logged-in markers are present and paywall/sign-in markers are absent.

#### E2E Tests

Use a local test web app rather than third-party sites for repeatable CI/e2e.

The test app should provide:

- Public page.
- Login page that sets app cookies.
- OAuth-like provider page on a separate host/port that sets provider cookies and redirects back.
- Authenticated dashboard.
- Page requiring app session only.
- Page requiring app plus provider session for testing composition.
- JS-rendered content page.
- Search endpoint and form, reserved for future action/API-mode tests.

E2E scenarios:

- `aget <public-url>` fetches with empty session.
- Import app session and fetch authenticated dashboard.
- Compose provider plus app sessions and fetch a page requiring both.
- Prune provider session and verify app-only page still works.
- Verify broad state is never persisted after Chrome import.
- Verify temp Playwright state is removed after every run, including failures.
- Verify authenticated output metadata is marked sensitive.

CI should run unit and local integration tests on every change. E2E browser tests can run locally and in CI if dependencies are available; otherwise CI should clearly mark them skipped with a reason.

### Agent Execution Contract

Because `aget` is designed for agents, command behavior must be predictable:

- Default output should be concise human-readable text.
- `--json` should return machine-readable structured output with stable fields.
- Non-zero exit codes should distinguish categories: usage error, backend unavailable, timeout, requires user action, auth failed, extraction failed.
- Commands should never wait indefinitely for login, navigation, or page load.
- Timeouts should be configurable globally and per command.
- Commands should not open visible browser UI unless explicitly requested with `--interactive` or an explicit open/login command.
- Sensitive values should never appear in stdout/stderr unless `--show-secrets` is passed.

Suggested default timeouts:

- CLI command total: 60s.
- Page navigation: 30s.
- Extraction: 45s.
- Backend startup: 20s.
- Login/user action: not allowed in non-interactive commands.

