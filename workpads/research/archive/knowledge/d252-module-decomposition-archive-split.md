# D252: Module Decomposition Archive Split

## Decision

Split the historical D123-D150 module/test decomposition archive into a stable short index plus smaller section files.

## Boundary

- `archive/knowledge/d123-d150-module-decomposition.md` remains the routing entrypoint.
- `archive/knowledge/d123-d150-module-decomposition/d123-d132-core-production-splits.md` preserves the first production extraction/CDP split decisions.
- `archive/knowledge/d123-d150-module-decomposition/d133-d141-orchestration-and-test-target-splits.md` preserves artifact/render/state/login/owned orchestration and first integration target split decisions.
- `archive/knowledge/d123-d150-module-decomposition/d142-d150-integration-test-splits.md` preserves later integration-test splits and I19i completion evidence.

## Validation

- Concatenating the three section files with the original archive title exactly reconstructed the pre-split archive content.
- Section file sizes are 147, 144, and 137 lines; the routing index is 9 lines.

`workpads/research/tasks.md` remains un-compacted.
