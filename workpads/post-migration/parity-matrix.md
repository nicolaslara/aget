# Parity Matrix

This file is the ledger for PAR-001 through PAR-003.

Only track features `aget` implements. Do not copy upstream tests blindly; use
license-aware, behavior-level adapted tests when safer.

| Source project | Source path / commit | Upstream behavior | Aget feature | Local test path | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Crawl4AI | TBD | Public fetch | `aget get` static fetch | TBD | missing | Fill during PAR-001. |
| Crawl4AI | TBD | Rendered wait behavior | `--wait-for-selector` | TBD | missing | CSS-only waits; JavaScript waits remain rejected. |
| Crawl4AI | TBD | Markdown/html/text/json output | `--content-format` | TBD | missing | Cover implemented formats only. |
| Crawl4AI | TBD | Selectors and exclusions | `--selector`, `--exclude-selector`, backend options | TBD | missing | Include target/excluded elements after namespace decision. |
| agent-browser | TBD | Current-tab extraction | `aget current-tab` | TBD | missing | Consent-gated CDP path. |
| agent-browser | TBD | Login lifecycle | `session login start/finish/cancel` | TBD | missing | Include provider-session injection semantics. |
| agent-browser | TBD | Chrome/profile import | `session import chrome` | TBD | missing | Chrome/CDP verified paths only. |
| agent-browser | TBD | Cleanup and redaction | sessions, temp files, warnings | TBD | missing | Cover sensitive outputs and structured errors. |
