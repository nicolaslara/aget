#!/usr/bin/env python3
"""
Proof-of-concept: copy a normal Chrome profile into Crawl4AI's profile store.

This does not read credentials into agent context. It copies browser state on disk
after the user has logged in with normal Chrome and quit Chrome. The destination
is a Crawl4AI-owned profile directory suitable for `crwl crawl --profile <name>`.
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path


EXCLUDE_DIRS = {
    "Cache",
    "Code Cache",
    "GPUCache",
    "ShaderCache",
    "GrShaderCache",
    "GraphiteDawnCache",
    "Crashpad",
    "component_crx_cache",
    "optimization_guide",
    "blob_storage",
    "File System",
    "GCM Store",
}

EXCLUDE_PREFIXES = (
    "Singleton",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Copy Chrome profile into Crawl4AI profiles")
    parser.add_argument(
        "--chrome-user-data-dir",
        default=str(Path.home() / "Library/Application Support/Google/Chrome"),
        help="Chrome user data directory",
    )
    parser.add_argument("--chrome-profile", default="Default", help="Chrome profile directory")
    parser.add_argument("--as-profile", required=True, help="Crawl4AI destination profile name")
    return parser.parse_args()


def should_skip(path: Path) -> bool:
    name = path.name
    return name in EXCLUDE_DIRS or any(name.startswith(prefix) for prefix in EXCLUDE_PREFIXES)


def copy_tree(src: Path, dst: Path) -> None:
    dst.mkdir(parents=True, exist_ok=True)
    for child in src.iterdir():
        if should_skip(child):
            continue

        target = dst / child.name
        try:
            if child.is_dir():
                copy_tree(child, target)
            elif child.is_file():
                shutil.copy2(child, target)
        except OSError as exc:
            print(f"warning: skipped {child}: {exc}", file=sys.stderr)


def main() -> int:
    args = parse_args()
    source_root = Path(args.chrome_user_data_dir).expanduser()
    source_profile = source_root / args.chrome_profile
    dest_root = Path.home() / ".crawl4ai" / "profiles" / args.as_profile

    if not source_root.is_dir():
        print(f"Chrome user data dir not found: {source_root}", file=sys.stderr)
        return 1
    if not source_profile.is_dir():
        print(f"Chrome profile dir not found: {source_profile}", file=sys.stderr)
        return 1
    if dest_root.exists():
        print(f"Destination already exists: {dest_root}", file=sys.stderr)
        print("Use a new --as-profile name for this proof-of-concept.", file=sys.stderr)
        return 1

    dest_root.mkdir(parents=True)

    local_state = source_root / "Local State"
    if local_state.is_file():
        shutil.copy2(local_state, dest_root / "Local State")
    else:
        print(f"warning: Local State not found: {local_state}", file=sys.stderr)

    copy_tree(source_profile, dest_root / args.chrome_profile)

    print(f"Copied Chrome profile '{args.chrome_profile}' to Crawl4AI profile '{args.as_profile}'")
    print(f"Destination: {dest_root}")
    print(f"Try: uvx --from crawl4ai crwl crawl <url> --profile {args.as_profile} --output markdown")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
