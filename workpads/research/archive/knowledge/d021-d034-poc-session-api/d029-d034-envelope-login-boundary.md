### D29: Response format and page content format are separate concepts

I8a keeps `--json` as a compatibility alias and adds `--envelope` as the clearer agent control-plane response flag. `--format` remains the fetched page content format. This means `aget --envelope get <url> --format markdown` should be read as: return structured status/error/artifact/session metadata to the caller, with markdown as the extracted page content. Human-facing `aget get <url>` still prints markdown directly by default.

The naming is still not perfect because `--format json` means JSON page content while `--json` remains accepted as a response-envelope alias. OpenCode integration should prefer `--envelope` and treat the structured response envelope as the behavior source of truth; a later API cleanup can still consider clearer content-format names if `--format json` proves confusing.

### D30: Agent-driven login bootstrap uses an explicit user-action loop

I8b implements a generic user-driven login bootstrap rather than a site-profile system. `aget session login start <name> --url <target>` opens a visible, `aget`-owned `agent-browser` profile/session at the target URL. The user completes the site's login manually in that browser. `aget session login finish <name>` then exports browser state, filters it to the URL-derived allowed domains, saves the scoped result as the normal local `<name>` session, and removes the raw temp state. `cancel` closes only the pending `aget` login session.

The flow deliberately does not script, collect, or store credentials, and it does not persist provider cookies by default. If the final relying-party session is insufficient without provider cookies, that should be treated as a product finding requiring explicit provider-session composition rather than silent broad state persistence. `aget get` no longer maps site-specific content markers to `requires_user_action`; agents must interpret fetched content and decide whether to start or retry a login/session flow.

### D31: I8b is blocked on manual real-site verification

Automated/local I8b validation passed after blocker fixes. Review found and fixes addressed HTTPS-only login URLs, duplicate pending starts, finish close failures, and pending cleanup ordering. Remaining blocker is the manual authorized HelloInterview e2e (`real_hellointerview_login_flow_fetches_paywalled_markdown`), which still needs local agent-browser/Crawl4AI setup plus explicit user go-ahead/login.

### D32: Manual agent-flow verification improved bootstrap handling but I8b remains blocked

This was the actual CLI flow an agent would use, not the ignored Rust test. `agent-browser` was not on PATH, so the run used a temporary wrapper at `/var/folders/3y/smwkyhkn7gdfw7rz8cnmd40r0000gn/T/opencode/aget-agent-browser-npx` around `npx -y agent-browser`; `npx -y agent-browser --version` returned `0.27.0`. Unauthenticated `aget --json get <HelloInterview URL> --format markdown --timeout 120` returned `extraction_failed` from Crawl4AI waiting for `body`, not `requires_user_action`.

`session login start hellointerview` initially failed because the bare profile `aget-hellointerview` was treated as a missing Chrome profile; the code now defaults to `AGET_HOME/tmp/agent-browser/aget-hellointerview`, and the real start/cancel smoke passes. `session login finish hellointerview` initially failed on real agent-browser state because cookie `expires` was a float; the parser now accepts floating expires in both login and Chrome import paths. After that fix, `session login finish hellointerview` succeeded and saved a local redacted session with 3 cookies and 1 storage origin.

A safe marker check in the opened agent-browser profile still found paywall/sign-in markers, so the browser was not actually authenticated during the forced continuation. Session-backed `aget --json get <URL> --session hellointerview --format markdown` still failed in Crawl4AI waiting for `body`; retry with `--wait-for html` and longer timeouts still failed waiting for `html`. I8b remains blocked: the login/start/finish mechanics are improved, but the final agent-flow acceptance has not passed.

### D33: Site-specific extraction behavior is out of scope for the binary

The review in `CLAUDE_REVIEW.md` identified that `aget get` had crossed the generic fetcher boundary by matching HelloInterview hostnames/content and returning a site-shaped login CTA. That behavior is out of scope for the binary even if HelloInterview remains a useful representative manual test site.

Decision: `aget` returns fetched content and generic extraction outcomes. It does not classify page content as a paywall/login wall for specific sites, and it does not name built-in sessions in retry advice. Calling agents or future skills decide whether a page's content means login is required and which caller-chosen session name to use.

Task tracking was updated to keep the partially implemented login bootstrap visible as `I8b`, add `I8b-followup` for removing site-specific coupling, add `I8a-followup` for response API stabilization before OpenCode integration, and add `I8d` for extractor/session-glue consolidation before `I9`.

### D34: I8a-followup stabilizes structured CLI output around one envelope

`--envelope` is now the preferred structured-output flag and `--json` remains a compatibility alias. All successful structured command output uses one agent-facing shape:

```json
{"ok": true, "command": "get", "data": {}, "warnings": [], "timing_ms": {"total": 0}}
```

Errors use the matching command-bearing shape:

```json
{"ok": false, "command": "get", "error": {"code": "extraction_failed", "message": "..."}}
```

Per-command payloads now live under `data`; cross-command control-plane fields stay at the top level. For `get`, backend/extraction warnings are promoted to top-level `warnings`, and the fetched page content plus artifacts, sessions, sensitivity, limits, and output options are under `data`. The on-disk run `metadata.json` format is unchanged for now because it is a run artifact rather than the CLI control-plane API.

Focused review found that parse-time errors were still Clap-formatted under `--json`/`--envelope`, and that command-bearing error output needed stronger tests. The fix now emits structured `usage_error` envelopes for parse/validation failures when structured output is requested, while preserving normal Clap help/version output and human-mode parse errors.

Verification updated CLI, get, and session tests to assert the envelope shape before OpenCode integration depends on it. Confidence: high for the CLI contract change, with the remaining product risk deferred to I10 around whether sensitive `get` content should be embedded inline in structured output by default.

