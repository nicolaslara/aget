# D364: Large-File Ergonomics Audit

## Decision

No new source, test, or support-workpad split is needed in this checkpoint.
`workpads/research/tasks.md` remains intentionally uncompacted as the full executable backlog.

## Evidence

- `git ls-files | xargs wc -l | sort -nr | head -40` shows only `workpads/research/tasks.md` and `Cargo.lock` above the current source/test comfort range.
- The largest tracked source/test file after those intentional exceptions is `src/browser_cdp/tests/chrome/cdp_client/attached_page/capture.rs` at 382 lines.
- Top-level workpad routers stay compact: `workpads/research/knowledge.md` is 66 lines and `workpads/research/references.md` is 75 lines.
- The workpad archive already contains smaller referenced files for prior dense support bundles, so future agents should route through indexes instead of compacting `tasks.md`.

## Boundary

- Do not compact `workpads/research/tasks.md`.
- Revisit file splitting only when a tracked source/test/support file grows back into the original multi-thousand-line pressure range or starts blocking focused agent work.
- Use git-tracked file scans (`git ls-files`, `rg --files`) instead of broad `find` scans so ignored local build artifacts do not pollute the audit.

## Validation

- `git ls-files | xargs wc -l | sort -nr | head -40`
- `find workpads/research/archive/knowledge -type f -maxdepth 1 | wc -l`
- `git diff --check`

## Follow-Up

- I19d, I19e, and I19h remain the open migration tasks.
