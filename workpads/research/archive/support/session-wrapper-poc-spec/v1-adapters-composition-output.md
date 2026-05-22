## V1 Adapters, Composition, And Output Metadata

### Crawl4AI Adapter

Use the proven helper shape from `workpads/research/benchmarks/crawl4ai_with_storage_state.py`.

The adapter should:

- Accept URL and temp state path.
- Configure `BrowserConfig(storage_state=<path>, channel="chrome" or default chromium)`.
- Use `CrawlerRunConfig(cache_mode=CacheMode.BYPASS, remove_overlay_elements=True)`.
- Return markdown, metadata, and success/failure.
- Map common `aget get` options to Crawl4AI config where possible.
- Pass unsupported or advanced Crawl4AI options through a backend-specific escape hatch.

V1 can shell out to:

```bash
uv run --with crawl4ai <helper.py> --url <url> --state <state> --output <out>
```

The helper should support output formats required by the CLI:

- `markdown` for normal agent context.
- `html` for debugging or later custom extraction.
- `text` for low-noise fallback.
- `json` for metadata plus selected content fields.

### Session Composition To Playwright State

Before extraction, selected sessions are combined into a temporary file:

```json
{
  "cookies": [],
  "origins": []
}
```

Rules:

- No selected sessions means empty `cookies` and `origins`.
- Multiple selected sessions are merged.
- Duplicates with identical value are deduplicated.
- Conflicting duplicates fail.
- Temporary file is deleted after backend exits.

### cmux Adapter Notes

Use cmux only when available and explicitly requested.

Known behavior from benchmark:

- `cmux --json browser cookies get --domain 127.0.0.1` returned scoped cookies correctly.
- `cmux --json browser cookies get --url http://127.0.0.1:9168` returned broad cookies in this environment.
- Therefore V1 must call per-domain export and still post-filter returned cookies.
- `cmux browser state save` should not be used by default because it exports broad state.

### agent-browser Adapter Notes

Known behavior from benchmark:

- `agent-browser --profile Default` can snapshot an authenticated Chrome profile after manual login.
- `agent-browser state save` exports decrypted Playwright-like state that Crawl4AI can consume.
- Raw exported state is broad and sensitive; use only as short-lived temp input.
- `agent-browser state load` did not successfully replay an externally-built scoped local cookie state in the local test. Do not rely on agent-browser as the replay backend in V1.

### Output Metadata

Each run should write metadata:

```json
{
  "url": "https://...",
  "created_at": "...",
  "sessions": ["hellointerview"],
  "extractor": "crawl4ai",
  "sensitive": true,
  "output": "content.md",
  "warnings": [
    "authenticated session used",
    "output may contain private content"
  ],
  "timing_ms": {
    "total": 1234,
    "extractor": 1000
  },
  "limits": {
    "max_chars": 8000,
    "truncated": false
  }
}
```
