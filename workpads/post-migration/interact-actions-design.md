# Safe Generic Interact / Actions Design

## Decision

`aget interact` should be a local browser-action command for explicitly
authorized, bounded flows. It is a CLI feature, not a server, MCP surface, or
site-specific assistant.

The first implementation should be plan-file driven:

```bash
aget interact "https://example.com/app" \
  --actions actions.json \
  --allow-actions \
  --capture-screenshot \
  --output /tmp/aget-interact.md

aget interact current-tab \
  --cdp-port 9222 \
  --actions actions.json \
  --allow-actions \
  --allow-private-content \
  --capture-screenshot \
  --output /tmp/current-tab-interact.md
```

`--actions` is required and points to a local JSON file. Avoiding long inline
action JSON keeps plans reviewable and reduces accidental shell-history leaks.
`--allow-actions` is required for every run because the command can mutate page
state, trigger navigation, or capture private content.

## Scope

Supported action types for the first implementation:

- `wait`: wait for a selector, load state, network idle, or fixed duration.
- `click`: click one CSS selector.
- `type`: type text into one input-like selector.
- `select`: choose one option in one select-like control.
- `submit`: submit one form or click one submit control.
- `capture`: save a named screenshot and/or HTML snapshot.
- `extract`: run the existing extraction pipeline against the current page DOM.

Out of scope for the first implementation:

- Arbitrary JavaScript actions or predicates.
- Drag/drop, file upload, clipboard, downloads, tabs/windows, permissions,
  geolocation, device emulation, HAR capture, or credential vaults.
- CAPTCHA solving, paywall bypass, login-wall detection, site-specific retry
  advice, or built-in names for particular services.
- Remote execution, hosted service APIs, daemon mode, or MCP.

## Action File

The action file is JSON. Unknown top-level fields should be rejected until a
schema version needs extension.

```json
{
  "schema_version": "aget.actions.v1",
  "defaults": {
    "action_timeout_ms": 5000,
    "navigation_timeout_ms": 15000
  },
  "actions": [
    {
      "type": "wait",
      "selector": "main",
      "timeout_ms": 5000
    },
    {
      "type": "click",
      "selector": "button[aria-label='Search']"
    },
    {
      "type": "type",
      "selector": "input[name='q']",
      "text": "install instructions"
    },
    {
      "type": "submit",
      "selector": "form#search",
      "confirm": true
    },
    {
      "type": "capture",
      "name": "after-search",
      "screenshot": true,
      "html": true
    },
    {
      "type": "extract",
      "selector": "main",
      "content_format": "markdown"
    }
  ]
}
```

Common action fields:

| Field | Meaning |
| --- | --- |
| `type` | Required action type. |
| `name` | Optional stable caller label. Required for persisted `capture` files. |
| `selector` | CSS selector. Commands must fail if it matches zero elements or more than one element unless the action opts into `match: "first"`. |
| `timeout_ms` | Per-action timeout. Defaults to the plan default. |
| `sensitive` | Redacts user-supplied values and marks resulting artifacts sensitive. |

Selector ambiguity should be treated as a safety issue. Defaulting to a random
or first match can click the wrong control. `match: "first"` is allowed only for
read-like actions such as `wait`, `capture`, and `extract`; mutation actions
should require a unique selector.

## Consent And Private Content

Consent is explicit and layered:

| Condition | Required flag |
| --- | --- |
| Any interact run | `--allow-actions` |
| Current-tab action run | `--allow-private-content` |
| Named session or imported browser state | `--allow-private-content` |
| Screenshot capture | Capture action with `"screenshot": true` plus `--capture-screenshot`; also `--allow-private-content` when sensitive |
| Sensitive typing | `--allow-sensitive-input` |
| Submit action | per-action `"confirm": true` and `--allow-submit` |

`current-tab` remains the most sensitive path because it acts on whatever page
the user has already opened. It should continue to require a user-provided CDP
port and `--allow-private-content`.

Session-backed interact runs inherit the existing replay-scope rules. Provider
sessions remain login-bootstrap inputs for `session login start`; they must not
be treated as permission to automate unrelated target pages.

## Sensitive Input

Typing into `input[type=password]`, `[autocomplete=current-password]`,
`[autocomplete=one-time-code]`, or fields whose accessible name looks
credential-like should be rejected by default with `unsafe_action`.

Sensitive typing is allowed only when all are true:

- The action has `"sensitive": true`.
- The command includes `--allow-sensitive-input`.
- The target selector is unique.
- The audit log redacts the text value.

The audit log may record length, input source kind, and a stable hash of the
typed bytes, but never the literal sensitive text. The action file itself is
caller-owned input; copied `aget` artifacts must use the redacted form.

## Confirmation Boundaries

The command should stop with `requires_confirmation` before crossing boundaries
that can mutate remote state or widen data access unexpectedly:

- `submit` without `"confirm": true` or without `--allow-submit`.
- Cross-origin navigation after a mutation action unless the destination origin
  is allowed by `--allow-domain`.
- New tab/window, popup, file chooser, permission prompt, or download.
- Browser dialog prompts other than non-input alerts.
- Navigation to a URL scheme outside `http`, `https`, `file`, and `raw`.

Alert dialogs can be dismissed only when they do not ask for input and the
action result records the dialog text. Prompt dialogs must stop.

## Timeouts And Bounds

Default limits:

- Maximum 50 actions per plan.
- Default action timeout: 5 seconds.
- Default navigation timeout: 15 seconds.
- Default total timeout: existing command `--timeout` or 60 seconds.
- Maximum capture name length: 80 ASCII filename-safe characters.

Each action result records `started_at`, `elapsed_ms`, and terminal status.
Timeouts report the failing action index and do not retry automatically.

## Audit And Redaction

Every interact run writes audit artifacts, even on failure:

```text
AGET_HOME/runs/<run-id>/
  metadata.json
  actions-request.json
  actions-result.json
  content.md
  captures/
    after-search.html
    after-search.png
  debug-trace.json
```

`actions-request.json` is a normalized, redacted copy of the action plan.
`actions-result.json` records action statuses, timings, warnings, navigation
transitions, artifact paths, and errors. It must not include cookies,
localStorage values, request headers, response headers, extracted page content,
or unredacted sensitive typed values.

`debug-trace.json` is written only when the command also requests
`--capture-trace`; action audit files are always written.

URL redaction follows the debug artifact policy: sensitive sources redact query
and fragment details in trace/audit artifacts.

## Envelope Shape

Success uses the existing envelope shell:

```json
{
  "ok": true,
  "schema_version": "aget.envelope.v1",
  "command": "interact",
  "data": {
    "run_id": "20260526T120000Z-abc123",
    "source": {
      "kind": "url",
      "url": "https://example.com/app",
      "sessions": [],
      "sensitive": false
    },
    "initial_url": "https://example.com/app",
    "final_url": "https://example.com/search?q=install",
    "actions": {
      "total": 6,
      "succeeded": 6,
      "failed_index": null,
      "result_path": ".../actions-result.json"
    },
    "content_format": "markdown",
    "content": "# Results\n...",
    "artifacts": {
      "content": ".../content.md",
      "metadata": ".../metadata.json",
      "actions_request": ".../actions-request.json",
      "actions_result": ".../actions-result.json",
      "captures": [
        {
          "name": "after-search",
          "path": ".../captures/after-search.png",
          "media_type": "image/png",
          "sensitive": false
        }
      ]
    },
    "sensitive": false
  },
  "warnings": [],
  "timing_ms": {"total": 1234}
}
```

If the last action is `extract`, the command may populate `data.content` using
the same inline-content rules as `get`. Sensitive runs should default to writing
content to artifacts and omitting inline content.

## Failure Semantics

Failures preserve partial audit artifacts. The current shared `ErrorResponse`
shape has no `data` field, so `ACT-003` must either extend it with optional
command data or add an interact-specific failure printer that keeps the same
top-level error shell plus partial run metadata:

```json
{
  "ok": false,
  "schema_version": "aget.envelope.v1",
  "command": "interact",
  "error": {
    "code": "selector_not_found",
    "message": "action 2 click selector matched no elements"
  },
  "data": {
    "run_id": "20260526T120000Z-abc123",
    "failed_action_index": 2,
    "artifacts": {
      "metadata": ".../metadata.json",
      "actions_request": ".../actions-request.json",
      "actions_result": ".../actions-result.json"
    }
  }
}
```

New stable action-level codes should be introduced only if the existing error
taxonomy cannot represent the outcome:

- `action_timeout`
- `selector_not_found`
- `selector_ambiguous`
- `requires_confirmation`
- `unsafe_action`
- `navigation_blocked`
- `action_not_supported`

Existing `backend_unavailable`, `extraction_failed`, `usage_error`, and
`io_error` remain appropriate for browser availability, extraction failures,
bad CLI input, and filesystem failures.

Partial success is explicit. The command never pretends a failed plan completed.
Artifacts created before the failure remain internal run artifacts and are
managed by `aget artifacts delete/prune`.

## Implementation Slices

Recommended follow-up tasks:

1. `ACT-002`: Add the action schema/parser, validation, redacted plan writer,
   and deterministic unit tests.
2. `ACT-003`: Add the CLI envelope, run-artifact layout, and fake-browser
   action executor seam without real CDP actions.
3. `ACT-004`: Implement CDP `wait`, `click`, `type`, `select`, and `submit`
   against deterministic local pages.
4. `ACT-005`: Implement `capture` and `extract` actions using existing debug
   artifact and extraction pipelines.
5. `ACT-006`: Update README, `skills/aget/SKILL.md`, and OpenCode wrappers
   after the CLI command exists.

## Test Strategy

Schema and parser tests:

- Reject unknown action types and unknown top-level fields.
- Reject missing selectors, invalid CSS selectors, duplicate capture names, and
  plans over the action count limit.
- Reject sensitive typing without `--allow-sensitive-input`.
- Reject submit without both per-action confirmation and `--allow-submit`.

Audit and redaction tests:

- Redacted request artifacts never contain sensitive typed text.
- Sensitive source URLs redact query and fragment details in audit/trace files.
- Failure metadata includes failing action index and partial artifacts.

Fake-browser executor tests:

- Unit-test action sequencing, timeout handling, confirmation boundaries, and
  failure codes without launching Chrome.
- Ensure click/type/select/submit are called with unique selector contracts.

Deterministic local browser tests:

- Serve local HTML pages that exercise wait, click, type, select, submit,
  same-origin navigation, cross-origin rejection, capture, and extract.
- Use mock CDP current-tab tests for attach/capture behavior.
- Do not use external services, real credentials, real accounts, or
  site-specific fixtures.

Lifecycle tests:

- `artifacts inspect` reports action request/result and capture files.
- `artifacts delete/prune` remove internal action artifacts and preserve
  caller-owned external `--output` files.

## Review Bar

Before implementation starts, review the design for:

- Whether consent flags are explicit enough for private browser state.
- Whether mutation boundaries avoid accidental submit/cross-origin behavior.
- Whether audit artifacts are useful without leaking credentials or page
  content by default.
- Whether the fake-browser seam gives deterministic CLI-level coverage before
  real CDP action work.
