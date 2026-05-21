# D232: Archive Knowledge Bundle Split

## Decision

- Keep `workpads/research/tasks.md` as the full planned-task source of truth; do not compact or remove planned tasks.
- Replace the three largest historical knowledge bundles with short routing indexes at their original paths.
- Move dense historical content into smaller archive files by decision range:
  - `d001-d020-foundation-research.md`
  - `r12-mvp-architecture-proposal.md`
  - `d021-d034-poc-session-api.md`
  - `d035-d048-poc-hardening-backends.md`
  - `d049-d054-migration-setup-and-parity.md`
  - `d055-d063-owned-extractor-foundation.md`
  - `d064-d076-browser-default-switch.md`
  - `d077-d091-owned-extractor-options-cleanup.md`
  - `d092-d108-markdown-browser-slices.md`
  - `d109-d122-selector-overlay-shadow.md`
- Keep top-level `knowledge.md` and `current-decision-index.md` as routing surfaces so future agents can load only the needed range.

## Validation

- `find workpads/research/archive/knowledge -maxdepth 1 -type f ... | xargs wc -l`
- `git diff --check`
