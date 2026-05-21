# Knowledge Archive D173: AgetBrowser Direct Test And Naming Cleanup

### D173: Exercise `AgetBrowser` directly and rename active browser tests

The next I19j slice added a crate-local `AgetBrowser` unit test that cancels a pending login flow directly through the engine boundary, bypassing the public `Aget` facade and command-backed `agent-browser` compatibility path. This gives the browser engine direct no-Chrome coverage for pending metadata cleanup and tool-owned login profile removal.

Active mock-site browser fallback test names now use `aget_browser` naming, and the ignored Chrome-backed browser fallback tests use `AgetBrowserBackend` instead of the legacy `OwnedBrowserAutomationBackend` shim. Current agent-facing docs now call the login browser an `AgetBrowser`-managed profile. Stable public extractor/fallback labels still keep `aget-owned-*` strings in this slice to avoid changing envelope behavior during the mechanical refactor.

Validation:

- `cargo test aget_browser::tests::cancels_pending_login_without_aget_facade_or_command_backend`
- `cargo test --test mock_site_browser`
- `cargo test --test session_cli`
- `cargo fmt --check`

Confidence: High. This is a naming and test-boundary slice with no intended browser behavior change.
