# D351: CDP WebSocket Size Limits

## Decision

`aget` disables incoming WebSocket message and frame size limits for owned CDP transport, matching agent-browser's CDP client.

## Source Evidence

- `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, `cli/src/native/cdp/client.rs`.
- agent-browser builds a `WebSocketConfig` with `max_message_size: None` and `max_frame_size: None`, then passes it to `connect_async_with_config`.
- Local `tungstenite` 0.29.0 defaults are bounded at 64 MiB per message and 16 MiB per frame unless overridden.

## Implementation

- `src/browser_cdp/client/transport.rs` now connects with `tungstenite::client::connect_with_config` and an owned `cdp_websocket_config`.
- The owned config uses `WebSocketConfig::default().max_message_size(None).max_frame_size(None)`.
- Existing CDP connect error wording and post-connect socket timeout setup are preserved.
- Added deterministic unit coverage in `src/browser_cdp/tests/chrome/cdp_client/setup.rs` for the owned config boundary.

## Validation

- `cargo test browser_cdp_websocket_config_allows_large_frames`
- `cargo test browser_cdp_accepts_binary_response_frames`
- `cargo test browser_cdp_skips_invalid_binary_response_frames`
- `cargo test browser_cdp_skips_malformed_response_frames`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Follow-Up

- This closes another narrow I19e transport parity boundary. I19e still needs broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and fuller startup/error classification before final migration review.
