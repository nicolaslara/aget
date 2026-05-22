# D350: Malformed CDP Frames

## Decision

`aget` skips malformed text and UTF-8 binary WebSocket frames from CDP endpoints and keeps waiting for the active command response.

## Source Evidence

- `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, `cli/src/native/cdp/client.rs`.
- agent-browser's CDP reader accepts text frames, accepts UTF-8 binary frames, skips invalid UTF-8 binary frames, and continues past messages that do not parse as its typed CDP message shape.

## Implementation

- `src/browser_cdp/client/transport.rs` now returns `Ok(None)` for malformed text and UTF-8 binary JSON frames.
- Valid text and UTF-8 binary JSON frames keep the existing response path.
- Valid CDP command error objects still become stable extraction failures in `wait_for_response`.
- Added deterministic mock WebSocket coverage in `src/browser_cdp/tests/chrome/cdp_client/setup.rs` where malformed text and binary frames arrive before a valid `Browser.getVersion` response.

## Validation

- `cargo test browser_cdp_skips_malformed_response_frames`
- `cargo test browser_cdp_skips_invalid_binary_response_frames`
- `cargo test browser_cdp_accepts_binary_response_frames`
- `cargo test browser_cdp_navigation_error_text_is_reported`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Follow-Up

- This locks another narrow CDP transport parity boundary. I19e still needs broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and fuller startup/error classification before final migration review.
