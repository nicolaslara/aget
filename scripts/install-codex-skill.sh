#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/install-codex-skill.sh [--copy|--symlink] [--force] [--codex-home DIR]

Install the aget Codex skill into $CODEX_HOME/skills/aget.

Options:
  --copy            Copy the skill directory. This is the default.
  --symlink         Symlink the checkout skill directory.
  --force           Replace an existing aget skill directory or symlink.
  --codex-home DIR  Use DIR instead of $CODEX_HOME or $HOME/.codex.
  -h, --help        Show this help.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
source_dir="$repo_root/skills/aget"
codex_home="${CODEX_HOME:-$HOME/.codex}"
mode="copy"
force=0

while (($#)); do
  case "$1" in
    --copy)
      mode="copy"
      shift
      ;;
    --symlink)
      mode="symlink"
      shift
      ;;
    --force)
      force=1
      shift
      ;;
    --codex-home)
      if [[ $# -lt 2 ]]; then
        printf 'error: --codex-home requires a directory\n' >&2
        exit 2
      fi
      codex_home="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'error: unknown argument: %s\n\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ ! -f "$source_dir/SKILL.md" ]]; then
  printf 'error: missing source skill: %s\n' "$source_dir/SKILL.md" >&2
  exit 1
fi

target_parent="$codex_home/skills"
target_dir="$target_parent/aget"

mkdir -p "$target_parent"

if [[ -e "$target_dir" || -L "$target_dir" ]]; then
  existing_link=""
  if [[ -L "$target_dir" ]]; then
    existing_link="$(readlink "$target_dir")"
  fi

  if [[ "$force" -ne 1 && "$existing_link" != "$source_dir" ]]; then
    printf 'error: %s already exists; pass --force to replace it\n' "$target_dir" >&2
    exit 1
  fi

  rm -rf "$target_dir"
fi

case "$mode" in
  copy)
    cp -R "$source_dir" "$target_dir"
    printf 'Installed aget Codex skill at %s\n' "$target_dir"
    ;;
  symlink)
    ln -s "$source_dir" "$target_dir"
    printf 'Installed aget Codex skill symlink: %s -> %s\n' "$target_dir" "$source_dir"
    ;;
esac

printf 'Restart Codex to pick up the installed or replaced skill.\n'
