# D259: Agent-Browser Fallback Command Split

## Decision

Split command-compat browser fallback helpers out of `src/extraction/fallback_command.rs`.

## Boundary

- `src/extraction/fallback_command.rs` keeps fallback orchestration: temporary profile/session creation, state load, open, content extraction, close handling, warnings, and `BrowserFallbackResult` shaping.
- `src/extraction/fallback_command/command.rs` owns `AGET_AGENT_BROWSER_COMMAND` subprocess execution, private stdout/stderr capture, timeout handling, and fallback failure classification.
- `src/extraction/fallback_command/temp.rs` owns private temporary fallback profile/output files and cleanup.
- `src/extraction/fallback_command/text.rs` owns the simple compatibility HTML-to-text conversion used when agent-browser returns page HTML.

## Compatibility

The split is mechanical. `CommandBrowserFallbackBackend` still reaches the adapter through `run_agent_browser_fallback`, the fallback timeout from `BrowserFallbackRequest` still drives every command invocation, and close-error precedence remains unchanged.

## Validation

- `cargo test --test get_cli session::fallback`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
