# D349: Invalid Binary CDP Frames

## Decision

`aget` skips invalid UTF-8 binary WebSocket frames from CDP endpoints and keeps waiting for the active command response.

## Source Evidence

- `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, `cli/src/native/cdp/client.rs`.
- agent-browser's CDP reader accepts `Message::Text`, accepts UTF-8 `Message::Binary`, and continues past binary frames that cannot be decoded as UTF-8.

## Implementation

- `src/browser_cdp/client/transport.rs` now returns `Ok(None)` for invalid UTF-8 binary frames, preserving the existing valid text and UTF-8 binary JSON parsing paths.
- Added deterministic mock WebSocket coverage in `src/browser_cdp/tests/chrome/cdp_client/setup.rs` where an invalid binary frame arrives before a valid `Browser.getVersion` response.

## Validation

- `cargo fmt --check`
- `cargo test browser_cdp_skips_invalid_binary_response_frames`
- `cargo test browser_cdp_accepts_binary_response_frames`
- `cargo test`
- `git diff --check`

## Follow-Up

- This locks a narrow CDP transport parity boundary. I19e still needs broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and fuller startup/error classification before final migration review.
