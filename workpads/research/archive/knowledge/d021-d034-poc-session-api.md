### D21: I3 empty-session Crawl4AI fetch is implemented

I3 is complete: `aget get` defaults to empty Playwright storage state, shells out to the local Crawl4AI helper, and writes run artifacts as `content.md` and `metadata.json`. The stable JSON contract includes `extractor: "crawl4ai"`, and `scripts/demo_real_cli.sh` is the reusable real CLI demo path showing the default-backend flow.

Real demo runs surfaced Crawl4AI stdout progress logs; the helper now handles this by parsing the final JSON line and redirecting Crawl4AI stdout to stderr so progress noise no longer breaks extraction.

Verification covered fake-backend and local-server tests for success, timeout, error paths, temp cleanup, and noisy stderr; `cargo test`, `cargo fmt --check`, and `git diff --check` passed, and Oracle blocker review was PASS.

Future work stays split as I4 for session replay and I6 for output shaping.

### D22: I4 local cookie replay path is implemented

I4 adds explicit one-session replay for `aget get <url> --session <name>`. The Rust CLI loads the named local session, composes it into a temporary Playwright storage-state file, and passes that file through the existing Crawl4AI helper. Empty fetches still compose `{"cookies": [], "origins": []}` and return `sessions: []`, `sensitive: false`.

Verification now covers two layers. The normal fake-backend integration test asserts that the generated Playwright state contains the hand-written cookie fixture and that command output plus `metadata.json` record `sessions: ["local"]` and `sensitive: true`. An ignored real-backend integration test, `real_crawl4ai_replays_named_session_cookie`, starts a local cookie echo server and verifies default Crawl4AI behavior: empty state does not send the synthetic `sid` cookie, while `--session local` sends `sid=secret-cookie` through the rendered browser request.

The real test also surfaced two useful backend behaviors. Crawl4AI injects its own `cookiesEnabled=true` cookie even with empty user state, so tests should assert absence of the selected session cookie rather than zero Cookie header. Crawl4AI 0.8.6 flags tiny local pages as `minimal_text`, so local replay fixtures need enough visible text to pass structural checks.

Post-implementation review found and fixed three hardening points: session-backed fetches are marked sensitive whenever any session is selected, session names reject path-like values before loading/saving/deleting, and nonzero backend exits cannot be converted into successful `aget` results even if stdout contains `{"ok": true}`.

### D23: I5 optional cmux cookie import is implemented

I5 adds `aget session import cmux --surface <surface> --name <name> --domain <domain>...`. The import path shells out to cmux's cookie API through an optional `AGET_CMUX_COMMAND` override, stores the result as `SessionSource::Cmux { surface }`, marks the session sensitive, and persists only cookies plus explicit `allowed_cookie_domains`. It does not call `cmux browser state save` and does not use cmux URL scoping.

The adapter treats cmux's domain filter as coarse because source research confirmed cmux filters domains by substring. `aget` therefore post-filters every returned cookie by exact/suffix domain rules before persistence. Host-only `example.com` and domain cookie `.example.com` are not accepted when only `docs.example.com` is allowed; the parent domain must be explicitly allowed before broader parent-domain cookies are imported. Because cmux's cookie JSON does not expose `HttpOnly`, imported cookies default to `http_only: true` to avoid weakening browser cookie protections during replay.

Verification covers fake and optional real paths. `tests/session_cli.rs` uses a fake cmux CLI to prove repeated `--domain`, sensitive session persistence, redacted inspect output, disallowed-cookie filtering, and missing-backend `backend_unavailable`. The ignored `real_cmux_imports_loopback_cookie` test uses a user-provided disposable cmux surface, imports a loopback cookie, and replays it through `aget get --session` with a fake Crawl4AI backend that reads the generated Playwright state and sends the cookie to a loopback echo server. The ignored `real_cmux_import_replays_loopback_cookie_through_crawl4ai` test uses the same loopback-only import setup and replays the imported session through the real Crawl4AI backend.

Verification passed: `cargo test --test session_cli`, `cargo test domain_matching_rejects_substring_only_matches`, `cargo test`, `cargo fmt --check`, `cargo check`, `git diff --check`, and a manual fake-cmux CLI smoke. `lsp_diagnostics` remains blocked for macro-heavy Rust files by the local rust-analyzer/proc-macro API mismatch (`proc-macro server's api version (6) is newer than rust-analyzer's (5)`), while cargo compile/tests are clean.

Post-implementation review initially found two blockers and one acceptance gap. The blockers were fixed by defaulting unknown cmux cookies to `http_only: true`, replacing shell-interpreted `AGET_CMUX_COMMAND` execution with direct `Command::new`, and tightening leading-dot domain cookies so parent-domain cookies require the parent domain to be explicitly allowed. The acceptance gap was fixed by adding the ignored real cmux plus real Crawl4AI loopback replay test. Security and context re-reviews then passed with no remaining blockers.

### D24: I6 output shaping is Rust-owned where limits affect contract stability

I6 adds `aget get` output shaping flags for `--format markdown|html|text|json`, CSS include/exclude selectors, `--wait-for`, `--max-chars`, and repeated `--extractor-option backend.key=value`. Rust forwards only backend-supported options to the Crawl4AI helper: format, selector, exclude selector, wait condition, and extractor options. Earlier WIP accepted `--only-main` and `--max-tokens` as metadata-only flags, but those were later removed before OpenCode integration because they were not enforced.

`--wait-for` is intentionally CSS-only in v1 for authenticated-session safety. The helper accepts `css:<selector>` and plain CSS selector strings, but rejects `js:` waits and obvious JavaScript function syntax before importing or running Crawl4AI. This prevents user-supplied wait conditions from executing JavaScript in a browser context that may include replayed local session state.

The Crawl4AI helper treats extractor options as an explicit allowlist, not an arbitrary escape hatch. V1 supports `crawl4ai.target_elements`, `crawl4ai.excluded_tags`, `crawl4ai.only_text`, `crawl4ai.word_count_threshold`, `crawl4ai.wait_until`, `crawl4ai.page_timeout`, `crawl4ai.wait_for_timeout`, `crawl4ai.delay_before_return_html`, and `crawl4ai.wait_for_images` when the installed Crawl4AI config constructor accepts the key. Unknown or unnamespaced keys fail before importing or running Crawl4AI, so dangerous options such as `crawl4ai.js_code` are not silently ignored or executed.

Character truncation is enforced after backend extraction in Rust using `.chars()` so results are deterministic across backends and cannot cut a UTF-8 scalar in half. After a successful backend parse, Rust sanitizes the retained backend stdout capture so untruncated content is not left in the run directory when `--max-chars` later shortens final content. For `--format json`, the CLI still returns a complete JSON response envelope; only the extracted `content` string is truncated.

For `--format text`, the Crawl4AI helper now prefers `result.extracted_content`, then derives plain text from `cleaned_html` or raw `html` with a stdlib HTML parser, and only falls back to markdown if no HTML content is available. This keeps real backend text output from silently being markdown in the common no-`extracted_content` case.

The stable output metadata now includes `output_options` plus expanded `limits` fields: `truncated_by`, `content_chars_before_truncation`, and `content_chars_after_truncation`. This preserves the I4/I5 session/sensitivity behavior while giving agents enough metadata to decide whether to refetch with larger limits or a narrower selector.

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

### D29: Response format and page content format are separate concepts

I8a keeps `--json` as a compatibility alias and adds `--envelope` as the clearer agent control-plane response flag. `--format` remains the fetched page content format. This means `aget --envelope get <url> --format markdown` should be read as: return structured status/error/artifact/session metadata to the caller, with markdown as the extracted page content. Human-facing `aget get <url>` still prints markdown directly by default.

The naming is still not perfect because `--format json` means JSON page content while `--json` remains accepted as a response-envelope alias. OpenCode integration should prefer `--envelope` and treat the structured response envelope as the behavior source of truth; a later API cleanup can still consider clearer content-format names if `--format json` proves confusing.

### D30: Agent-driven login bootstrap uses an explicit user-action loop

I8b implements a generic user-driven login bootstrap rather than a site-profile system. `aget session login start <name> --url <target>` opens a visible, `aget`-owned `agent-browser` profile/session at the target URL. The user completes the site's login manually in that browser. `aget session login finish <name>` then exports browser state, filters it to the URL-derived allowed domains, saves the scoped result as the normal local `<name>` session, and removes the raw temp state. `cancel` closes only the pending `aget` login session.

The flow deliberately does not script, collect, or store credentials, and it does not persist provider cookies by default. If the final relying-party session is insufficient without provider cookies, that should be treated as a product finding requiring explicit provider-session composition rather than silent broad state persistence. `aget get` no longer maps site-specific content markers to `requires_user_action`; agents must interpret fetched content and decide whether to start or retry a login/session flow.

### D31: I8b is blocked on manual real-site verification

Automated/local I8b validation passed after blocker fixes. Review found and fixes addressed HTTPS-only login URLs, duplicate pending starts, finish close failures, and pending cleanup ordering. Remaining blocker is the manual authorized HelloInterview e2e (`real_hellointerview_login_flow_fetches_paywalled_markdown`), which still needs local agent-browser/Crawl4AI setup plus explicit user go-ahead/login.

### D32: Manual agent-flow verification improved bootstrap handling but I8b remains blocked

This was the actual CLI flow an agent would use, not the ignored Rust test. `agent-browser` was not on PATH, so the run used a temporary wrapper at `/var/folders/3y/smwkyhkn7gdfw7rz8cnmd40r0000gn/T/opencode/aget-agent-browser-npx` around `npx -y agent-browser`; `npx -y agent-browser --version` returned `0.27.0`. Unauthenticated `aget --json get <HelloInterview URL> --format markdown --timeout 120` returned `extraction_failed` from Crawl4AI waiting for `body`, not `requires_user_action`.

`session login start hellointerview` initially failed because the bare profile `aget-hellointerview` was treated as a missing Chrome profile; the code now defaults to `AGET_HOME/tmp/agent-browser/aget-hellointerview`, and the real start/cancel smoke passes. `session login finish hellointerview` initially failed on real agent-browser state because cookie `expires` was a float; the parser now accepts floating expires in both login and Chrome import paths. After that fix, `session login finish hellointerview` succeeded and saved a local redacted session with 3 cookies and 1 storage origin.

A safe marker check in the opened agent-browser profile still found paywall/sign-in markers, so the browser was not actually authenticated during the forced continuation. Session-backed `aget --json get <URL> --session hellointerview --format markdown` still failed in Crawl4AI waiting for `body`; retry with `--wait-for html` and longer timeouts still failed waiting for `html`. I8b remains blocked: the login/start/finish mechanics are improved, but the final agent-flow acceptance has not passed.

### D33: Site-specific extraction behavior is out of scope for the binary

The review in `CLAUDE_REVIEW.md` identified that `aget get` had crossed the generic fetcher boundary by matching HelloInterview hostnames/content and returning a site-shaped login CTA. That behavior is out of scope for the binary even if HelloInterview remains a useful representative manual test site.

Decision: `aget` returns fetched content and generic extraction outcomes. It does not classify page content as a paywall/login wall for specific sites, and it does not name built-in sessions in retry advice. Calling agents or future skills decide whether a page's content means login is required and which caller-chosen session name to use.

Task tracking was updated to keep the partially implemented login bootstrap visible as `I8b`, add `I8b-followup` for removing site-specific coupling, add `I8a-followup` for response API stabilization before OpenCode integration, and add `I8d` for extractor/session-glue consolidation before `I9`.

### D34: I8a-followup stabilizes structured CLI output around one envelope

`--envelope` is now the preferred structured-output flag and `--json` remains a compatibility alias. All successful structured command output uses one agent-facing shape:

```json
{"ok": true, "command": "get", "data": {}, "warnings": [], "timing_ms": {"total": 0}}
```

Errors use the matching command-bearing shape:

```json
{"ok": false, "command": "get", "error": {"code": "extraction_failed", "message": "..."}}
```

Per-command payloads now live under `data`; cross-command control-plane fields stay at the top level. For `get`, backend/extraction warnings are promoted to top-level `warnings`, and the fetched page content plus artifacts, sessions, sensitivity, limits, and output options are under `data`. The on-disk run `metadata.json` format is unchanged for now because it is a run artifact rather than the CLI control-plane API.

Focused review found that parse-time errors were still Clap-formatted under `--json`/`--envelope`, and that command-bearing error output needed stronger tests. The fix now emits structured `usage_error` envelopes for parse/validation failures when structured output is requested, while preserving normal Clap help/version output and human-mode parse errors.

Verification updated CLI, get, and session tests to assert the envelope shape before OpenCode integration depends on it. Confidence: high for the CLI contract change, with the remaining product risk deferred to I10 around whether sensitive `get` content should be embedded inline in structured output by default.

