# D353: CDP Dialog Auto Handling

## Decision

`aget` auto-accepts blocking `alert` and `beforeunload` JavaScript dialogs observed on the owned CDP transport while leaving `confirm` and `prompt` dialogs for explicit handling.

## Source Evidence

- `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, `cli/src/native/actions.rs`.
- agent-browser starts a dialog handler that calls `Page.handleJavaScriptDialog` with `accept: true` for `alert` and `beforeunload` events.
- agent-browser deliberately leaves `confirm` and `prompt` dialogs unhandled by the automatic path so the agent can decide explicitly.

## Implementation

- `src/browser_cdp/client/transport.rs` now parses incoming text and UTF-8 binary CDP frames through one helper before routing command responses.
- When a `Page.javascriptDialogOpening` event has type `alert` or `beforeunload`, the transport sends `Page.handleJavaScriptDialog` with `accept: true` and the event `sessionId` when present.
- The handled dialog event is consumed so active command waits continue until their real command response arrives.
- `confirm` and `prompt` dialog events remain visible to the command wait loop and are not auto-accepted.
- Existing malformed-frame skipping, binary-frame parsing, command timeout behavior, and WebSocket keepalive behavior are preserved.
- Added deterministic mock WebSocket coverage in `src/browser_cdp/tests/chrome/cdp_client/setup.rs` for alert auto-acceptance during an active `Browser.getVersion` command and for prompt non-acceptance.

## Validation

- `cargo test browser_cdp_auto_accepts_alert_dialogs_while_waiting_for_response`
- `cargo test browser_cdp_does_not_auto_accept_prompt_dialogs`
- `cargo test browser_cdp_skips_malformed_response_frames`
- `cargo test browser_cdp_accepts_binary_response_frames`
- `cargo test browser_cdp_sends_keepalive_ping_while_waiting_for_response`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Follow-Up

- This closes another narrow I19e CDP event-handling parity boundary. I19e still needs broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and fuller startup/error classification before final migration review.
