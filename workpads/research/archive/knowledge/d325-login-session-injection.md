# D325: Provider-Session Injection For Login Start

I23 adds explicit provider-session injection to the owned login lifecycle:

- `aget session login start <target> --url <url> --session <provider>` now loads named local sessions through the `Aget` facade before browser startup.
- The facade reuses `compose_playwright_state` so ambiguous cookie/localStorage/sessionStorage conflicts fail before any pending login file or Chrome profile is created.
- Owned login startup seeds the headed Chrome profile through the existing CDP `load_state` primitive, then navigates to the requested login/target URL.
- Pending-login metadata and JSON/plain output now record injected session names only, not secret values.
- `login finish` still exports only the URL-derived target allowed domains, so provider sessions are not merged into the target bucket unless the user later composes sessions explicitly.
- The command-backed `agent-browser` compatibility login path returns a usage error for nonempty `--session`, because provider-session injection is implemented by the owned browser backend.

Source check:

- `references/repos/agent-browser/cli/src/commands.rs` exposes separate `state load` / `state save` commands.
- `references/repos/agent-browser/cli/src/native/state.rs` owns state loading and saving primitives.
- `references/repos/agent-browser/cli/src/native/e2e_tests.rs` covers launch-time and explicit state loading. The `aget` implementation copies the behavior concept through owned CDP state loading, not upstream code.

Agent-facing guidance:

- Root `skills/aget/SKILL.md` now documents `login start --session <provider>` and the privacy boundary.
- The OAuth-safe manual smoke recipe now includes an opt-in no-double-login check that records only command results, injected session names, and whether prompts were reduced.

Validation:

- `cargo fmt --check`
- `cargo test --test aget_api start_login_session`
- `cargo test --test session_cli session_login_start`
- `cargo test --lib parses_session_login_start`
- `cargo test`
- `git diff --check`

Confidence: Medium-high. Deterministic tests cover API injection, disjoint composition, conflict-before-backend behavior, parser behavior, and command output fields. Real provider no-double-login behavior remains a manual smoke because it requires user-approved credentials and provider-specific UI.
