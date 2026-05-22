# D346: Binary CDP Response Frames

## Decision

`aget` keeps accepting UTF-8 binary WebSocket frames from CDP endpoints as normal JSON responses.

## Source Evidence

- `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, `cli/src/native/cdp/client.rs`.
- agent-browser's CDP reader accepts both `Message::Text` and UTF-8 `Message::Binary` frames before parsing the CDP JSON message, skipping invalid binary UTF-8 frames.

## Implementation

- `src/browser_cdp/client/transport.rs` already accepted binary frames by converting the payload to UTF-8 and parsing the resulting JSON.
- Added deterministic mock WebSocket coverage in `src/browser_cdp/tests/chrome/cdp_client/setup.rs`.
- Added shared test helper `reply_ok_binary` in `src/browser_cdp/tests/chrome.rs`.

## Validation

- `cargo test browser_cdp_accepts_binary_response_frames`

## Follow-Up

- This locks a narrow transport parity boundary. I19e still needs broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and fuller startup/error classification before final migration review.
