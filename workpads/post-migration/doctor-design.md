# `aget doctor` Design

Status: DR-001 design.

`aget doctor` is a local diagnostics command for the CLI product. It checks the
parts that affect current supported commands and reports whether the user can
run static fetches, session operations, browser-backed flows, current-tab, cmux
import, and OpenCode wrapper calls.

It must not check for Crawl4AI, `agent-browser`, MCP, server processes, or
hosted services.

## Command Shape

```bash
aget doctor
aget --envelope json doctor
aget doctor --quick
aget doctor --check store --check chrome
```

Flags:

| Flag | Purpose |
| --- | --- |
| `--quick` | Skip slow or process-spawning probes. Default doctor should also avoid launching Chrome; quick mainly skips directory sizing and command subprocess probes. |
| `--check <name>` | Repeatable subset selector. Initial names: `binary`, `store`, `artifacts`, `chrome`, `current-tab`, `cmux`, `opencode`. |
| `--fix` | Reserved for later. DR-002 should reject or omit until there is a real repair action. |

Global `--envelope json` selects the stable JSON output. Human output remains
the default.

## Check Model

Each check has:

| Field | Meaning |
| --- | --- |
| `id` | Stable snake-case ID, unique in one run. |
| `category` | One of `binary`, `store`, `artifacts`, `chrome`, `current_tab`, `cmux`, `opencode`. |
| `status` | `ok`, `warn`, or `fail`. |
| `message` | Short human-readable result. |
| `detail` | Optional redacted detail for debugging. |
| `remediation` | Optional user action. |
| `sensitive` | Whether details were redacted or omitted. |

Exit codes:

| Code | Meaning |
| --- | --- |
| `0` | No `fail` checks. Warnings may be present. |
| `1` | One or more `fail` checks. |
| `2` | CLI usage error, such as unknown `--check`. |

Static fetch must remain available even when optional browser or cmux checks
warn/fail. Missing Chrome is a `warn` unless a user selects only `chrome` and
the check is explicitly framed as browser-backed readiness; it is not a global
doctor failure.

## Checks

### Binary

Checks:

- `binary.version`: report package version from `env!("CARGO_PKG_VERSION")`.
- `binary.path`: report `std::env::current_exe()` if available.
- `binary.cwd`: report current working directory.

Statuses:

- `ok` when version is available and executable path can be read.
- `warn` when path or cwd cannot be read but the process is otherwise running.

Redaction:

- Paths are local filesystem metadata. Show full paths in human mode only when
  they are not under obvious secret-bearing locations. JSON should include paths
  because doctor is local diagnostics, but never include environment variable
  values other than selected safe paths described below.

### Store / `AGET_HOME`

Checks:

- `store.home`: resolve the effective `AGET_HOME` path, using `AGET_HOME` if set
  or `~/.aget` otherwise.
- `store.layout`: ensure `sessions/`, `runs/`, `cache/`, and `tmp/` exist.
- `store.permissions`: on Unix, verify `AGET_HOME` and `sessions/` are `0700`.
- `store.writable`: create and remove a small probe file in `tmp/`.
- `store.sessions`: list session files and count valid `.json` names.

Statuses:

- `fail` if the home path cannot be resolved, layout cannot be created/read, or
  `tmp/` is not writable.
- `warn` for loose Unix permissions that can still be repaired by normal
  `SessionStore::ensure_layout`.
- `ok` when layout, permissions, and writable probe pass.

Implementation note:

- DR-002 can reuse `SessionStore::from_env()` / `SessionStore::new()` for layout
  creation, but should separately inspect permissions after creation.

### Run Artifacts

Checks:

- `artifacts.runs_dir`: confirm `runs/` exists and is readable.
- `artifacts.count`: count immediate run directories.
- `artifacts.size`: estimate total size under `runs/`, skipped in `--quick`.
- `artifacts.metadata`: sample a small bounded number of `metadata.json` files
  and verify valid JSON shape where present.

Statuses:

- `fail` if `runs/` is unreadable.
- `warn` if malformed metadata is found or run artifact count/size exceeds a
  documented threshold.
- `ok` otherwise.

Redaction:

- Do not include source URLs, page titles, content snippets, cookie/storage
  values, or metadata payloads in doctor output. Report counts, sizes, and run
  directory names only.

### Chrome / CDP Availability

Checks:

- `chrome.command`: identify the selected Chrome executable from
  `AGET_CHROME_COMMAND`, platform defaults, or `PATH`.
- `chrome.executable`: verify the path exists and appears executable.
- `chrome.launch_capable`: optional future probe; DR-002 should not launch
  Chrome by default.

Statuses:

- `ok` if a candidate executable is found and usable.
- `warn` if Chrome is missing. Static fetch still works.
- `warn` if `AGET_CHROME_COMMAND` is set but the file is missing or not
  executable; browser-backed flows will fail until fixed.

Redaction:

- The Chrome executable path is not secret by itself. Include it as a path
  detail. Do not include profile paths unless the user passed them to a future
  doctor option.

### Current-Tab Prerequisites

Checks:

- `current_tab.command`: report that `current-tab` requires explicit
  `--cdp-port` and `--allow-private-content`.
- `current_tab.port`: optional future check only if a user passes a port to
  doctor. DR-002 should not scan common ports.

Statuses:

- `ok` for command availability and consent boundary documentation.
- `warn` if the user asks to probe a specific port and it is unreachable.

Redaction:

- Do not scan browser tabs, fetch CDP target lists, or include page URLs. A
  local port number is acceptable diagnostic information only when supplied by
  the user.

### cmux

Checks:

- `cmux.command`: resolve `AGET_CMUX_COMMAND` or `cmux` on `PATH`.
- `cmux.executable`: verify the command can be spawned in `--quick` skip mode or
  path-exists mode.

Statuses:

- `ok` if cmux is available.
- `warn` if missing. Only `aget session import cmux` is affected.
- `fail` only for an explicitly selected `--check cmux` mode that cannot run
  the configured command at all and the user asked for strict component
  readiness.

Redaction:

- Do not include cmux surface names or cookie output. Doctor must not invoke
  `cmux browser ... cookies get`.

### OpenCode Tool Resolution

Checks:

- `opencode.tool`: if `.opencode/tools/aget.ts` exists in the current working
  tree, report it.
- `opencode.binary`: if `AGET_OPENCODE_BIN` is set, verify it points to a file
  or command path. Otherwise report that the OpenCode wrapper will call `aget`
  from `PATH`.

Statuses:

- `ok` when the wrapper file is present and the configured binary is usable.
- `warn` when the wrapper is present but `AGET_OPENCODE_BIN` points to a missing
  file, or when neither `AGET_OPENCODE_BIN` nor `aget` on `PATH` can be found.
- `ok` / `info`-style human wording when no `.opencode/tools/aget.ts` is present
  in the current tree; this is optional integration.

Redaction:

- `AGET_OPENCODE_BIN` path can be shown. Do not dump the environment.

## Human Output

Default output should be compact and scan-friendly:

```text
aget doctor

ok   binary.version        aget 0.1.0
ok   store.layout          /Users/me/.aget is ready
ok   store.writable        tmp/ is writable
warn chrome.command        Chrome not found; static fetch still works
warn cmux.command          cmux not found; session import cmux is unavailable
ok   opencode.tool         .opencode/tools/aget.ts found

Summary: 4 ok, 2 warn, 0 fail
```

Human output must not include page content, session secret values, cookie names,
storage keys, full artifact metadata, or CDP target URLs.

## JSON Envelope

With `--envelope json`, use the existing top-level envelope:

```json
{
  "ok": true,
  "schema_version": "aget.envelope.v1",
  "command": "doctor",
  "data": {
    "summary": {"ok": 4, "warn": 2, "fail": 0},
    "checks": [
      {
        "id": "store.layout",
        "category": "store",
        "status": "ok",
        "message": "AGET_HOME layout is ready",
        "detail": {"home": "/Users/me/.aget"},
        "remediation": null,
        "sensitive": false
      }
    ]
  },
  "warnings": [],
  "timing_ms": {"total": 12}
}
```

If any check has status `fail`, top-level `ok` should be `false` and exit code
should be `1`, but the JSON shape should still be a success-shaped doctor
payload rather than an error envelope. Usage errors still use the normal error
envelope.

## Redaction Policy

Doctor may report:

- local binary paths,
- `AGET_HOME`,
- counts and sizes,
- component names and availability,
- explicitly configured command paths such as `AGET_CHROME_COMMAND`,
  `AGET_CMUX_COMMAND`, and `AGET_OPENCODE_BIN`.

Doctor must not report:

- cookie values,
- localStorage/sessionStorage values,
- unredacted session JSON,
- source page content,
- source URLs from run metadata,
- current-tab target URLs or titles,
- arbitrary environment variable dumps.

Malformed JSON or I/O errors should be summarized with file basename or check ID
where possible, not with sensitive payload content.

## DR-002 Implementation Notes

- Add `Doctor(DoctorCommand)` to `src/cli.rs` and include `doctor` in
  `alias_url_index`'s known command list.
- Add a `src/cli/doctor.rs` parser module and `src/main_doctor.rs` execution
  module to keep command parsing separate from diagnostics.
- Reuse `main_envelope` helpers for structured output.
- Keep checks pure and deterministic where possible. Do not launch Chrome or
  query CDP targets by default.
- Tests should cover: healthy temp `AGET_HOME`, missing optional Chrome/cmux,
  bad/warn permissions on Unix, JSON shape and check ID uniqueness, and
  `current-tab`/cmux missing dependencies not failing static fetch readiness.
