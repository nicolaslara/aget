# Debug Artifact Policy

## Decision

Debug artifacts are opt-in run artifacts for troubleshooting extraction. They
must not be captured by default.

Supported flags:

```text
--capture-trace
--capture-screenshot
```

`--capture-trace` writes `debug-trace.json` under the run directory. The trace
is control-plane diagnostic data only: source URL, final URL, content format,
extractor, session names, sensitivity flag, warnings, timings, limits, cache,
usage, output options, and error shape when applicable. It must not include
extracted page content, cookies, localStorage values, headers, or browser
profile data. For sensitive sources, URLs in the trace redact query and
fragment details.

`--capture-screenshot` writes `screenshot.png` only when extraction uses a
browser/CDP path. Static extraction does not launch a browser solely to satisfy
the screenshot flag; it emits a warning and writes no screenshot. Screenshot
capture failure is non-fatal and emits a warning because extracted text remains
the primary result.

## Sensitivity

Debug artifact metadata includes a per-file `sensitive` boolean. Screenshots
from session-backed fetches and all current-tab captures are sensitive because
they may contain visible private page content. Trace files inherit the source
sensitivity even though they avoid page content.

## Artifact Layout

Successful runs may include:

```text
AGET_HOME/runs/<run-id>/content.md
AGET_HOME/runs/<run-id>/metadata.json
AGET_HOME/runs/<run-id>/debug-trace.json
AGET_HOME/runs/<run-id>/screenshot.png
```

Failed runs with `--capture-trace` may include `metadata.json` and
`debug-trace.json`. Failed runs do not promise screenshots.

The public envelope and `metadata.json` expose debug paths as:

```json
{
  "artifacts": {
    "debug": {
      "trace": {
        "path": ".../debug-trace.json",
        "media_type": "application/json",
        "sensitive": true
      },
      "screenshot": {
        "path": ".../screenshot.png",
        "media_type": "image/png",
        "sensitive": true
      }
    }
  }
}
```

## Lifecycle Compatibility

`aget artifacts inspect` lists debug files with internal ownership. `delete`
and `prune` continue to remove only internal run directories, which includes
debug artifacts. Caller-owned `--output` files outside the run directory remain
preserved.

## Current Limits

- Screenshot capture is implemented for browser/CDP render paths and
  `current-tab`; static fetches do not generate screenshots.
- Traces are intentionally diagnostic JSON, not browser protocol logs or HAR
  files.
- No site-specific access-state, paywall, login, or CAPTCHA interpretation is
  added for debug artifacts.
