# OAuth-Safe Browser Login Design: Browsers, Errors, Tests, And Smoke

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
8. To validate provider-session injection without recording credentials, keep a provider session such as `oauth` or `github`, then start the target login with that provider state explicitly injected:

   ```bash
   cargo run --quiet -- --envelope json session login start "$TARGET_SESSION" --url "$URL" --session "$PROVIDER_SESSION"
   cargo run --quiet -- --envelope json session login finish "$TARGET_SESSION"
   cargo run --quiet -- --envelope json get "$URL" --session "$TARGET_SESSION" --content-format markdown --inline-content never --output /tmp/aget-oauth-target.md
   ```

   Expected evidence is limited to command result codes, `injected_sessions` names in the start envelope, whether the user saw fewer provider prompts, and whether the target session later verifies. Do not record screenshots, prompts, raw state, cookies, or private content.
9. Inspect and clean up as needed:

   ```bash
   cargo run --quiet -- session inspect "$SESSION"
   cargo run --quiet -- session delete "$SESSION"
   ```

Evidence to record locally, not in git: command timestamps, browser/profile choice, allowed domains, result codes, whether verification content was usable, and whether any `requires_user_action` lock/message occurred. Do not store screenshots, raw state, cookies, or private page content in workpads.

Equivalent one-shot CLI command after the user approves local browser-state import:

```bash
cargo run --quiet -- --envelope json session authorize "$SESSION" \
  --url "$URL" \
  --browser chrome \
  --browser-profile Default \
  --allow-domain "$DOMAIN" \
  --must-contain "$EXPECTED_PRIVATE_MARKER" \
  --must-not-contain "$EXPECTED_GATED_MARKER" \
  --output /tmp/aget-oauth-verified.md
```

Expected prompt wording for callers:

- Before import: "This page still appears gated. May I import scoped local browser state from Chrome profile `<profile>` for `<domain>`?"
- On profile lock: "Quit the selected browser/profile, then rerun the same authorization command."
- On `verification_failed`: "The session was imported, but the verification checks did not pass. Complete login in the selected browser/profile, then rerun `aget session authorize` to re-import and verify."
- On `verified`: "The named `aget` session is verified for this URL. Use it explicitly with `aget get <url> --session <session>`."

## Implementation Boundary

I20 does not require site-specific login detection. I21 should implement generic orchestration and tests. The calling agent remains responsible for interpreting content and choosing user-facing wording when no verification predicates are supplied.
