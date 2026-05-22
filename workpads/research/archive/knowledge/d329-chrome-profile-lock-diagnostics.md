# D329: Chrome Profile-Lock Startup Diagnostics

Owned Chrome/CDP startup now classifies more profile-lock startup variants as `requires_user_action`.

Source check:

- `references/repos/agent-browser/README.md` documents Chrome profile reuse through copied temp profiles and notes that Chrome may need to be closed when profile files are locked.
- `references/repos/agent-browser/docs/src/app/sessions/page.mdx` repeats the same Chrome profile reuse boundary and Windows locked-file note.
- `references/repos/agent-browser/docs/src/app/changelog/page.mdx` records direct startup error reporting and reliable Chrome launch as agent-browser behavior worth preserving.

Implementation boundary:

- Chrome stderr containing `Process Singleton`, `Singleton Lock`, another Chrome process, or `user data directory is already in use` now maps to `requires_user_action`.
- Relevant Chrome stderr still appears in the surfaced error message so the user can see the concrete local Chrome failure.
- Sandbox startup hints, generic backend-unavailable classification, silent-exit hints, launch retries, and CDP discovery behavior are unchanged.
- This is a bounded diagnostic parity slice; it does not change browser launch flags, profile copying, or auth/session import behavior.

Validation:

- `cargo test --lib chrome_startup_error`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High. The change is limited to startup-error string classification and is covered by focused tests for both `Process Singleton` and `user data directory is already in use` messages, plus the full standard test gate.
