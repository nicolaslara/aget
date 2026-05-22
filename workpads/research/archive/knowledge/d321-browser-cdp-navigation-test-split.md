# D321: Browser CDP Navigation Test Split

## Decision

Keep `src/browser_cdp/tests/chrome/cdp_client/navigation.rs` as the browser CDP navigation test route and move the individual navigation scenarios into behavior-owned child modules:

- `navigation/same_document.rs`: same-document navigation does not wait for lifecycle or network-idle events.
- `navigation/errors.rs`: `Page.navigate` result-level `errorText` is surfaced.
- `navigation/lifecycle_timeout.rs`: lifecycle wait timeout reporting.
- `navigation/network_idle.rs`: network-idle route.
- `navigation/network_idle/load_event.rs`: network-idle load-event start coverage.
- `navigation/network_idle/reset_window.rs`: network-idle reset coverage.
- `navigation/network_idle/timeout.rs`: network-idle timeout coverage.

## Rationale

The split is mechanical and preserves the current mock-CDP coverage while reducing another large support file. It keeps the navigation test namespace close to the CDP client helpers and avoids changing runtime browser/CDP behavior.

## Validation

- `cargo test browser_cdp::tests::chrome::cdp_client::navigation --lib`
