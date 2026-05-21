# OAuth-Safe Browser Login And Import Design

Status: I20 draft. This document defines the target decision tree and public vocabulary for I21/I22 implementation. It is intentionally generic: `aget` reports fetch, import, and verification outcomes; the calling agent or user interprets site-specific login, subscription, and account state.

## Goals

- Keep OAuth/password entry in a real user-controlled browser whenever possible.
- Make authenticated fetches use explicit named `aget` sessions, never ambient browser state.
- Treat browser state, imported cookies, localStorage, sessionStorage, authenticated HTML, and authenticated markdown as credential-equivalent local data.
- Verify that scoped imported state works before reporting the session as usable.
- Return `requires_user_action` for local user steps instead of closing, restarting, or disturbing the user's browser.

## Vocabulary

| Term | Meaning |
| --- | --- |
| Browser source | The local source to read state from, such as Chrome profile import, cmux surface, or a future browser-family-specific importer. |
| Browser family | A public choice such as `chrome`, `chromium`, `brave`, `arc`, `firefox`, or `safari`. I21 should implement Chrome-family import first and report unsupported families explicitly. |
| Browser profile | A user-visible browser profile name or directory name, such as `Default` or `Profile 1`. |
| Profile path | An explicit filesystem path to a profile/user-data directory approved by the user. |
| Aget session | A scoped local session persisted under `AGET_HOME`, filtered to explicit `--allow-domain` values, and replayed only when selected with `--session`. |
| Verification URL | The target URL used to confirm whether an imported session actually unlocks useful content. |
| Verification predicate | Optional generic checks such as `must_contain`, `must_not_contain`, `selector_exists`, or `status_success`. These are caller-provided and must not become built-in site heuristics. |

Existing CLI names remain valid for the current implementation:

```bash
aget session import chrome --chrome-profile <profile-or-path> --name <session> --allow-domain <domain>...
aget session login start <session> --url <login-or-target-url> [--profile <aget-profile-path>]
aget session login finish <session>
aget session login cancel <session>
```

I21 can add an orchestration command without removing the primitives. Proposed shape:

```bash
aget session authorize <session> \
  --url <verification-url> \
  --browser chrome \
  --browser-profile Default \
  --allow-domain <domain>... \
  [--must-contain <text>] \
  [--must-not-contain <text>] \
  [--output <path>] \
  [--envelope json]
```

I22 should decide whether the lower-level import command keeps `--chrome-profile` forever or gains browser-neutral aliases such as `--browser`, `--browser-profile`, and `--profile-path`.

## Default Decision Tree

1. Fetch the target URL without a session.
   - Use `aget get <url> --envelope json --inline-content auto`.
   - If content is sufficient, stop. Do not import browser state.
   - If the caller observes gated/login/subscription content, ask the user before reading any local browser state.
2. Try explicit real-browser import first.
   - Ask the user to approve the source browser/profile and allowed domains.
   - Run scoped import from the approved real browser profile.
   - Persist only filtered state; delete raw broad export.
   - If import fails with `requires_user_action`, relay the message and stop. Do not close the user's browser.
3. Verify the imported session.
   - Fetch the verification URL with `--session <session>`.
   - Use caller-supplied generic predicates when available.
   - If no predicates are supplied, report the fetched outcome and artifacts; the caller decides whether content is still gated.
   - Do not report the session as usable until verification succeeds or the caller explicitly accepts the result.
4. If imported state is missing or verification still looks unauthenticated, ask the user to sign in through their normal browser.
   - Tell the user which browser/profile and domains will be imported afterward.
   - Ask them to finish login/OAuth in the real browser and quit/unlock that profile if import requires it.
   - Re-import the same scoped domains.
   - Re-run verification before using the session.
5. Use `aget session login start` only as fallback.
   - It opens an `aget`-owned Chrome profile and is useful for controlled, non-OAuth, or test flows.
   - It must not be the default OAuth path because automated/fresh profiles can be rejected by providers and prior manual testing showed a fresh Chrome `--user-data-dir` did not persist the expected OAuth auth state.

## Dedicated Profile Decision

Dedicated `aget` profiles remain desirable for isolation but should be optional/deferred for OAuth-backed sites:

- Default for OAuth: import from a user-approved real browser profile after the user has logged in there.
- Optional fallback: `aget`-owned login profile for controlled test flows or non-OAuth sites.
- Deferred: making a dedicated `aget` profile the default OAuth path. It needs stronger evidence across OAuth providers, session persistence, and browser family behavior.

## Browser Choice Design

I21 should keep implementation scope narrow:

- Supported import source: Chrome-family import through the current owned Chrome/CDP path.
- Supported profile references: Chrome profile directory/display name and explicit profile path.
- Unsupported browser families: return `usage_error` or `backend_unavailable` with clear unsupported-family wording; do not silently treat Arc/Brave/Firefox/Safari as Chrome.

I22 should evaluate each family separately:

- Chrome/Chromium: likely first-class import and optional profile-path support.
- Brave/Arc: possible Chromium-family import only after profile layout, cookie decryption, lock handling, and path discovery are verified.
- Firefox/Safari: defer import unless there is a scoped, local, tested state export path with equivalent filtering and cleanup.

Opening a user browser for login and importing browser state are separate capabilities. A browser may be reasonable to open for user login while still unsupported for state import.

## Lock Handling And Error Text

Import must never close the user's browser. Profile lock or in-use failures should return `requires_user_action`.

Recommended message:

```text
The selected Chrome profile appears to be in use. Quit this browser/profile, then rerun the same import command.
```

Recommended JSON shape:

```json
{
  "ok": false,
  "schema_version": "aget.envelope.v1",
  "command": "session.import.chrome",
  "error": {
    "code": "requires_user_action",
    "message": "The selected Chrome profile appears to be in use. Quit this browser/profile, then rerun the same import command."
  }
}
```

Missing auth state after a readable import should also be actionable but should not imply a site-specific conclusion:

```text
Imported browser state contained no cookies or storage for the allowed domains. Sign in through the approved browser/profile, then retry import.
```

## Deterministic Test Plan

I21 should add mocked-site tests for the orchestration command or API without real credentials:

- `authorize_fetches_without_session_before_import`: first fetch uses no session and records non-sensitive output.
- `authorize_imports_real_profile_before_owned_login`: orchestration tries the approved import source before any `aget`-owned login flow.
- `authorize_locked_profile_returns_requires_user_action`: fake import profile lock returns `requires_user_action` and does not start or close a browser.
- `authorize_import_success_but_verification_predicate_fails`: imported state saves a session, verification fetch remains gated or fails a caller-supplied predicate, and the result asks for user login/re-import.
- `authorize_reimport_then_verification_succeeds`: first import lacks auth, second import contains scoped auth, verification succeeds, and the session is reported usable.
- `authorize_sensitive_verification_omits_inline_content_by_default`: session-backed verification uses `inline_content=auto` and returns artifact paths without embedding sensitive page content.

Existing mock-site fixtures can model this with public, gated, authenticated, expired, and logout pages. The tests should assert command order, envelope errors, session sensitivity, raw-state cleanup, and absence of password/OAuth prompt handling in `aget`.

## Manual OAuth Smoke Recipe

Use only an account and site the user is authorized to access. Do not paste passwords, MFA codes, or OAuth prompts into agent chat.

1. Pick a target URL, session name, approved browser/profile, and allowed domains.
2. Run an unauthenticated baseline fetch:

   ```bash
   cargo run --quiet -- --envelope json get "$URL" --content-format markdown --output /tmp/aget-oauth-baseline.md
   ```

3. If the content appears gated, ask the user to confirm importing local browser state.
4. Import the approved real browser profile:

   ```bash
   cargo run --quiet -- --envelope json session import chrome --chrome-profile Default --name "$SESSION" --allow-domain "$DOMAIN"
   ```

5. If the profile is locked, ask the user to quit that browser/profile and rerun the same import command.
6. Verify with the named session:

   ```bash
   cargo run --quiet -- --envelope json get "$URL" --session "$SESSION" --content-format markdown --inline-content never --output /tmp/aget-oauth-verified.md
   ```

7. If verification still appears gated, ask the user to sign in through their normal browser, then repeat import and verification.
8. Inspect and clean up as needed:

   ```bash
   cargo run --quiet -- session inspect "$SESSION"
   cargo run --quiet -- session delete "$SESSION"
   ```

Evidence to record locally, not in git: command timestamps, browser/profile choice, allowed domains, result codes, whether verification content was usable, and whether any `requires_user_action` lock/message occurred. Do not store screenshots, raw state, cookies, or private page content in workpads.

## Implementation Boundary

I20 does not require site-specific login detection. I21 should implement generic orchestration and tests. The calling agent remains responsible for interpreting content and choosing user-facing wording when no verification predicates are supplied.
