#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/install-agent-integrations.sh [targets] [options]

Install aget agent integrations after the aget CLI is installed.

With no target flags, installs global skill integrations for Codex, Claude Code,
Gemini CLI, and Windsurf. Project-local adapters are installed only when
--project-dir is provided or when their target flag is used with --project-dir.

Targets:
  --all             Install all supported integrations. Project-local adapters
                    require --project-dir.
  --codex          Install skills/aget into $CODEX_HOME/skills/aget.
  --claude         Install skills/aget into ~/.claude/skills/aget.
  --gemini         Install skills/aget into ~/.gemini/skills/aget.
  --windsurf       Install skills/aget into ~/.codeium/windsurf/skills/aget.
  --opencode       Install project-local OpenCode tools into .opencode/.
  --cursor         Install a project Cursor rule into .cursor/rules/.
  --copilot        Install a project Copilot instruction file into .github/instructions/.

Options:
  --copy            Copy directories/files. This is the default.
  --symlink         Symlink directories/files from this checkout.
  --force           Replace existing aget integrations.
  --project-dir DIR Install project-local adapters into DIR.
  --codex-home DIR  Use DIR instead of $CODEX_HOME or $HOME/.codex.
  --claude-home DIR Use DIR instead of $HOME/.claude.
  --gemini-home DIR Use DIR instead of $HOME/.gemini.
  --windsurf-home DIR
                    Use DIR instead of $HOME/.codeium/windsurf.
  -h, --help        Show this help.

Examples:
  scripts/install-agent-integrations.sh --all --force
  scripts/install-agent-integrations.sh --all --project-dir /path/to/repo --force
  scripts/install-agent-integrations.sh --codex --claude --symlink --force
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
skill_source="$repo_root/skills/aget"
cursor_source="$repo_root/integrations/cursor/aget.mdc"
copilot_source="$repo_root/integrations/copilot/aget.instructions.md"
opencode_tool_source="$repo_root/.opencode/tools/aget.ts"
opencode_lib_source="$repo_root/.opencode/lib/aget_args.ts"

mode="copy"
force=0
all=0
codex=0
claude=0
gemini=0
windsurf=0
opencode=0
cursor=0
copilot=0
target_flags=0
project_dir=""
codex_home="${CODEX_HOME:-$HOME/.codex}"
claude_home="$HOME/.claude"
gemini_home="$HOME/.gemini"
windsurf_home="$HOME/.codeium/windsurf"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all)
      all=1
      target_flags=1
      shift
      ;;
    --codex)
      codex=1
      target_flags=1
      shift
      ;;
    --claude)
      claude=1
      target_flags=1
      shift
      ;;
    --gemini)
      gemini=1
      target_flags=1
      shift
      ;;
    --windsurf)
      windsurf=1
      target_flags=1
      shift
      ;;
    --opencode)
      opencode=1
      target_flags=1
      shift
      ;;
    --cursor)
      cursor=1
      target_flags=1
      shift
      ;;
    --copilot)
      copilot=1
      target_flags=1
      shift
      ;;
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
    --project-dir)
      if [[ $# -lt 2 ]]; then
        printf 'error: --project-dir requires a directory\n' >&2
        exit 2
      fi
      project_dir="$2"
      shift 2
      ;;
    --codex-home)
      if [[ $# -lt 2 ]]; then
        printf 'error: --codex-home requires a directory\n' >&2
        exit 2
      fi
      codex_home="$2"
      shift 2
      ;;
    --claude-home)
      if [[ $# -lt 2 ]]; then
        printf 'error: --claude-home requires a directory\n' >&2
        exit 2
      fi
      claude_home="$2"
      shift 2
      ;;
    --gemini-home)
      if [[ $# -lt 2 ]]; then
        printf 'error: --gemini-home requires a directory\n' >&2
        exit 2
      fi
      gemini_home="$2"
      shift 2
      ;;
    --windsurf-home)
      if [[ $# -lt 2 ]]; then
        printf 'error: --windsurf-home requires a directory\n' >&2
        exit 2
      fi
      windsurf_home="$2"
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

if [[ "$target_flags" -eq 0 || "$all" -eq 1 ]]; then
  codex=1
  claude=1
  gemini=1
  windsurf=1
  if [[ -n "$project_dir" ]]; then
    opencode=1
    cursor=1
    copilot=1
  fi
fi

if [[ ! -f "$skill_source/SKILL.md" ]]; then
  printf 'error: missing source skill: %s\n' "$skill_source/SKILL.md" >&2
  exit 1
fi

if [[ "$opencode" -eq 1 || "$cursor" -eq 1 || "$copilot" -eq 1 ]]; then
  if [[ -z "$project_dir" ]]; then
    printf 'error: --opencode, --cursor, and --copilot require --project-dir DIR\n' >&2
    exit 2
  fi
  project_dir="$(cd "$project_dir" && pwd)"
fi

replace_target() {
  local source="$1"
  local target="$2"
  local label="$3"

  if [[ ! -e "$source" ]]; then
    printf 'error: missing %s source: %s\n' "$label" "$source" >&2
    exit 1
  fi

  mkdir -p "$(dirname "$target")"

  if [[ -e "$target" || -L "$target" ]]; then
    if [[ -L "$target" && "$mode" == "symlink" && "$(readlink "$target")" == "$source" ]]; then
      printf 'Already installed %s: %s -> %s\n' "$label" "$target" "$source"
      return
    fi
    if [[ "$force" -ne 1 ]]; then
      printf 'error: %s already exists; pass --force to replace it\n' "$target" >&2
      exit 1
    fi
    rm -rf "$target"
  fi

  case "$mode" in
    copy)
      cp -R "$source" "$target"
      printf 'Installed %s at %s\n' "$label" "$target"
      ;;
    symlink)
      ln -s "$source" "$target"
      printf 'Installed %s symlink: %s -> %s\n' "$label" "$target" "$source"
      ;;
  esac
}

if ! command -v aget >/dev/null 2>&1; then
  printf 'warning: aget is not on PATH yet; install the CLI before relying on these integrations.\n' >&2
fi

if [[ "$codex" -eq 1 ]]; then
  replace_target "$skill_source" "$codex_home/skills/aget" "Codex skill"
fi

if [[ "$claude" -eq 1 ]]; then
  replace_target "$skill_source" "$claude_home/skills/aget" "Claude Code skill"
fi

if [[ "$gemini" -eq 1 ]]; then
  replace_target "$skill_source" "$gemini_home/skills/aget" "Gemini CLI skill"
fi

if [[ "$windsurf" -eq 1 ]]; then
  replace_target "$skill_source" "$windsurf_home/skills/aget" "Windsurf skill"
fi

if [[ "$opencode" -eq 1 ]]; then
  replace_target "$opencode_tool_source" "$project_dir/.opencode/tools/aget.ts" "OpenCode aget tool"
  replace_target "$opencode_lib_source" "$project_dir/.opencode/lib/aget_args.ts" "OpenCode aget argument helper"
fi

if [[ "$cursor" -eq 1 ]]; then
  replace_target "$cursor_source" "$project_dir/.cursor/rules/aget.mdc" "Cursor aget rule"
fi

if [[ "$copilot" -eq 1 ]]; then
  replace_target "$copilot_source" "$project_dir/.github/instructions/aget.instructions.md" "Copilot aget instructions"
fi

printf 'Restart or reload each agent harness so it discovers newly installed integrations.\n'
