# D308: Workpad Support Routing Refresh

Date: 2026-05-22

## Decision

Keep `workpads/research/tasks.md` as the full executable backlog.
Compact supporting files by making top-level `knowledge.md` and `references.md` short routers, then move dense completed-history detail into archive files that agents can open only when needed.

## Current Migration State Detail

- Active workpad: `workpads/research/tasks.md`.
- Main migration umbrella: I19 on branch `dep-migration-homegrown-backends`.
- Completed migration phases include source clones/inventory, backend abstraction audit, parity tests, default owned runtime switch, and PoC command-default demotion.
- Later completed migration phases include engine naming cleanup, module/test/support decomposition, and follow-up cleanup-module splitting.
- Still open: I19d, I19e, and I19h.
- I19d remains open for full Crawl4AI-quality readability/markdown and still-richer rendered-page readiness.
- Recent I19d detail:
  D281-D286 cover metadata, fallback metadata, agent-facing option docs, and code-block whitespace. D293-D307 added source-backed Crawl4AI markdown option support.
- I19e remains open for broader rendered JavaScript parity, manual real logged-in profile/keychain smoke execution, and startup/error classification. D284 and D287 hold current wait/startup message detail.
- I19h remains open because final migration review requires review subagents for test adequacy, architecture cohesion, and security/privacy.

## Support-File Routing Detail

- Do not compact `workpads/research/tasks.md`; it remains the complete executable backlog.
- Top-level `knowledge.md` should keep only current direction, current open work, routing tables, verification expectations, and open questions.
- Top-level `references.md` should keep only source snapshots, active architecture pointers, external primary-source pointers, and archive routing.
- The largest historical knowledge bundles route through split archive indexes after D232.
- The historical session-wrapper PoC spec and implementation plan route through smaller support slices after D243 and D263-D265.
- The R12, D55-D63, D64-D76, D77-D91, D92-D108, D109-D122, engine-refactor, and OAuth-design bundles route through smaller support slices after D268-D280.
- CDP page-script helpers route through behavior-owned modules after D276.
- Crawl4AI local-content URL compatibility routes through D277.
- Raw/local base-URL option behavior routes through D278.
- Main-content negative class/id exclusion routes through D279.
- Page metadata propagation routes through D281.
- Fallback metadata propagation routes through D282.
- Networkidle reset/timeout parity coverage routes through D283.
- CDP navigation wait timeout messages route through D284.
- Agent-facing option doc alignment routes through D285.
- Code-block whitespace preservation routes through D286.
- Chrome early-exit startup message parity routes through D287.
- Browser CDP discovery tests route through behavior-owned modules after D288.
- Mock-site docs-contract tests route through behavior-owned modules after D289.
- Owned extractor option parsing helpers route through a child module after D290.
- Owned page rendering/readiness helpers route through child modules after D291.
- Aget facade session orchestration routes through a child module after D292.
- Crawl4AI markdown option compatibility routes through D293-D307.

## Validation

- `wc -l workpads/research/*.md workpads/research/archive/knowledge/*.md workpads/research/archive/references/*.md workpads/research/archive/support/*.md`
- `awk 'length($0)>240 {print FILENAME ":" FNR ":" length($0)}' workpads/research/knowledge.md workpads/research/references.md workpads/research/*.md`
- `awk 'length($0)>240 {print FILENAME ":" FNR ":" length($0)}' workpads/research/knowledge.md workpads/research/references.md workpads/research/archive/knowledge/d308-workpad-support-routing.md`
- D308 route check with `rg -n "d308-workpad-support-routing|D308|D177-D308"` across the top-level knowledge file, current decision index, D308 archive note, and task file.
- `git diff --check`
