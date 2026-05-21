# Knowledge Archive D182: Aget Facade Module Split

### D182: Split helper surfaces out of the `Aget` facade

The I19q mechanical split converted `src/aget.rs` into a module directory while preserving public `aget::aget::*` paths and top-level `lib.rs` re-exports:

- `src/aget/mod.rs`: public `Aget`/`AgetWith` facade, dependency wiring, session orchestration, authorization flow orchestration, and `get` request construction.
- `src/aget/authorize.rs`: authorization request/result DTOs, authorization state, predicate result DTO, and predicate evaluation helper.
- `src/aget/backends.rs`: browser automation trait, default extractor/browser backend enums, compatibility command browser backend, and `AgetBrowserBackend`.
- `src/aget/get_request.rs`: `GetRequest` builder and `run` bridge into extraction.
- `src/aget/session_store.rs`: session-store backend trait, filesystem-backed adapter, and extraction session-store bridge implementation.

Validation:

- `cargo fmt`
- `cargo test --test aget_api`
- `cargo fmt --check`
- `cargo test --test session_cli authorize::session_authorize_imports_verifies_and_omits_sensitive_inline_content` after the first full-suite run hit a transient loopback connection reset in that test.
- `cargo test`
- `git diff --check`

Confidence: High for the mechanical split. Public facade API coverage passed, the transient loopback failure passed on direct rerun, and the full suite passed before marking I19q complete.
