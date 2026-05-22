### D21: I3 empty-session Crawl4AI fetch is implemented

I3 is complete: `aget get` defaults to empty Playwright storage state, shells out to the local Crawl4AI helper, and writes run artifacts as `content.md` and `metadata.json`. The stable JSON contract includes `extractor: "crawl4ai"`, and `scripts/demo_real_cli.sh` is the reusable real CLI demo path showing the default-backend flow.

Real demo runs surfaced Crawl4AI stdout progress logs; the helper now handles this by parsing the final JSON line and redirecting Crawl4AI stdout to stderr so progress noise no longer breaks extraction.

Verification covered fake-backend and local-server tests for success, timeout, error paths, temp cleanup, and noisy stderr; `cargo test`, `cargo fmt --check`, and `git diff --check` passed, and Oracle blocker review was PASS.

Future work stays split as I4 for session replay and I6 for output shaping.

### D22: I4 local cookie replay path is implemented

I4 adds explicit one-session replay for `aget get <url> --session <name>`. The Rust CLI loads the named local session, composes it into a temporary Playwright storage-state file, and passes that file through the existing Crawl4AI helper. Empty fetches still compose `{"cookies": [], "origins": []}` and return `sessions: []`, `sensitive: false`.

Verification now covers two layers. The normal fake-backend integration test asserts that the generated Playwright state contains the hand-written cookie fixture and that command output plus `metadata.json` record `sessions: ["local"]` and `sensitive: true`. An ignored real-backend integration test, `real_crawl4ai_replays_named_session_cookie`, starts a local cookie echo server and verifies default Crawl4AI behavior: empty state does not send the synthetic `sid` cookie, while `--session local` sends `sid=secret-cookie` through the rendered browser request.

The real test also surfaced two useful backend behaviors. Crawl4AI injects its own `cookiesEnabled=true` cookie even with empty user state, so tests should assert absence of the selected session cookie rather than zero Cookie header. Crawl4AI 0.8.6 flags tiny local pages as `minimal_text`, so local replay fixtures need enough visible text to pass structural checks.

Post-implementation review found and fixed three hardening points: session-backed fetches are marked sensitive whenever any session is selected, session names reject path-like values before loading/saving/deleting, and nonzero backend exits cannot be converted into successful `aget` results even if stdout contains `{"ok": true}`.

### D23: I5 optional cmux cookie import is implemented

I5 adds `aget session import cmux --surface <surface> --name <name> --domain <domain>...`. The import path shells out to cmux's cookie API through an optional `AGET_CMUX_COMMAND` override, stores the result as `SessionSource::Cmux { surface }`, marks the session sensitive, and persists only cookies plus explicit `allowed_cookie_domains`. It does not call `cmux browser state save` and does not use cmux URL scoping.

The adapter treats cmux's domain filter as coarse because source research confirmed cmux filters domains by substring. `aget` therefore post-filters every returned cookie by exact/suffix domain rules before persistence. Host-only `example.com` and domain cookie `.example.com` are not accepted when only `docs.example.com` is allowed; the parent domain must be explicitly allowed before broader parent-domain cookies are imported. Because cmux's cookie JSON does not expose `HttpOnly`, imported cookies default to `http_only: true` to avoid weakening browser cookie protections during replay.

Verification covers fake and optional real paths. `tests/session_cli.rs` uses a fake cmux CLI to prove repeated `--domain`, sensitive session persistence, redacted inspect output, disallowed-cookie filtering, and missing-backend `backend_unavailable`. The ignored `real_cmux_imports_loopback_cookie` test uses a user-provided disposable cmux surface, imports a loopback cookie, and replays it through `aget get --session` with a fake Crawl4AI backend that reads the generated Playwright state and sends the cookie to a loopback echo server. The ignored `real_cmux_import_replays_loopback_cookie_through_crawl4ai` test uses the same loopback-only import setup and replays the imported session through the real Crawl4AI backend.

Verification passed: `cargo test --test session_cli`, `cargo test domain_matching_rejects_substring_only_matches`, `cargo test`, `cargo fmt --check`, `cargo check`, `git diff --check`, and a manual fake-cmux CLI smoke. `lsp_diagnostics` remains blocked for macro-heavy Rust files by the local rust-analyzer/proc-macro API mismatch (`proc-macro server's api version (6) is newer than rust-analyzer's (5)`), while cargo compile/tests are clean.

Post-implementation review initially found two blockers and one acceptance gap. The blockers were fixed by defaulting unknown cmux cookies to `http_only: true`, replacing shell-interpreted `AGET_CMUX_COMMAND` execution with direct `Command::new`, and tightening leading-dot domain cookies so parent-domain cookies require the parent domain to be explicitly allowed. The acceptance gap was fixed by adding the ignored real cmux plus real Crawl4AI loopback replay test. Security and context re-reviews then passed with no remaining blockers.

### D24: I6 output shaping is Rust-owned where limits affect contract stability

I6 adds `aget get` output shaping flags for `--format markdown|html|text|json`, CSS include/exclude selectors, `--wait-for`, `--max-chars`, and repeated `--extractor-option backend.key=value`. Rust forwards only backend-supported options to the Crawl4AI helper: format, selector, exclude selector, wait condition, and extractor options. Earlier WIP accepted `--only-main` and `--max-tokens` as metadata-only flags, but those were later removed before OpenCode integration because they were not enforced.

`--wait-for` is intentionally CSS-only in v1 for authenticated-session safety. The helper accepts `css:<selector>` and plain CSS selector strings, but rejects `js:` waits and obvious JavaScript function syntax before importing or running Crawl4AI. This prevents user-supplied wait conditions from executing JavaScript in a browser context that may include replayed local session state.

The Crawl4AI helper treats extractor options as an explicit allowlist, not an arbitrary escape hatch. V1 supports `crawl4ai.target_elements`, `crawl4ai.excluded_tags`, `crawl4ai.only_text`, `crawl4ai.word_count_threshold`, `crawl4ai.wait_until`, `crawl4ai.page_timeout`, `crawl4ai.wait_for_timeout`, `crawl4ai.delay_before_return_html`, and `crawl4ai.wait_for_images` when the installed Crawl4AI config constructor accepts the key. Unknown or unnamespaced keys fail before importing or running Crawl4AI, so dangerous options such as `crawl4ai.js_code` are not silently ignored or executed.

Character truncation is enforced after backend extraction in Rust using `.chars()` so results are deterministic across backends and cannot cut a UTF-8 scalar in half. After a successful backend parse, Rust sanitizes the retained backend stdout capture so untruncated content is not left in the run directory when `--max-chars` later shortens final content. For `--format json`, the CLI still returns a complete JSON response envelope; only the extracted `content` string is truncated.

For `--format text`, the Crawl4AI helper now prefers `result.extracted_content`, then derives plain text from `cleaned_html` or raw `html` with a stdlib HTML parser, and only falls back to markdown if no HTML content is available. This keeps real backend text output from silently being markdown in the common no-`extracted_content` case.

The stable output metadata now includes `output_options` plus expanded `limits` fields: `truncated_by`, `content_chars_before_truncation`, and `content_chars_after_truncation`. This preserves the I4/I5 session/sensitivity behavior while giving agents enough metadata to decide whether to refetch with larger limits or a narrower selector.

