# D265: Split Historical Session-Wrapper Product Spec

## Decision

Split the dense historical session-wrapper product spec support file into smaller routed archive files without compacting or removing planned tasks.

## Boundary

- `workpads/research/archive/support/session-wrapper-poc-spec/product-spec.md` is now a compact routing index.
- `product-spec/overview.md` owns summary, problem, goals, and non-goals.
- `product-spec/core-concepts.md` owns empty-session, session, session-combination, composed-session, and provider-session concepts.
- `product-spec/user-journeys.md` owns public fetch, cmux import, Chrome import, session-backed fetch, combined-session fetch, and OAuth cleanup journeys.
- `product-spec/ux-security.md` owns UX principles plus security and privacy requirements.

## Rationale

The old product spec remains useful historical context, but agents should not need to load the full product narrative when they only need UX, security, journey, or concept details. The stable entrypoint remains in place as a router.

## Validation

- `rg -n "product-spec/(overview|core-concepts|user-journeys|ux-security)|Session Wrapper PoC Product Spec" workpads/research/archive/support/session-wrapper-poc-spec.md workpads/research/archive/support/session-wrapper-poc-spec/product-spec.md workpads/research/archive/support/session-wrapper-poc-spec/product-spec`
- `wc -l workpads/research/archive/support/session-wrapper-poc-spec/product-spec.md workpads/research/archive/support/session-wrapper-poc-spec/product-spec/*.md`
- `git diff --check`
