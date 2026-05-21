# Aget Engine Refactor Plan

## Goal

Refactor the owned Crawl4AI and `agent-browser` replacements into self-contained internal engines:

- `AgetExtractor`: the local extraction engine that replaces the Crawl4AI behavior `aget` depends on.
- `AgetBrowser`: the local browser/CDP engine that replaces the `agent-browser` behavior `aget` depends on.

`Aget` remains the public product facade. These engines should be directly usable in tests and internally reusable, but they are not intended as standalone user-facing APIs yet.

## Why

The migration has already split oversized files into smaller modules, but the replacement engines are still represented mostly as thin backend structs over free functions:

- `OwnedExtractorBackend` plus `src/extraction/*`
- `OwnedBrowserAutomationBackend` plus `src/browser_cdp/*` and some `src/session/*` glue

That shape works for the CLI, but it makes backend-level testing and reasoning less direct. A test that wants to exercise the local Crawl4AI-like extractor or local browser engine should not need to go through full `Aget` orchestration, named session persistence, CLI envelopes, or run-artifact policy.

The name `owned` has also become confusing. It described the migration away from external command backends, not the product concept. The code should use product names such as `AgetExtractor` and `AgetBrowser` for the local engines.

## Target Shape

### `Aget`

`Aget` stays the high-level facade and owns product policy:

- named session persistence
- `AGET_HOME`
- session composition
- replay-scope enforcement
- local run artifacts
- JSON envelope shape
- sensitive-output defaults
- authorization workflow
- CLI-facing behavior

`Aget` depends on engine capabilities. It should not contain low-level extraction, markdown, CDP, Chrome process, or browser-state implementation details.

### `AgetExtractor`

`AgetExtractor` should be the local extraction engine.

Possible module layout:

```text
src/aget_extractor/
  mod.rs
  config.rs
  request.rs
  response.rs
  options.rs
  http.rs
  render.rs
  html_clean.rs
  markdown.rs
```

Suggested API shape:

```rust
pub struct AgetExtractor {
    browser: AgetBrowser,
    config: AgetExtractorConfig,
}

pub struct ExtractRequest {
    pub url: String,
    pub state: PlaywrightState,
    pub content_format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for_selector: Option<String>,
    pub max_chars: Option<usize>,
    pub backend_options: Vec<ExtractorOption>,
    pub timeout: Duration,
}

pub struct ExtractResponse {
    pub final_url: String,
    pub content: String,
    pub content_format: OutputFormat,
    pub warnings: Vec<String>,
    pub rendered: bool,
    pub metadata: ExtractMetadata,
}
```

It should own:

- static HTTP fetch
- decision to render through browser/CDP
- extraction option parsing for the supported compatibility surface
- CSS selectors, exclusions, target elements, and main-content selection
- HTML cleanup
- markdown/text/html/json content conversion
- extraction warnings and extraction-local metadata

It should not own:

- named sessions
- `SessionStore`
- `~/.aget/runs` artifact policy
- JSON control-plane envelopes
- user-facing auth workflows
- persisted session files

`OwnedExtractorBackend` should be renamed or replaced by an adapter such as `AgetExtractorBackend`, which wraps `AgetExtractor` and implements the existing `ExtractorBackend` trait for `Aget`.

### `AgetBrowser`

`AgetBrowser` should be the local browser/CDP engine.

Possible module layout:

```text
src/aget_browser/
  mod.rs
  config.rs
  request.rs
  response.rs
  chrome_process.rs
  cdp_client.rs
  discovery.rs
  render.rs
  state.rs
  login.rs
  profile.rs
  scripts.rs
  process.rs
```

Suggested API shape:

```rust
pub struct AgetBrowser {
    config: AgetBrowserConfig,
}

impl AgetBrowser {
    pub fn render(&self, request: BrowserRenderRequest) -> Result<RenderedPage, AgetError>;
    pub fn export_state(&self, request: BrowserStateExportRequest) -> Result<PlaywrightState, AgetError>;
    pub fn start_login(&self, request: BrowserLoginStartRequest) -> Result<StartedLoginBrowser, AgetError>;
    pub fn finish_login(&self, request: BrowserLoginStateExportRequest) -> Result<PlaywrightState, AgetError>;
    pub fn close_login(&self, request: BrowserLoginCloseRequest) -> Result<(), AgetError>;
}
```

It should own:

- Chrome binary discovery
- Chrome launch/retry/shutdown behavior
- Chrome process and process-group cleanup
- CDP endpoint discovery
- CDP client/session command plumbing
- page rendering and wait behavior
- cookie/localStorage/sessionStorage load and export
- browser state export primitives
- login browser lifecycle primitives
- startup diagnostics and `requires_user_action` classification inputs

It should not own:

- persisted `Session` files
- named-session import commands
- `session import chrome --name ...` persistence
- login pending metadata files unless those are deliberately moved into a browser-flow subcomponent
- JSON CLI envelopes
- `Aget` authorization workflow

`OwnedBrowserAutomationBackend` should be renamed or replaced by an adapter such as `AgetBrowserBackend`, which wraps `AgetBrowser` and implements `BrowserAutomationBackend` and `BrowserFallbackBackend` for `Aget`.

## Compatibility Backends

The command-backed Crawl4AI and `agent-browser` adapters should remain explicit compatibility surfaces:

- `AGET_CRAWL4AI_COMMAND`
- `AGET_AGENT_BROWSER_COMMAND`

They should stay thin and separate from `AgetExtractor` / `AgetBrowser`. Do not add automatic fallback from local engines to compatibility backends.

## Naming Direction

Stop using `owned` for the active local backend names. Prefer:

- `AgetExtractor`
- `AgetExtractorBackend`
- `AgetBrowser`
- `AgetBrowserBackend`
- `DefaultExtractorBackend::Aget` or `DefaultExtractorBackend::Local`
- `DefaultBrowserAutomationBackend::Aget` or `DefaultBrowserAutomationBackend::Local`

Historical workpad notes may still mention `owned` when referring to past commits, but live code, README, current task status, and new tests should move toward the new names.

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
