# D365: Frame-Tree Storage Export

## Decision

Owned browser state export now augments explicit storage-origin probing with allowed origins discovered from `Page.getFrameTree`.

## Source Evidence

- `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`.
- `cli/src/native/state.rs` collects origins from the current frame tree before saving storage state.
- The same file then probes remaining origins through a temporary target and collects localStorage/sessionStorage.

## Implementation

- `CdpClient::export_state` keeps the existing explicit `allowed_domains` candidate origins.
- Before enabling blank-response storage probing, owned export asks `Page.getFrameTree` for current page and child-frame URLs.
- Frame URLs are converted to origins only when their host matches the explicit allow-domain scope, including subdomains.
- Disallowed frame origins and non-origin URLs such as `about:blank` are ignored.
- Cookie export, storage loading, blank-response navigation, and existing explicit origin probing remain unchanged.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo test browser_cdp_export_state_collects_allowed_frame_storage_origins`
- `cargo test frame_storage_candidate_origins_keep_explicit_allow_domain_scope`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

## Follow-Up

- I19e remains open for broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and still-fuller startup/error classification.
