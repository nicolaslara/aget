# Aget Engine Refactor Plan: Migration, Testing, And Guardrails

## Migration Slices

Keep the refactor mechanical and separately committable.

1. Introduce `AgetBrowser` as a wrapper around the existing `browser_cdp` module without changing behavior.
2. Change the browser backend adapter to hold and call `AgetBrowser`.
3. Rename `OwnedBrowserAutomationBackend` to `AgetBrowserBackend` or add the new type first and remove the old name after call sites move.
4. Move browser capability types toward the `AgetBrowser` boundary where useful, without moving session persistence policy into the browser engine.
5. Introduce `AgetExtractor` as a wrapper around the existing owned extraction modules without changing behavior.
6. Change the extractor backend adapter to hold and call `AgetExtractor`.
7. Rename `OwnedExtractorBackend` to `AgetExtractorBackend` or add the new type first and remove the old name after call sites move.
8. Move extractor-local request/response/config types under the `AgetExtractor` boundary.
9. Update direct tests to exercise `AgetExtractor` and `AgetBrowser` without going through full `Aget` when the behavior under test is engine-local.
10. Update README, project skill, OpenCode descriptions, and workpad status to stop describing current local code as `owned`.

## Testing Strategy

Add direct engine tests where they reduce setup and make failures clearer:

- `AgetExtractor` static fetch, selector, HTML cleanup, markdown, and option parsing tests.
- `AgetExtractor` rendered extraction tests using a test `AgetBrowser` or a local Chrome smoke when required.
- `AgetBrowser` Chrome startup diagnostics, CDP discovery, state export, render waits, and login lifecycle tests.
- Adapter tests proving `Aget` still wires the engines through existing traits.

Keep existing CLI/API tests for user-facing behavior:

- envelope shape
- artifact policy
- session persistence
- auth/session workflows
- compatibility env-var selection

## Guardrails

- Preserve public CLI/API behavior unless a task explicitly authorizes a behavior change.
- Preserve the existing `ExtractorBackend`, `BrowserAutomationBackend`, and `BrowserFallbackBackend` contracts until the new engine APIs are stable.
- Do not move redaction/artifact/envelope policy into the engines.
- Do not move named-session persistence into `AgetBrowser` or `AgetExtractor`.
- Do not broaden browser support claims while refactoring.
- Do not copy upstream Crawl4AI or `agent-browser` code/tests as part of this refactor.
- Update this plan when implementation reveals a better boundary, a naming conflict, or a policy that belongs at a different layer.
