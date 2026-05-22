# D260: Browser CDP Readiness Split

## Decision

Split browser CDP runtime readiness and evaluation helpers out of `src/browser_cdp/client/navigation.rs`.

## Boundary

- `src/browser_cdp/client/navigation.rs` keeps page navigation, blank-response navigation, lifecycle-event waits, network-idle tracking, navigation response/error handling, session message matching, and network request id extraction.
- `src/browser_cdp/client/navigation/readiness.rs` owns runtime readiness/evaluation helpers: selector waits, image completeness polling, full-page scan evaluation, rendered overlay cleanup evaluation, and string evaluation.

## Compatibility

The split is mechanical. Existing render/current-tab callers and tests continue to call the same `CdpClient` methods with the same browser-CDP internal visibility.

## Validation

- `cargo test browser_cdp::tests::chrome::cdp_client`
- `cargo test browser_cdp::tests::scripts`
- `cargo test aget_browser::tests::renders_current_tab_from_explicit_cdp_port_without_aget_facade_or_command_backend`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
