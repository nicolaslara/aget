### D25: Explicit copy/import is the MVP auth ownership model

R4a compared three auth/session ownership models: direct use of existing browser data, a dedicated `aget` browser/profile, and explicit copy/import into `aget`'s local session store. The MVP default should remain explicit copy/import into scoped `aget` sessions. Direct existing-browser use and dedicated login/profile flows are useful advanced modes, but they have larger consent, lifecycle, and reliability surfaces.

| Model | Fetch-time store | UX | Technical feasibility | Platform constraints | Privacy risk | Credential leakage risk | Profile lock/corruption risk | Auditability |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Direct existing browser data | User's live browser/profile or debug session at fetch time | Most seamless if already logged in; highest risk of surprising the user because the tool touches the active browser environment | Feasible through CDP/debug-port, WebDriver/BiDi, extension/native bridge, or tool-specific auto-connect; CDP is browser-version-dependent and not a stable testing API | Chrome user data directories are platform/channel-specific; Chrome remote debugging exposes full local browser control; Firefox profiles are OS-locked while in use; WebDriver BiDi is the standards path but still requires explicit remote-control setup | Highest, because every fetch may see ambient browser state unrelated to the target task | Highest, because a local automation endpoint or extension can expose cookies, storage, DOM, and private page content broadly | Medium to high if reusing profile directories directly; concurrent browser/profile use can fail or risk data loss across versions | Weak unless every direct access is explicitly logged with consent, target origin, profile/session identifier, and redacted result metadata |
| Dedicated `aget` browser/profile login | `aget`-owned profile at fetch time | Clear ownership once created; user logs in through a tool-owned browser/profile instead of normal browser | Feasible with Playwright persistent contexts, agent-browser persistent profile paths, or future Rust CDP/WebDriver adapter | OAuth/SSO can reject automation-controlled browsers; users may need visible login flows; cross-browser support varies; profile location and lifecycle are `aget`-owned | Medium, because data is isolated from the user's main browser but still broad within the profile | Medium, because profile data remains credential-equivalent and may include provider cookies/storage beyond the relying-party app | Low to medium if each profile is single-owner and not shared with other running browser processes | Strong: `aget` can name the profile, record consent, target domains/origins, and warn when provider domains are present |
| Explicit copy/import into `aget` storage | `aget`'s scoped local session JSON at fetch time | Slightly more explicit setup, but best agent UX afterward: `aget get <url> --session <name>` never reads ambient browser state | Proven locally via cmux domain cookie import and agent-browser state export feeding Crawl4AI; requires robust filtering from broad source state to explicit cookie domains/storage origins | Browser/profile export can require Chrome to be quit or remote debugging enabled; exported state files are plaintext unless encrypted; backend output formats differ and must be normalized | Lowest for normal fetches because only pre-approved scoped state is used; import step is the risky boundary | Medium at import time because raw exported state is credential-equivalent; low during fetch if raw state is filtered, redacted, and deleted | Low for persisted `aget` sessions because original profile is not reused at fetch time; import may still hit locks or incomplete snapshots | Strongest: every session records source, allowed domains/origins, sensitivity, and provenance; normal inspect redacts values |

Recommendation:

- MVP default: explicit copy/import into `aget` storage. Fetches use only `aget`'s scoped local session file plus temporary Playwright state. This matches the no-ambient-auth rule and makes the consent boundary auditable.
- Near-term implementation order: manual fixtures, cmux import, and agent-browser/Chrome import as explicit import commands. Import commands must post-filter by allowlist, delete raw broad state after success and failure, and return `requires_user_action` instead of closing or disturbing the user's browser.
- Advanced/deferred: direct CDP/auto-connect/current-browser access should be an explicit advanced mode, not a default fetch path. A dedicated `aget login/profile` flow is attractive after import works, but should wait until the product can open the exact target login URL, explain profile ownership, and avoid automating credentials.

Audit and logging rules for all models:

- Log metadata only: timestamp, command, source type, session/profile name or salted hash, requested domains/origins, target origin, result code, warning class, and whether sensitive data was used.
- Do not log raw cookies, storage values, access tokens, passwords, encryption keys, full private page content, unredacted provider identifiers, or raw browser-state file paths when they may reveal account names.
- Treat full URLs as potentially sensitive in authenticated contexts; record origin by default and keep full URL only in local run metadata when needed for reproducibility.
- Apply data minimization to import and audit records: collect only explicitly allowed domains/origins, retain raw broad exports for the shortest possible time, and support deletion/disposition of sessions and logs.

Open questions for the later security/privacy model:

- What encryption-at-rest boundary is required before broader use: only session files, or also run metadata, logs, cache, and temporary raw state?
- Should audit logs be a separate feature with retention controls, or should provenance stay embedded in session/run metadata for the MVP?
- Should authenticated run metadata record full URLs by default, origin-only by default, or configurable redacted URLs?
- What exact consent UX is required before direct CDP/auto-connect access to an existing browser, given that local debugging endpoints expose broad browser control?
- How should `aget` detect and report partial or stale imports caused by locked browser profiles without leaking profile paths or cookie names?

### D26: R4 persistent profile strategy favors explicit state import over live profile reuse

R4 compared Playwright persistent contexts, CDP attach, and WebDriver-based profile reuse for authenticated local extraction. The recommendation stays aligned with D25: the MVP should use explicit copy/import into `aget` storage for normal fetches, with `agent-browser`/Chrome import as an import-time bridge. Direct use of a live existing browser/profile should remain an advanced/deferred mode because it has the broadest control surface, browser/version sensitivity, and profile-locking risk.

| Strategy | Login reuse behavior | Risks | Platform constraints | Fit for `aget` |
| --- | --- | --- | --- | --- |
| Playwright persistent context | Uses an on-disk `userDataDir` and keeps cookies/local storage/profile data across launches. `storageState` can also export cookies, localStorage, and IndexedDB for replay into a fresh context, but Playwright does not persist `sessionStorage` through the storage-state API. | Persistent profile directories are credential-equivalent local state. Reusing a user's default Chrome profile is explicitly discouraged; shared mutable state can leak across tasks or break when tests/fetches mutate server-side state. | Browsers do not allow multiple running instances with the same user data directory. The safe pattern is a dedicated automation profile directory, not the user's main profile. | Good for a future `aget login/profile` flow where `aget` owns the profile lifecycle. For I7, exported storage state is the better bridge because `aget` can filter and persist only scoped session material. |
| CDP attach / Chrome remote debugging | Can attach to an already-running or explicitly launched Chromium/Chrome instance and inspect/control pages through DevTools Protocol WebSockets. It can observe the browser's live authenticated page state when the endpoint has access. | A CDP endpoint is effectively a live browser control/data endpoint for open pages. It exposes authenticated DOM/storage/cookies via debugging capabilities, is Chrome/CDP-version sensitive, and has a sharp consent boundary. | Chrome 136+ blocks `--remote-debugging-port`/`--remote-debugging-pipe` against the default Chrome data directory unless a non-standard `--user-data-dir` is supplied. User data directories contain cookies/history/bookmarks and use singleton/lock files; cross-version profile reuse can cause degraded behavior, crashes, or data loss. | Useful as an explicit advanced/current-browser mode later. Not the MVP default. It may be used indirectly by `agent-browser` during import, but `aget` should treat raw exported state as short-lived and filter it before persistence. |
| WebDriver / WebDriver BiDi | Standardizes browser automation sessions and, with BiDi, bidirectional messaging/subscriptions. Persistent login reuse is possible only through browser-specific launch options such as Chrome `--user-data-dir` or Firefox `-profile`. | Profile reuse inherits browser-profile risks while providing less direct, portable auth-state semantics than Playwright storage state. The spec does not standardize a safe persistent-profile/auth-state model. | WebDriver capabilities are portable for session creation, timeouts, proxy, and browser metadata, but profile persistence is vendor-specific. BiDi improves eventing/control but does not define profile persistence. | Good standards direction for future pure browser automation research, but not the shortest path for I7. It does not replace the current `agent-browser` plus filtered storage-state import plan. |

MVP auth/session strategy after R4:

- Keep `aget get` empty-session by default.
- Keep normal authenticated fetches on scoped `aget` session files, never ambient browser stores.
- Implement I7 as explicit Chrome import through `agent-browser`: create/use a named temporary agent-browser session, export raw state to a private temp file, filter by explicit domains/origins, save only scoped state, and delete the raw broad export on success and failure.
- If Chrome/profile state cannot be acquired cleanly, return `requires_user_action`; do not close or disturb the user's running browser.
- Defer a first-class `aget login/profile` persistent-context flow until the product can own a dedicated profile directory, open the exact target login URL, explain profile ownership, and document sessionStorage limitations.
- Defer direct CDP/current-browser attach until it has explicit consent UX, endpoint exposure warnings, and redacted audit metadata.

Confidence: High for the MVP direction. The recommendation is supported by primary Playwright, Chrome/Chromium, WebDriver, and Selenium sources, and it matches local benchmark evidence from D17/D18 plus the explicit ownership model from D25. Remaining uncertainty is implementation-specific: how reliably `agent-browser` can export state across Chrome profile lock/version conditions without requiring user action.

### D27: I7 Chrome import uses agent-browser only as a scoped acquisition backend

I7 adds `aget session import chrome --profile <profile> --name <name> --domain <domain>...`. The implementation generates a unique temporary agent-browser session name (`aget-import-<pid>-<timestamp>`), runs the equivalent of `agent-browser --profile <profile> --session <temp> open about:blank`, `agent-browser --session <temp> state save <raw-state-path>`, then `agent-browser --session <temp> close`, and never uses broad close/close-all. `AGET_AGENT_BROWSER_COMMAND` exists for fake-command tests and is executed directly with `Command::new`, not through a shell.

The raw agent-browser state is treated as bearer material because it can contain live cookies plus local/session storage. It is written only under `~/.aget/tmp` to a pre-created `0600` temp file, parsed as Playwright-compatible `{cookies, origins}`, filtered by the explicit domain allowlist, and deleted on success and failure. Persisted sessions contain only allowlisted cookies and localStorage origins whose parsed origin host matches the same exact/suffix rules used by cmux import; unfiltered raw state is never saved to `sessions/`. Conflicting duplicate cookies or same-origin localStorage keys are rejected instead of silently choosing one value; disjoint localStorage keys for the same origin can be merged during session composition.

Chrome/profile acquisition can fail when Chrome is still running, the profile is locked/in use, the user is not logged in, or no auth state is present for the allowlist. These cases map to stable `requires_user_action`; `aget` does not try to quit Chrome or automate login. Missing `agent-browser` maps to `backend_unavailable`, and malformed storage-state JSON maps to `extraction_failed`.

Manual verification commands for a user-authorized Chrome profile:

```bash
export AGET_HOME="$(mktemp -d)"
cargo run -- --json session import chrome --profile Default --name hi --domain hellointerview.com --domain www.hellointerview.com
cargo run -- session inspect hi
cargo run -- --json get "https://www.hellointerview.com/learn/behavioral/course/adapting-to-big-tech-behaviorals" --session hi --timeout 60
```

If the import returns `requires_user_action`, manually quit Chrome after saving work and rerun the same import command. Do not run any command that closes Chrome on the user's behalf. After a successful manual import, inspect `~/.aget/tmp` and confirm no `agent-browser-raw-state-*.json` files remain; inspect the saved session only with `--show-secrets` if explicitly needed because values are live bearer material.

### D28: I8 multi-session composition keeps sessions explicit and provenance-preserving

I8 adds repeated `aget get <url> --session <name> --session <name>` support and `aget session compose <new-name> --session <name>...`. Request-time composition loads the named local sessions in flag order, builds one temporary Playwright storage-state file, returns the selected session names in JSON/metadata, and marks the run sensitive whenever any session is selected. The empty-session default remains unchanged.

Persisted composition saves a new `SessionSource::Composed { sessions }` session without mutating its sources. Cookie provenance is preserved when already present and filled from the contributing source session otherwise. Storage-origin provenance is preserved for single-source origins; multi-source origins keep merged localStorage entries but omit a single origin-level source because no one source owns the whole origin.

Conflict handling remains strict and redacted. Cookie conflicts report only name/domain/path, same-origin localStorage conflicts report only key/origin, and `session compose` rejects a target that matches any source or already exists because there is no `--force` flag. Disjoint localStorage keys for the same origin are merged deterministically so provider/app sessions can compose without losing separate storage entries.

Post-review hardening added generic session-backed backend failure errors plus artifact redaction for cookie/localStorage values that reached the backend, plain `session inspect` provenance output, target-overwrite rejection, and a local app/provider cookie-flow test. Verification passed `cargo test --test get_cli`, `cargo test --test session_cli`, full `cargo test`, `cargo fmt --check`, and `git diff --check`. LSP diagnostics remain limited by the known local rust-analyzer proc-macro version mismatch; cargo compile/tests are clean.

