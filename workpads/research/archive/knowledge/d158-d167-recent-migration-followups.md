# Knowledge Archive D158-D167: Recent Migration Follow-Ups

### D158: I19d improves owned main-content candidate selection

Before the slice, the owned extractor selected default main content only when `<main>`, `[role="main"]`, or `<article>` was unique; otherwise it fell back to `<body>`, which could reintroduce headers, promotional cards, and footer text. Source inspection for this slice looked at Crawl4AI's `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`, especially `RelevantContentFilter`'s included/excluded semantic tags, negative label patterns, minimum word count, and chunk extraction approach.

`aget` did not port Crawl4AI's full filtering stack. Instead, the owned extractor now keeps the existing simple semantic default but ranks multiple `main`/`role=main`/`article` candidates using a deterministic local score:

- word count increases score;
- link-heavy candidates are penalized;
- semantic tags get small positive bonuses;
- labels such as nav, footer, sidebar, ad, promo, comment, related, share, and social are penalized.

This moves the owned default closer to Crawl4AI-style content pruning without adding query/BM25/LLM filtering or site-specific heuristics. `tests/mock_site_cli/owned.rs` now covers multiple article candidates and verifies that the useful story is selected instead of body-level noise or a promo card.

Validation:

- `cargo fmt`
- `cargo test --test mock_site_cli owned::homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. The heuristic is deterministic and narrowly tested; it intentionally does not claim full Crawl4AI readability parity.

### D159: I19d implements owned word-count pruning

The owned extractor now applies `crawl4ai.word_count_threshold` instead of only validating it. Source inspection for this slice used:

- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `remove_empty_elements_fast` removes low-word leaf elements bottom-up, skips bypass tags such as `a`, `img`, `br`, table cells/rows, and preserves descendants inside `pre`/`code`;
- `references/repos/crawl4ai/crawl4ai/utils.py`, where cleaned-content helpers recursively remove empty and low-word tags based on the configured threshold.

`aget` ports the safe deterministic part into `src/extraction/html_clean.rs`: empty/low-word leaf pruning now accepts a threshold, keeps the existing bypass tags, preserves code-block descendants, and never removes selected root/target elements. `OwnedExtractorOptions` defaults the threshold to `1`, preserving previous empty-leaf cleanup unless callers explicitly set `crawl4ai.word_count_threshold=<n>`. The parser now rejects non-`usize` values with an explicit non-negative-integer error.

Coverage in `tests/mock_site_cli/owned.rs` verifies that a threshold of `4` removes short captions and short leaf paragraphs while keeping a useful paragraph and preserving the rest of the owned extractor parity slice.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --test mock_site_cli owned::homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This is source-backed and deterministic, but still a bounded cleanup option rather than full Crawl4AI readability filtering.

### D160: I19d recognizes labeled content containers in owned main-content selection

The next readability slice broadens the owned default main-content heuristic beyond `main`, `[role=main]`, and `article`. Source inspection for this slice re-used `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`, where `RelevantContentFilter` includes `section` and `div` as content-bearing structures, excludes nav/footer/header/sidebar-style noise, and uses negative class/id patterns as part of relevance filtering.

`aget` now considers labeled `section` and `div` candidates when no explicit selector or wait selector is provided. Generic `section`/`div` elements only enter the default candidate set when their id/class/role/aria-label contains positive content labels such as `content`, `article`, `story`, `post`, `entry`, `doc`, or `main`; negative labels such as nav, footer, sidebar, ad, promo, comment, related, share, and social still penalize the score. This keeps the heuristic generic and avoids site-shaped paywall/login behavior while covering common pages that wrap the useful article in `<div id="article-content">` or similar containers instead of semantic `<main>`/`<article>`.

Coverage in `tests/mock_site_cli/owned.rs` verifies that a labeled story container is selected by default while surrounding nav, sidebar/promo, and footer content are excluded.

Validation:

- `cargo test homegrown_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`

Confidence: Medium-high. The behavior is deterministic and source-backed, but it is still a bounded local heuristic rather than a full Crawl4AI pruning/readability port.

### D161: Follow-up CLI decomposition keeps command parsing behavior-owned

After the original I19i extraction/CDP split, the largest remaining source file was `src/cli.rs`. A small follow-up split moved get-command arguments to `src/cli/get.rs`, session command/browser/import/login argument types to `src/cli/session.rs`, and CLI parser tests to `src/cli/tests.rs`. `src/cli.rs` now keeps the top-level parser, global options, shared output/envelope/backend-option types, URL aliasing, and parse helpers, while re-exporting the moved command types so the public `aget::cli::*` and crate-level exports remain stable.

This is a mechanical decomposition only; no command names, flags, defaults, or public type names intentionally changed.

Validation:

- `cargo test cli::tests --lib`

Confidence: High. The moved parser tests passed after the split, and the change preserves the same derive-based clap surfaces.

### D162: Follow-up binary orchestration split isolates session CLI execution

The next decomposition slice moved session-command execution out of `src/main.rs` into `src/main_session.rs`. The binary entrypoint now keeps process exit handling, top-level command parsing, get-command request assembly, structured envelope helpers, and error translation. The new session module owns session subcommand dispatch, browser/profile CLI argument resolution, login/import/authorize human output, session inspection redaction views, and session command timing.

This is a mechanical split only. It does not intentionally change command names, output envelopes, human text, profile validation, redaction, or backend selection.

Validation:

- `cargo test --test cli`
- `cargo test --test session_cli session_list_json_has_stable_shape`

Confidence: High. The moved code remains inside the binary crate, preserves the same private helper access through the parent module, and focused CLI/session tests pass.

### D163: I19d gives semantic figure/details blocks stable markdown boundaries

The next markdown-quality slice ports a small semantic-content boundary from Crawl4AI. Source inspection used:

- `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`, where `RelevantContentFilter` treats `figure`, `figcaption`, `details`, `summary`, `address`, `time`, and `cite` as included content tags rather than noise;
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `CustomHTML2Text` delegates non-special semantic tags through the base HTML2Text flow while preserving markdown generation through its cleaned HTML input.

`OwnedExtractorBackend` now renders `figure`, `figcaption`, `details`, `summary`, and `address` with block boundaries instead of letting them collapse into adjacent inline text. Inline semantic tags such as `cite` and `time` continue to render as text. This keeps captions, expandable details, and contact blocks readable in markdown without adding site-specific extraction logic.

Coverage in `tests/mock_site_cli/owned.rs` verifies a figure image plus caption, a details/summary block, and an address block inside the existing static markdown parity fixture.

Validation:

- `cargo test homegrown_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`

Confidence: Medium-high. This is source-backed and deterministic, but it is a local markdown-boundary improvement rather than full Crawl4AI readability parity.

### D164: I19d keeps automatic absolute links title-insensitive

The next markdown-default slice ports a small Crawl4AI/html2text automatic-link edge case. Source inspection used `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`, where `handle_data` emits `<absolute-url>` as soon as an anchor's visible text exactly matches its absolute `href`; the later anchor-close branch clears the pending automatic link and does not render the anchor `title`.

`aget` now preserves the same behavior in the owned markdown renderer. Absolute URL anchors still become automatic markdown links even when the source anchor has a `title` attribute. Ordinary non-automatic links continue to preserve escaped titles.

Coverage in `tests/mock_site_cli/owned.rs` adds a titled canonical URL link and verifies the output remains `<https://example.com/docs>` instead of `[https://example.com/docs](https://example.com/docs "Docs title")`.

Validation:

- `cargo test homegrown_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`

Confidence: High. The behavior is source-backed, narrow, and covered by the existing owned markdown link-default fixture.

### D165: Follow-up mock-site owned fixture split

The next decomposition slice reduced the largest remaining tracked Rust file, `tests/mock_site_cli/owned.rs`, without changing test behavior. Route construction for the owned extractor parity site now lives in `tests/mock_site_cli/owned_site.rs`; the original owned parity test keeps the assertions and imports the shared fixture builder from the test crate root.

This keeps extraction assertions easier to scan while leaving the mock route corpus available to future owned extraction test splits.

Validation:

- `cargo fmt --check`
- `cargo test homegrown_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test`

Confidence: High. This is a mechanical test-fixture split with no production behavior change.

### D166: Follow-up session import test split isolates cmux coverage

The next decomposition slice reduced `tests/session_cli/imports.rs` by moving cmux-specific import coverage into `tests/session_cli/imports_cmux.rs`. The original imports module now keeps Chrome/browser-profile import behavior, while the new module owns mocked cmux import, missing-cmux error handling, and ignored real-cmux loopback checks.

This is a mechanical test split only. It keeps assertion bodies and helper usage unchanged, but separates the cmux compatibility surface from Chrome-owned import behavior so future browser-import work can load less unrelated test context.

Validation:

- `cargo fmt`
- `cargo test --test session_cli session_import_cmux`
- `cargo test --test session_cli session_import_chrome_saves_filtered_state_and_cleans_raw_file`
- `cargo fmt --check`
- `cargo test`

Confidence: High. The split moves tests along backend boundaries and does not change product or adapter behavior.

### D167: Follow-up CDP discovery test split

The next decomposition slice reduced `src/browser_cdp/tests.rs` by moving CDP discovery and Chrome-startup diagnostics coverage into `src/browser_cdp/tests/discovery.rs`. The new submodule owns DevTools stderr parsing, `DevToolsActivePort` parsing and stale-file cleanup, `/json/version` and `/json/list` discovery fallbacks, direct websocket fallback discovery, startup hint classification, and the local HTTP discovery helpers.

This is a mechanical test split only. The root CDP test module still covers cookie/storage translation, page-script expressions, Chrome launch retry, and ignored real Chrome state/login smoke tests. While validating the split, the full suite exposed that the fake-Chrome stderr fallback test's two-second timeout was too tight under parallel load; the test now uses a five-second timeout while preserving the same assertion.

Validation:

- `cargo test browser_cdp::tests::discovery`
- `cargo test browser_cdp::tests::discovery::wait_for_devtools_active_port_uses_stderr_fallback`
- `cargo test browser_cdp::tests::`
- `cargo fmt --check`
- `cargo test`

Confidence: High. The split follows the existing `src/browser_cdp/discovery.rs` boundary and does not change production CDP behavior.
