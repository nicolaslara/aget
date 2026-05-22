# OAuth-Safe Browser Login Design: Vocabulary And Flow

Status: I20 design, with the first I21 CLI/API implementation in place for Chrome profile import. This document defines the target decision tree and public vocabulary for I21/I22 implementation. It is intentionally generic: `aget` reports fetch, import, and verification outcomes; the calling agent or user interprets site-specific login, subscription, and account state.

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

Current I21 CLI behavior:

- `aget session authorize` supports `--browser chrome`, `--browser-profile`, and `--chrome-profile` as a compatibility alias.
- The command runs baseline fetch, Chrome import, and session-backed verification in one flow.
- JSON envelopes report `state: verified` or `state: verification_failed`.
- Baseline and verification entries omit inline `content` by default; authenticated verification content is available through artifacts or the optional `--output` path.
- `requires_user_action` import failures are returned as structured errors and do not save the requested session.

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
