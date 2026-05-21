# Knowledge Archive D151-D157: OAuth-Safe Authorization And Browser Import

### D151: I20 drafts OAuth-safe browser login and import decision tree

I20 now has a dedicated design artifact at `workpads/research/oauth-safe-browser-login-design.md`. The design keeps OAuth/password entry in a real user-controlled browser whenever possible, preserves explicit named `aget` sessions as the only authenticated fetch surface, and treats browser state plus authenticated artifacts as credential-equivalent local data.

Key decisions:

- OAuth default: unauthenticated fetch first, then user-approved real-browser profile import, then verification fetch before reporting a session usable.
- If verification still looks unauthenticated or caller-supplied predicates fail, ask the user to sign in through their normal browser and re-import; do not fall back silently to automation login.
- `aget session login start` remains a fallback for controlled or non-OAuth flows, not the first OAuth path.
- Dedicated `aget` profiles remain optional/deferred for OAuth because previous manual testing showed a fresh Chrome `--user-data-dir` profile did not persist the expected auth state, while importing an already logged-in normal Chrome profile worked.
- Browser choice is split into opening a browser for user login versus importing browser state. I21 should implement Chrome-family import first; I22 should verify Arc/Brave/Firefox/Safari separately before claiming support.
- Profile lock failures should return `requires_user_action` with wording that asks the user to quit the selected browser/profile and rerun the same import. `aget` must not close the user's browser.

The design also records deterministic mocked tests for I21 and a manual real-OAuth smoke recipe. Confidence is medium-high: the design is consistent with current `README.md`, `.cursor/skills/aget/SKILL.md`, `.opencode/tools/aget.ts`, and earlier D25/D26 auth ownership decisions, but I20 remains open until we decide whether executable mocked tests are part of I20 completion or entirely I21 implementation.

### D152: I20 completed as design scope and I21 owns executable OAuth workflow tests

I20 is complete as a design task. The acceptance wording now distinguishes design deliverables from implementation deliverables: I20 defines the deterministic mocked-site test cases and manual OAuth smoke recipe, while I21 owns the executable tests plus first-class orchestration command/API.

This split matches the existing task boundary because I21 already requires a first-class workflow, mocked tests for unauthenticated/import/locked/reimport/sensitive-envelope states, and documented user prompts. Keeping executable tests in I21 avoids adding pass-through tests before the orchestration surface exists.

Validation:

- `git diff --check`
- Workpad cross-check: I20 points to `workpads/research/oauth-safe-browser-login-design.md`; I21 remains in progress for implementation and executable test coverage.

Confidence: High for the task-boundary clarification. No runtime code changed.

### D153: I21 starts OAuth-safe authorization API

I21 now has a first API-level authorization workflow: `Aget::authorize_chrome_session`. The flow is generic and does not classify site-specific login/paywall content. It:

- runs an unauthenticated baseline fetch first;
- imports scoped Chrome state into the requested named local session;
- verifies the same URL with that saved session;
- evaluates caller-supplied `must_contain` and `must_not_contain` predicates against verification content;
- reports `verified` versus `verification_failed` without deleting the saved session;
- preserves `requires_user_action` import failures after the baseline fetch and does not save a session on failed import.

Coverage lives in `tests/aget_api.rs` and uses in-process test backends to prove baseline/import/verify order, session persistence, predicate failure reporting, sensitive verification fetches, and profile-lock/user-action propagation. This is intentionally API-first; CLI command/envelope shape and mocked-site command tests remain open I21 work.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --test aget_api`

Confidence: Medium-high. The new API boundary is deterministic and tested, but I21 is not complete until the CLI surface, mocked-site tests, and user-prompt documentation are added.

### D154: I21 adds OAuth-safe authorization CLI

I21 now exposes the API workflow through `aget session authorize`. The command supports Chrome first through `--browser chrome`, `--browser-profile <profile>`, and the compatibility alias `--chrome-profile <profile>`. It runs the generic OAuth-safe sequence in one CLI flow:

- unauthenticated baseline fetch;
- scoped Chrome profile import into the named local session;
- session-backed verification fetch;
- optional caller-supplied `--must-contain` and `--must-not-contain` predicates;
- `verified` or `verification_failed` state in a `session.authorize` envelope.

The JSON view intentionally differs from the raw API result: it omits inline `content` from both baseline and verification entries, includes `verification_sensitive: true` for session-backed fetches, and keeps authenticated output in artifacts or the explicit `--output` path. `requires_user_action` import failures remain structured errors and do not save the requested session.

Mocked CLI coverage in `tests/session_cli/authorize.rs` verifies a baseline fetch before import, Chrome import plus session-backed verification, verification predicate failure, profile-lock propagation, session persistence on successful import, no session persistence on locked import, and omission of sensitive inline content. The design doc now records the one-shot manual command and expected caller prompts for import approval, profile locks, failed verification, and verified sessions.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --lib parses_session_authorize`
- `cargo test --test aget_api authorize_chrome_session`
- `cargo test --test session_cli session_authorize`

Confidence: Medium-high at this checkpoint. The CLI and envelope behavior are deterministic and tested with local mocks; the remaining I21 gap was explicit re-import-after-user-login coverage, addressed in D155.

### D155: I21 completes OAuth-safe authorization workflow coverage

I21 is complete after adding explicit re-import-after-user-login CLI coverage. The mocked scenario runs `aget session authorize` twice against the same named session:

- first import contains stale scoped browser state, so the baseline fetch is unauthenticated and the verification fetch fails the caller's `--must-contain` predicate;
- the saved session remains present but not verified;
- second import simulates the user completing login in the approved browser/profile and re-running the same authorization command;
- the verification fetch replays the updated scoped cookie, returns `verified`, and replaces the stale saved session state.

This closes the deterministic test-plan gap left after D154. The command still does not classify site-specific gated states itself; it only reports predicate outcomes, artifacts, sensitivity, and structured user-action errors.

Validation:

- `cargo fmt --check`
- `cargo test --test session_cli session_authorize`

Confidence: High for I21's deterministic behavior. Real OAuth site smoke testing remains manual and should be run only with an explicitly authorized account/site.

### D156: I22 starts browser-neutral import surface conservatively

I22 now has a first browser-choice implementation slice. The public CLI accepts:

```bash
aget session import browser --browser chrome --browser-profile <profile> --name <session> --allow-domain <domain>...
```

This maps only `--browser chrome` to the verified owned Chrome/CDP import path. Other parsed browser families (`chromium`, `brave`, `edge`, `arc`, `firefox`, and `safari`) return `usage_error` before invoking a backend or reading local browser state. This is deliberate: current code has a proven Chrome import path, but source-specific profile discovery, executable selection, lock behavior, cookie/keychain behavior, and manual smoke coverage are not yet strong enough to claim broader browser import support.

The compatibility spelling remains:

```bash
aget session import chrome --chrome-profile <profile> --name <session> --allow-domain <domain>...
```

The support matrix and future acceptance requirements live in `workpads/research/browser-choice-session-import-design.md`. README, the project aget skill, and the OpenCode tool now prefer the browser-neutral Chrome command while still documenting the compatibility spelling.

Validation:

- `cargo fmt --check`
- `cargo test --lib parses_session_import_browser`
- `cargo test --test session_cli session_import_browser`
- `cargo test --test session_cli session_authorize`

Confidence: Medium-high. The new surface is intentionally narrow and has deterministic parser/mock coverage plus an ignored manual Chrome smoke. I22 remains open until the workpad records whether this Chrome-only browser-neutral surface is enough for the task or whether additional per-family research/implementation should be done before completion.

### D157: I22 completes with Chrome-only import support

I22 is complete. The task is satisfied by the conservative support matrix rather than by adding unverified browser imports:

- browser-choice terminology is public through `session import browser --browser <family>`, `--browser-profile`, and `--profile-path`;
- the only supported browser import is `--browser chrome`, which reuses the verified scoped Chrome/CDP import path;
- unsupported families are parsed and return structured `usage_error` before backend invocation;
- unsupported browser guidance is recorded in `workpads/research/browser-choice-session-import-design.md`;
- mocked tests cover supported Chrome import and unsupported browser rejection;
- an ignored/manual Chrome smoke exists for explicitly approved local profiles.

Adding Chromium, Brave, Edge, Arc, Firefox, or Safari import now would overclaim support. Each family needs source-specific profile discovery/path handling, lock classification, state export behavior, raw-state cleanup checks, and a manual smoke before becoming supported.

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the I22 scope as completed. Browser-choice import now has honest public terminology and a safe failure mode for unsupported browsers.
