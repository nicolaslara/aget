# Knowledge Archive D174: Reference Compaction

### D174: Compact support references while preserving the task plan

After the engine-refactor slice reached a stable commit, the user clarified that support files can be compacted but `workpads/research/tasks.md` should not be compacted. The planned tasks remain in the top-level task file, including open and completed task detail.

`workpads/research/references.md` now acts as a current routing index. Dense historical reference detail was moved into:

- `workpads/research/archive/references/source-projects-and-comparables.md`
- `workpads/research/archive/references/benchmarks.md`
- `workpads/research/archive/references/architecture-inputs.md`
- `workpads/research/archive/references/browser-automation-and-rust-candidates.md`
- `workpads/research/archive/references/later-media-inputs.md`

This keeps the ordinary workpad load focused on current source snapshots, architecture pointers, active primary sources, and where to open deeper evidence. The archived reference rows preserve the previous source URLs, benchmark paths, license notes, and historical architecture mapping.

Validation:

- `wc -l workpads/research/references.md workpads/research/archive/references/*.md`
- `rg -n "archive/references|D174|Task I19k" workpads/research`
- `git diff --check`

Confidence: High. This is documentation routing only; no historical reference rows were intentionally deleted, and `tasks.md` was not compacted.
