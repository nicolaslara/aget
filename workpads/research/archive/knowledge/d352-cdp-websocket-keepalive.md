# D352: CDP WebSocket Keepalive

## Decision

`aget` sends periodic WebSocket Ping frames while owned CDP commands are waiting for responses.

## Source Evidence

- `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, `cli/src/native/cdp/client.rs`.
- agent-browser defines a 30-second WebSocket keepalive interval and spawns a task that sends empty `Message::Ping` frames to keep CDP connections alive through intermediate proxies.
- agent-browser also enables best-effort TCP keepalive, but its source comment identifies WebSocket Ping keepalive as the primary liveness mechanism.

## Implementation

- `src/browser_cdp/client/transport.rs` now sends an empty WebSocket Ping after the 30-second keepalive interval elapses while waiting for a CDP command response.
- Existing incoming Ping/Pong handling, malformed-frame skipping, command error handling, and timeout behavior are unchanged.
- The synchronous owned CDP client sends keepalives during active command waits rather than from a background task, matching the local blocking transport shape.
- Added deterministic mock WebSocket coverage in `src/browser_cdp/tests/chrome/cdp_client/setup.rs` where the server withholds the `Browser.getVersion` response until it observes the client keepalive Ping.

## Validation

- `cargo test browser_cdp_sends_keepalive_ping_while_waiting_for_response`
- `cargo test browser_cdp_accepts_binary_response_frames`
- `cargo test browser_cdp_skips_invalid_binary_response_frames`
- `cargo test browser_cdp_skips_malformed_response_frames`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Follow-Up

- This closes another narrow I19e CDP transport parity boundary. I19e still needs broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and fuller startup/error classification before final migration review.
