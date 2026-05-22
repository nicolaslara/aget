# D322: AgetBrowser Engine Test Split

## Decision

Keep `src/aget_browser/tests.rs` as the `AgetBrowser` engine test route and move the existing scenarios into behavior-owned child modules:

- `tests/helpers.rs`: mock CDP request/reply, HTTP discovery responses, and attached-page CDP serving helpers.
- `tests/login.rs`: pending-login cancellation through `AgetBrowser`.
- `tests/discovery.rs`: explicit-port CDP endpoint discovery.
- `tests/current_tab.rs`: composed current-tab rendering from an explicit CDP port.
- `tests/attached_page.rs`: attached-page rendering through the engine seam.

## Rationale

The split is mechanical and preserves the direct engine coverage introduced for `AgetBrowser` while reducing another large support file. It keeps shared mock-CDP helpers local to the test namespace and does not change runtime browser behavior.

## Validation

- `cargo test aget_browser::tests --lib`
