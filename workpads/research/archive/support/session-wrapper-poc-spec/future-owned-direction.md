## Future Product Direction: Site APIs And Actions

The session model should leave room for `aget` to become a safe local action/API layer for authenticated sites.

Examples:

- Search an online supermarket as the user.
- Add an item to cart.
- Post or comment on a user's behalf.
- Submit forms after explicit approval.
- Extract a reusable local “site API” from observed browser/network behavior.

This is out of scope for v1, but it should influence design now:

- Sessions must be composable and explicitly selected for each action.
- Mutating actions must be distinguishable from read-only extraction.
- Future tools need confirmation policies: dry-run, preview, require approval, execute.
- Network/API discovery should be local and provenance-rich.
- Stored site APIs should record required domains, session dependencies, request shapes, and risk level.
- Agent-facing commands should make side effects explicit.

Possible future commands:

```bash
aget api discover <site> --session <name>
aget api list <site>
aget api call <site.search> --session supermarket --param query="milk"
aget action preview <site.add_to_cart> --session supermarket --param sku=123
aget action run <site.add_to_cart> --session supermarket --param sku=123 --confirm
```

V1 should not build this, but it should avoid decisions that make it impossible. In particular, session provenance, selected-session metadata, non-interactive execution, JSON outputs, and read/write operation classification should exist early.

## V2 Direction: Owned Implementation

V2 should replace the wrapper internals gradually while preserving the same product model.

Likely v2 ownership areas:

- Native session store with encryption at rest.
- Direct browser/CDP integration for controlled profiles and current-tab extraction.
- Built-in scoped cookie and storage import from supported browsers.
- First-class OAuth flow assistant with temporary provider sessions.
- Owned extraction pipeline: rendered HTML to readability-pruned markdown.
- Token budgeting and objective/selector narrowing.
- Native structured extraction modes: markdown, text, HTML, JSON/schema.
- MCP/OpenCode integration.
- Safe action/API mode for selected sites, with explicit read/write classification and user approval policies.
- Better provenance: URL, final URL, selected DOM region, extraction strategy, session names, timestamps.

V2 should keep the same external principles:

- Empty session by default.
- Explicit named sessions.
- Composable sessions.
- Local-only authenticated content.
- Provider-session separation.
- No broad browser-state export unless explicitly requested.
