#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/package-release.sh [--target <triple>] [--out-dir <dir>] [--skip-build]

Build and package a Unix aget release tarball. The archive contains:
  aget
  README.md
  LICENSE
  skills/aget/SKILL.md
  scripts/install-codex-skill.sh
  scripts/install-agent-integrations.sh
  integrations for Cursor, Copilot, and OpenCode
USAGE
}

target=""
out_dir="dist"
skip_build=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      target="${2:-}"
      shift 2
      ;;
    --out-dir)
      out_dir="${2:-}"
      shift 2
      ;;
    --skip-build)
      skip_build=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'error: unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ -z "$target" ]]; then
  target="$(rustc -vV | sed -n 's/^host: //p')"
fi

case "$target" in
  *-windows-*)
    printf 'error: Windows release artifacts are deferred until REL-007 smoke coverage exists\n' >&2
    exit 2
    ;;
esac

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "$version" ]]; then
  printf 'error: could not read package version from Cargo.toml\n' >&2
  exit 1
fi

if [[ "$skip_build" -eq 0 ]]; then
  rustup target add "$target" >/dev/null
  cargo build --release --locked --target "$target"
fi

binary="target/${target}/release/aget"
if [[ ! -x "$binary" ]]; then
  printf 'error: release binary not found or not executable: %s\n' "$binary" >&2
  exit 1
fi

name="aget-v${version}-${target}"
archive="${name}.tar.gz"
mkdir -p "$out_dir"
rm -rf "$out_dir/$name" "$out_dir/$name.tar" "$out_dir/$archive" "$out_dir/$archive.sha256"
mkdir -p \
  "$out_dir/$name/skills/aget" \
  "$out_dir/$name/scripts" \
  "$out_dir/$name/integrations/cursor" \
  "$out_dir/$name/integrations/copilot" \
  "$out_dir/$name/.opencode/tools" \
  "$out_dir/$name/.opencode/lib"

install -m 755 "$binary" "$out_dir/$name/aget"
install -m 644 README.md "$out_dir/$name/README.md"
install -m 644 LICENSE "$out_dir/$name/LICENSE"
install -m 644 skills/aget/SKILL.md "$out_dir/$name/skills/aget/SKILL.md"
install -m 755 scripts/install-codex-skill.sh "$out_dir/$name/scripts/install-codex-skill.sh"
install -m 755 scripts/install-agent-integrations.sh "$out_dir/$name/scripts/install-agent-integrations.sh"
install -m 644 integrations/cursor/aget.mdc "$out_dir/$name/integrations/cursor/aget.mdc"
install -m 644 integrations/copilot/aget.instructions.md "$out_dir/$name/integrations/copilot/aget.instructions.md"
install -m 644 .opencode/tools/aget.ts "$out_dir/$name/.opencode/tools/aget.ts"
install -m 644 .opencode/lib/aget_args.ts "$out_dir/$name/.opencode/lib/aget_args.ts"

find "$out_dir/$name" -exec touch -t 202605240000 {} +

if tar --version 2>/dev/null | grep -q 'GNU tar'; then
  tar --format=ustar --owner=0 --group=0 --numeric-owner -C "$out_dir" -cf "$out_dir/$name.tar" "$name"
else
  COPYFILE_DISABLE=1 tar --format ustar --uid 0 --gid 0 --uname root --gname wheel -C "$out_dir" -cf "$out_dir/$name.tar" "$name"
fi

gzip -n -f "$out_dir/$name.tar"

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$@"
  else
    sha256sum "$@"
  fi
}

(
  cd "$out_dir"
  sha256_file "$archive" > "$archive.sha256"
  sha256_file ./*.tar.gz | sort -k 2 > SHA256SUMS
)

printf '%s\n' "$out_dir/$archive"
