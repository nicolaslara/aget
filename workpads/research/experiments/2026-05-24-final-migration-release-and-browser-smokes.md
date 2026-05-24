# Final Migration Release And Browser Smokes

Date: 2026-05-24

Code snapshot: current `dep-migration-homegrown-backends` worktree after provider-session guidance and retention-clarity fixes.

## Objective

Verify the final migration closure bar with the current release binary and local browser smokes:

- public fetch
- private/session fetch
- current-tab fetch
- provider-session-assisted login start path
- session inspect/list/delete without secret leakage
- focused ignored/manual browser smokes for rendered pages, storage, Chrome import, and headed login state export

## Commands And Results

### Deterministic standard gate

```bash
cargo fmt --check && cargo test
```

Result:

- Passed.
- Summary: 154 lib tests passed with 2 ignored; all integration suites passed; 4 session CLI tests and 1 get CLI test remained ignored for external/manual setups; doc tests passed.

### Focused local Chrome smokes

```bash
cargo test --test mock_site_browser -- --ignored
cargo test owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory --lib -- --ignored
cargo test owned_login_browser_exports_state_from_headed_profile_and_closes --lib -- --ignored
cargo test --test session_cli real_session_import_browser_chrome_profile -- --ignored
```

Result:

- Passed.
- Covered rendered JavaScript, CSS waits, render delay, image readiness, networkidle readiness, shadow DOM, iframe processing, localStorage-backed session rendering, browser fallback replay, real Chrome profile import/export, and headed login-browser state export/close.

### Release build

```bash
cargo build --release
```

Result:

- Passed and refreshed `target/release/aget`.

### Release public fetch

```bash
target/release/aget --envelope json get https://example.com \
  --output /private/tmp/aget-release-public.md \
  --max-chars 2000
```

Result:

- Passed through `aget-owned-extractor`.
- Returned markdown for Example Domain and wrote `/private/tmp/aget-release-public.md`.

### Release no-command-default-path smoke

```bash
env -u AGET_CRAWL4AI_COMMAND -u AGET_AGENT_BROWSER_COMMAND \
  PATH=/usr/bin:/bin:/usr/sbin:/sbin \
  target/release/aget --envelope json get https://example.com \
  --output /private/tmp/aget-release-no-command-path.md \
  --max-chars 1000
```

Result:

- Passed through `aget-owned-extractor`.
- Confirms default public fetch does not require Crawl4AI or `agent-browser` command adapters on PATH.

### Release private/session fetch

```bash
target/release/aget --envelope json get https://github.com/nicolaslara/zodl-desktop \
  --session github \
  --inline-content never \
  --output /private/tmp/aget-release-private-github.md \
  --max-chars 4000 \
  --timeout 30
```

Result:

- Passed with existing local `github` session.
- Output was sensitive and path-only in the envelope.
- Extraction used `aget-owned-browser-fallback` after the primary extractor failed.
- Wrote `/private/tmp/aget-release-private-github.md`; private page content was not copied into this note.

### Release current-tab fetch

Started a temporary Chrome profile with an explicit CDP port and navigated to `https://example.com`, then ran:

```bash
target/release/aget --envelope json current-tab \
  --cdp-port 9333 \
  --allow-private-content \
  --inline-content never \
  --output /private/tmp/aget-release-current-tab.md \
  --max-chars 2000 \
  --timeout 10
```

Result:

- Passed through `aget-owned-current-tab`.
- Result was marked sensitive and path-only.
- Wrote `/private/tmp/aget-release-current-tab.md`.
- Warning recorded the private-content boundary and explicit local CDP port.

### Release provider-session-assisted login start

```bash
target/release/aget --envelope json session login start github-provider-smoke \
  --url https://github.com/login \
  --session github \
  --timeout 10

target/release/aget --envelope json session login cancel github-provider-smoke \
  --timeout 10
```

Result:

- `login start` passed and reported `injected_sessions: ["github"]`.
- `login cancel` passed and cleaned the pending flow.
- No credentials were entered or recorded.

### Release session list/inspect/delete

Existing local session:

```bash
target/release/aget --envelope json session list
target/release/aget --envelope json session inspect github
```

Result:

- Listed `github`.
- Inspect output redacted all cookie and storage values.

Isolated delete cycle:

```bash
AGET_HOME=<temp-home> target/release/aget --envelope json session list
AGET_HOME=<temp-home> target/release/aget --envelope json session inspect demo
AGET_HOME=<temp-home> target/release/aget --envelope json session delete demo
AGET_HOME=<temp-home> target/release/aget --envelope json session list
```

Result:

- Listed and inspected a temporary `demo` session with no secrets.
- Deleted it successfully.
- Final list returned no sessions.

### Tracked-file hygiene

```bash
git ls-files | rg '(^references/repos/|(^|/)\.aget(/|$)|backend-stdout|backend-stderr|raw-state|/private/tmp|aget-release-private|aget-current-tab-profile|\.log$)' || true
```

Result:

- No matches.
- Tracked files do not include dependency source clone contents, `.aget` state, backend artifacts, raw browser state, private `/tmp` outputs, current-tab temp profiles, or logs.

## Review Findings And Resolution

Independent review passes covered test adequacy, architecture cohesion, security/privacy, and docs/user workflow clarity.

Accepted material findings:

- Provider-session skill guidance conflicted with replay-scope semantics.
  - Resolution: skill and README now direct provider sessions into `session login start --session <provider>` rather than unrelated target-site fetches.
- Source-of-truth docs/workpad status still said I19d/I19e/I19h were open.
  - Resolution: this note and final workpad updates record the accept/defer decisions and completion evidence.
- Custom-profile/import retention behavior needed clarity.
  - Resolution: README and skill now document `--profile-path` as advanced/direct-profile behavior, artifact retention after `session delete`, and provider-session injection as default-profile/owned-backend only. Code rejects injected sessions with custom login profiles.

Accepted non-blocker:

- Current-tab warning includes local CDP provenance.
  - Decision: not a blocker. Current-tab already requires `--allow-private-content` and an explicit `--cdp-port`; the warning is useful local diagnostic/provenance information and does not expose remote credentials or private page content by itself.

Deferred backlog after migration:

- Broader browser-family import support beyond verified Chrome.
- Richer artifact retention/garbage-collection commands.
- Further extraction polish that does not change real agent usefulness.
- Packaged install/setup/doctor improvements.
