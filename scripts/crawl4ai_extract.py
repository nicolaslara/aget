#!/usr/bin/env python3
"""Extract one URL through Crawl4AI using a Playwright storage-state file."""

import argparse
import asyncio
import contextlib
import json
import os
import sys
from pathlib import Path

from crawl4ai import AsyncWebCrawler, BrowserConfig, CacheMode, CrawlerRunConfig


async def run() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", required=True)
    parser.add_argument("--state", required=True, help="Path to Playwright storage_state JSON")
    parser.add_argument("--output", required=True, help="Markdown output path")
    parser.add_argument("--metadata", required=True, help="Backend metadata output path")
    parser.add_argument("--channel", default=None)
    args = parser.parse_args()

    state_path = Path(args.state).expanduser().resolve()
    output_path = Path(args.output).expanduser().resolve()
    metadata_path = Path(args.metadata).expanduser().resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    metadata_path.parent.mkdir(parents=True, exist_ok=True)

    browser_kwargs = {
        "headless": True,
        "browser_type": "chromium",
        "storage_state": str(state_path),
        "viewport_width": 1920,
        "viewport_height": 1080,
    }
    if args.channel:
        browser_kwargs["channel"] = args.channel

    browser_config = BrowserConfig(**browser_kwargs)
    crawler_config = CrawlerRunConfig(
        cache_mode=CacheMode.BYPASS,
        wait_for="css:body",
        remove_overlay_elements=True,
    )

    with contextlib.redirect_stdout(sys.stderr):
        async with AsyncWebCrawler(config=browser_config) as crawler:
            result = await crawler.arun(url=args.url, config=crawler_config)

    content = result.markdown or ""
    final_url = getattr(result, "url", None) or args.url
    response = {
        "ok": bool(result.success),
        "final_url": final_url,
        "content": content if result.success else None,
        "warnings": [],
        "error": None if result.success else (result.error_message or "Crawl4AI extraction failed"),
    }

    if result.success:
        write_private_text(output_path, content)

    write_private_text(metadata_path, json.dumps(response, indent=2, sort_keys=True) + "\n")
    print(json.dumps(response, sort_keys=True))
    return 0 if result.success else 1


def write_private_text(path: Path, content: str) -> None:
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    try:
        if hasattr(os, "fchmod"):
            os.fchmod(fd, 0o600)
        else:
            os.chmod(path, 0o600)
        with os.fdopen(fd, "w", encoding="utf-8") as file:
            fd = -1
            file.write(content)
    finally:
        if fd >= 0:
            os.close(fd)


if __name__ == "__main__":
    raise SystemExit(asyncio.run(run()))
