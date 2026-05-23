# D401: Owned Main-Content Split

Decision: split the owned main-content candidate selection and scoring helpers
into smaller modules without changing the default extraction entrypoint or
scoring behavior.

Boundary after the split:

- `main_content/mod.rs` owns the `default_main_content_element_id` entrypoint,
  candidate iteration, pruning-excluded ancestor checks, and body/root fallback.
- `main_content/scoring.rs` owns the main-content score formula, dense generic
  candidate acceptance, descendant positive-container checks, and Crawl4AI-like
  pruning score.
- `main_content/labels.rs` owns positive/negative label bonuses and penalties.
- `main_content/text.rs` owns word counting and normalized text joining used by
  the scorer.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
