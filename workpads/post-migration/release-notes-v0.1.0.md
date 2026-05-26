# aget v0.1.0

First CLI-only release of `aget`, a local-first, auth-aware web context tool
for agents.

## Highlights

- Owned default extractor, browser, and session paths; no server or MCP layer.
- `aget get` and `aget current-tab` for markdown/html/text/json extraction.
- `aget session` for explicit session replay, import, compose, authorization,
  and controlled login flows.
- `aget doctor` for local readiness diagnostics.
- `aget artifacts list/inspect/delete/prune` for local run-artifact lifecycle
  management under `AGET_HOME`.
- `aget batch`, `aget map`, and bounded `aget crawl` for multi-URL agent
  workflows.
- Release archive includes the `aget` binary, README, LICENSE, and
  `skills/aget` for global Codex skill installation.

## Install

Source install from the published tag:

```bash
cargo install --git https://github.com/nicolaslara/aget --tag v0.1.0 --locked aget
```

Binary install on macOS ARM:

```bash
curl -fLO https://github.com/nicolaslara/aget/releases/download/v0.1.0/aget-v0.1.0-aarch64-apple-darwin.tar.gz
curl -fLO https://github.com/nicolaslara/aget/releases/download/v0.1.0/aget-v0.1.0-aarch64-apple-darwin.tar.gz.sha256
shasum -a 256 -c aget-v0.1.0-aarch64-apple-darwin.tar.gz.sha256
tar -xzf aget-v0.1.0-aarch64-apple-darwin.tar.gz
install -d "$HOME/.local/bin"
install -m 755 aget-v0.1.0-aarch64-apple-darwin/aget "$HOME/.local/bin/aget"
"$HOME/.local/bin/aget" --help
```

Install the Codex skill from an unpacked release tarball:

```bash
CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
mkdir -p "$CODEX_HOME/skills"
rm -rf "$CODEX_HOME/skills/aget"
cp -R aget-v0.1.0-aarch64-apple-darwin/skills/aget "$CODEX_HOME/skills/aget"
```

Restart Codex after installing or replacing the global skill.

## Known Limits

- This first binary artifact is macOS ARM only.
- Browser-backed flows require local Chrome/CDP availability.
- cmux is optional and only used for explicit `session import cmux`.
- No hosted service, daemon, MCP server, or desktop app bundle is included.
- No notarized app bundle or Windows artifact is published in this release.
