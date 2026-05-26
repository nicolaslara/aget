# Windows Release Smoke Path

## Decision

Do not publish Windows artifacts until a real Windows runner builds and smokes
the binary. The current Unix release automation intentionally excludes Windows.
Runner labels below are based on GitHub's hosted-runner reference as checked on
2026-05-26:
`https://docs.github.com/actions/reference/runners/github-hosted-runners`.

First Windows target:

- `x86_64-pc-windows-msvc`
- GitHub runner: `windows-2025`
- Artifact name: `aget-v<version>-x86_64-pc-windows-msvc.zip`

Windows ARM can be evaluated later on `windows-11-arm`, after x64 packaging is
proven.

## Required Build

```powershell
rustup toolchain install stable --profile minimal
rustup default stable
cargo build --release --locked --target x86_64-pc-windows-msvc
```

The built binary must be:

```text
target/x86_64-pc-windows-msvc/release/aget.exe
```

## Required Package Contents

```text
aget.exe
README.md
LICENSE
skills/aget/SKILL.md
```

Add a Windows skill-install helper only after the CLI artifact itself passes the
smoke path. The Unix `scripts/install-codex-skill.sh` should not be presented
as a Windows install helper.

## Required Smoke Commands

Use PowerShell on the Windows runner.

```powershell
$ErrorActionPreference = "Stop"
$bin = "target\x86_64-pc-windows-msvc\release\aget.exe"
& $bin --version
```

Doctor must succeed for static CLI readiness. Optional Chrome/CDP or cmux
warnings are acceptable; failures are not.

```powershell
$env:AGET_HOME = Join-Path $env:RUNNER_TEMP "aget-home-doctor"
$doctor = & $bin --envelope json doctor --quick | ConvertFrom-Json
if (-not $doctor.ok) { throw "doctor failed" }
```

Static no-command-path fetch must not depend on browser or external adapters.

```powershell
$oldPath = $env:PATH
$env:PATH = "$env:SystemRoot\System32;$env:SystemRoot"
$env:AGET_HOME = Join-Path $env:RUNNER_TEMP "aget-home-static"
$get = & $bin --envelope json get "raw:<main><h1>No Command Path</h1></main>" | ConvertFrom-Json
$env:PATH = $oldPath
if (-not $get.ok) { throw "static get failed" }
if ($get.data.content -notmatch "No Command Path") { throw "static get content mismatch" }
```

Artifact commands must work with Windows paths.

```powershell
$env:AGET_HOME = Join-Path $env:RUNNER_TEMP "aget-home-artifacts"
$get = & $bin --envelope json get "raw:<main><h1>Windows Artifact</h1></main>" | ConvertFrom-Json
$runDir = Split-Path -Parent $get.data.artifacts.metadata
$runId = Split-Path -Leaf $runDir
$list = & $bin --envelope json artifacts list | ConvertFrom-Json
$inspect = & $bin --envelope json artifacts inspect $runId | ConvertFrom-Json
if (-not $list.ok) { throw "artifacts list failed" }
if (-not $inspect.ok) { throw "artifacts inspect failed" }
```

Package smoke must unzip the final artifact and rerun at least version, doctor,
static get, and artifact inspect against the unpacked `aget.exe`.

## Blockers Before Publishing

- Add ZIP packaging support instead of reusing Unix tarball packaging.
- Prove the PowerShell smoke sequence on a `windows-2025` runner.
- Confirm `doctor --quick` returns `ok: true` on Windows when Chrome and cmux
  are absent or optional.
- Confirm artifact paths, `AGET_HOME`, and run ID parsing are stable with
  Windows path separators.
- Add checksum generation for `.zip` artifacts and include them in release
  `SHA256SUMS`.

## Future Workflow Slice

Add a separate Windows package job only after the smoke commands above pass:

```yaml
windows-package:
  runs-on: windows-2025
  steps:
    - uses: actions/checkout@v4
    - run: rustup toolchain install stable --profile minimal
    - run: rustup default stable
    - run: cargo build --release --locked --target x86_64-pc-windows-msvc
    - run: .github/scripts/package-windows.ps1 -Target x86_64-pc-windows-msvc
    - run: .github/scripts/smoke-windows.ps1 -Target x86_64-pc-windows-msvc
```
