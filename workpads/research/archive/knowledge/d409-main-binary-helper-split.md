# D409: Main Binary Helper Split

Decision: split binary-only CLI argument inspection and output/error envelope
shaping out of `src/main.rs` while keeping `src/main.rs` as the dispatch
entrypoint for get, current-tab, and session commands.

Boundary after the split:

- `main.rs` owns process entry, parse error handling, and command dispatch.
- `main_args.rs` owns envelope-request detection and stable command-name
  inference for parse errors.
- `main_envelope.rs` owns success-envelope printing, `GetSuccess` envelope data
  shaping with inline-content suppression, generic envelope serialization used
  by session output views, and binary error-response mapping.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
