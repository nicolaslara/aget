# D357: D21-D34 Archive Split

## Decision

The historical D21-D34 knowledge bundle is split into smaller referenced section files while keeping the original path as a compact router.

## Implementation

- `archive/knowledge/d021-d034-poc-session-api.md` is now a compact index.
- Detailed D21-D24 content lives in `archive/knowledge/d021-d034-poc-session-api/d021-d024-fetch-replay-output.md`.
- Detailed D25-D28 content lives in `archive/knowledge/d021-d034-poc-session-api/d025-d028-auth-session-model.md`.
- Detailed D29-D34 content lives in `archive/knowledge/d021-d034-poc-session-api/d029-d034-envelope-login-boundary.md`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- Concatenating the three new section files matches the previous D21-D34 archive content byte-for-byte.
- The original archive path remains available as a router for existing links.
- Line counts after the split: router 17 lines; section files 46, 81, and 52 lines.

## Follow-Up

- Keep splitting oversized historical support bundles only when they materially hurt agent context loading.
- Do not compact `workpads/research/tasks.md`; add short task entries for support-file maintenance instead.
