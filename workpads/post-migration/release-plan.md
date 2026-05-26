# Release Plan

## Scope

`aget` releases are CLI binary releases. Do not add a server, MCP layer,
desktop app bundle, notarization flow, or hosted service to release the current
product. macOS notarized/app-bundle packaging remains deferred unless the user
explicitly asks for it.

## Install Targets

Supported install paths for the next release:

| Target | Audience | Command / Artifact | Notes |
| --- | --- | --- | --- |
| Developer checkout | contributors | `cargo install --path .` | Primary local install path before artifact publication. |
| Git source | contributors/agents | `cargo install --git https://github.com/nicolaslara/aget --tag v0.1.0 --locked aget` | Source install from the published repository tag; package name is explicit because the repo contains more than one binary package. |
| Source checkout | contributors/agents | `cargo run -- <command>` | Useful for smoke tests and unreleased work. |
| Release tarball | end users | `aget-<version>-<target>.tar.gz` | Contains one `aget` binary, README, LICENSE, and the Codex skill. |
| Codex global skill | agent workflows | copy or symlink `skills/aget` to `$CODEX_HOME/skills/aget` | Requires a Codex restart after install or replacement. |

Initial binary artifact targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

Windows artifacts are deferred until a Windows smoke path exists. The code has
some Windows-aware process helpers, but release confidence should come from an
actual package smoke before advertising Windows binaries.

## Binary Naming

Release archive names:

```text
aget-v<semver>-<target>.tar.gz
aget-v<semver>-<target>.tar.gz.sha256
```

Archive contents:

```text
aget
README.md
LICENSE
skills/aget/SKILL.md
```

The unpacked executable should be named `aget` on Unix targets. If Windows is
added later, use `aget.exe` inside `aget-v<semver>-x86_64-pc-windows-msvc.zip`.
The unpacked skill should be installable by copying `skills/aget` into
`$CODEX_HOME/skills/aget` and restarting Codex.

## Version Policy

- Source of truth: `Cargo.toml` package `version`.
- Current pre-1.0 releases may use patch/minor increments for user-visible CLI
  changes.
- Do not publish artifacts if `aget --version` disagrees with `Cargo.toml`.
- Record user-visible changes in `CHANGELOG.md` before producing release
  artifacts. If no changelog exists, create one in the release artifact task.

## License Policy

- Source of truth: `Cargo.toml` currently declares `MIT`.
- A tracked top-level `LICENSE` file must exist before release artifacts are
  produced.
- Archive contents must include `LICENSE`.
- Dependency license review is not required for every patch release, but any new
  runtime dependency added after the previous release must have a compatible
  license recorded in the release notes or workpad references.

## Checksums

Generate one SHA-256 file per archive:

```bash
(cd dist && shasum -a 256 "aget-v${version}-${target}.tar.gz" \
  > "aget-v${version}-${target}.tar.gz.sha256")
```

For a multi-target release, also generate a manifest:

```text
dist/SHA256SUMS
```

The manifest should contain all archive hashes and be regenerated from the final
files only.

## Release Smoke Gate

Run from a clean checkout before artifacts:

```bash
cargo fmt --check
git diff --check
cargo test
cargo build --release
```

Run a no-command-path smoke to prove default static fetch does not depend on old
external adapters:

```bash
tmpdir="$(mktemp -d)"
env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  AGET_HOME="$tmpdir/aget-home" \
  target/release/aget --envelope json get 'raw:<main><h1>No Command Path</h1></main>'
rm -rf "$tmpdir"
```

Run doctor against the release binary:

```bash
tmpdir="$(mktemp -d)"
AGET_HOME="$tmpdir/aget-home" target/release/aget --envelope json doctor --quick
rm -rf "$tmpdir"
```

Run installed-binary smoke:

```bash
cargo install --path . --locked
aget --version
aget --help
aget --envelope json doctor --quick
```

The release smoke passes when:

- The commands above exit successfully.
- `doctor` reports `ok: true`; optional component warnings are acceptable.
- The no-command-path smoke returns `ok: true` and extracted content.
- No active Crawl4AI or `agent-browser` runtime surface appears in README,
  skill, OpenCode tool, `src`, `tests`, or scripts outside historical/parity
  workpad notes.

## README And Skill Sync

Before producing artifacts:

1. Verify README install instructions mention the exact artifact names produced.
2. Verify `skills/aget/SKILL.md` uses the same command vocabulary as README and
   `aget --help`.
3. Run help snapshots:

```bash
target/release/aget --help
target/release/aget get --help
target/release/aget current-tab --help
target/release/aget session --help
target/release/aget doctor --help
```

4. Update `workpads/post-migration/references.md` with the commands and results
   used for release validation.

## Release Checklist

- [ ] `Cargo.toml` version chosen and matches `aget --version`.
- [ ] `CHANGELOG.md` has user-visible changes.
- [ ] Top-level `LICENSE` exists and matches `Cargo.toml`.
- [ ] README install section matches produced artifact names.
- [ ] `skills/aget/SKILL.md` and `.opencode/tools/aget.ts` match current CLI
      vocabulary.
- [ ] `cargo fmt --check`, `git diff --check`, and `cargo test` pass.
- [ ] `cargo build --release` passes.
- [ ] No-command-path smoke passes.
- [ ] `target/release/aget --envelope json doctor --quick` passes.
- [ ] Release archives are named `aget-v<semver>-<target>.tar.gz`.
- [ ] Per-archive `.sha256` files and `SHA256SUMS` are generated.
- [ ] Archives contain `aget`, `README.md`, `LICENSE`, and
      `skills/aget/SKILL.md`.
- [ ] Release notes include known limitations: Chrome/CDP required for
      browser-backed flows, cmux optional, no server/MCP layer, no notarized
      app bundle.
