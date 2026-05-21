# Browser-Choice Session Import Design

Status: I22 draft implementation slice. This document records the current public terminology and support matrix for browser-session import surfaces. It is intentionally conservative: unsupported browsers fail explicitly instead of sharing Chrome code paths without source-specific verification.

## Public Terms

| Term | Meaning |
| --- | --- |
| Browser family | A user-facing choice such as `chrome`, `chromium`, `brave`, `edge`, `arc`, `firefox`, or `safari`. |
| Browser profile | A browser-visible profile name or profile directory label, such as `Default` or `Profile 1`. |
| Profile path | An explicit filesystem path approved by the user. This is for advanced/manual cases where profile discovery is not enough or should not be guessed. |
| Browser import | Reading local browser cookies/storage, filtering it to explicit `--allow-domain` scopes, and saving it as a named `aget` session. |
| Login/open flow | Opening a browser for the user to sign in. This is separate from importing browser state and may have different browser support. |

## CLI Surface

Preferred import spelling:

```bash
aget session import browser \
  --browser chrome \
  --browser-profile Default \
  --name <session> \
  --allow-domain <domain>...
```

Advanced explicit path spelling:

```bash
aget session import browser \
  --browser chrome \
  --profile-path /path/to/profile-or-user-data-dir \
  --name <session> \
  --allow-domain <domain>...
```

Compatibility spelling kept for the existing Chrome path:

```bash
aget session import chrome --chrome-profile Default --name <session> --allow-domain <domain>...
```

## Support Matrix

| Browser family | Open for user login | Import browser state | Current behavior |
| --- | --- | --- | --- |
| `chrome` | Supported through the existing `aget`-owned Chrome login fallback. | Supported through owned Chrome/CDP import and the compatibility command adapter when selected. | `session import browser --browser chrome` maps to the verified Chrome import path. |
| `chromium` | Possible through the same local Chrome/CDP launch machinery when a compatible executable is selected. | Not claimed through the public browser-neutral import command yet. | Parsed but returns `usage_error` until source-specific discovery, lock handling, and manual smoke coverage are added. |
| `brave` | Possible through Chromium-family CDP, but not product-supported yet. | Not claimed yet. | Parsed but returns `usage_error`; use Chrome import if the session is available in Chrome, or wait for Brave-specific verification. |
| `edge` | Possible in principle through Chromium-family CDP. | Not claimed yet. | Parsed but returns `usage_error`. |
| `arc` | Possible only after Arc profile layout and state export behavior are researched. | Not claimed yet. | Parsed but returns `usage_error`. |
| `firefox` | Deferred; Firefox profile locking and WebDriver/CDP/BiDi behavior need a separate implementation. | Unsupported. | Parsed but returns `usage_error`. |
| `safari` | Deferred; Safari state import is platform-specific and not part of the current local CDP path. | Unsupported. | Parsed but returns `usage_error`. |

## Safety Rules

- Import must always require explicit `--allow-domain` values.
- Unsupported browsers must fail before invoking any backend command or reading local browser state.
- Browser profile locks must remain `requires_user_action`; `aget` must not close the user's browser.
- Browser source names in docs and envelopes must not imply broad support before deterministic tests and a manual smoke path exist.
- `session login start` remains an explicit fallback for controlled/non-OAuth flows, not the default OAuth path.

## Tests

Current I22 coverage:

- Parser coverage for `session import browser --browser chrome --browser-profile ...`.
- Parser coverage for unsupported browser values plus `--profile-path`.
- Mocked CLI coverage that Chrome browser import saves filtered state through the same scoped path as `session import chrome`.
- Mocked CLI coverage that unsupported browser import returns `usage_error`, does not invoke the backend, and does not save a session.
- Ignored/manual Chrome import smoke through `AGET_REAL_BROWSER_PROFILE` and `AGET_REAL_BROWSER_DOMAIN`.

Future support for `chromium`, `brave`, `edge`, or `arc` should add per-family profile discovery/path tests, lock classification tests, raw-state cleanup checks, and one ignored/manual local smoke before the browser is listed as import-supported.
