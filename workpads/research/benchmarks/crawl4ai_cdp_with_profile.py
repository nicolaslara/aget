#!/usr/bin/env python3
"""
Proof-of-concept: run Crawl4AI extraction against a system Chrome process that
uses a copied Chrome user-data-dir.

This mirrors the part of agent-browser that matters for macOS auth: use real
Google Chrome with the real Keychain rather than Playwright's bundled Chromium
or a mock keychain.
"""

from __future__ import annotations

import argparse
import asyncio
import socket
import subprocess
import time
from pathlib import Path

from crawl4ai import AsyncWebCrawler, BrowserConfig, CacheMode, CrawlerRunConfig


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Crawl with Crawl4AI via CDP and copied Chrome profile")
    parser.add_argument("--profile-dir", required=True, help="Copied Chrome user-data-dir")
    parser.add_argument("--profile-directory", default="Default", help="Chrome profile directory inside user-data-dir")
    parser.add_argument("--url", required=True, help="URL to crawl")
    parser.add_argument("--output", required=True, help="Markdown output path")
    parser.add_argument(
        "--chrome",
        default="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        help="Google Chrome executable path",
    )
    return parser.parse_args()


async def wait_for_cdp(port: int, timeout: float = 15.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            reader, writer = await asyncio.open_connection("127.0.0.1", port)
            writer.close()
            await writer.wait_closed()
            return
        except OSError:
            await asyncio.sleep(0.2)
    raise TimeoutError(f"Chrome CDP did not open on port {port}")


async def main() -> int:
    args = parse_args()
    profile_dir = Path(args.profile_dir).expanduser()
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)

    port = free_port()
    chrome_args = [
        args.chrome,
        f"--remote-debugging-port={port}",
        f"--user-data-dir={profile_dir}",
        f"--profile-directory={args.profile_directory}",
        "--no-first-run",
        "--no-default-browser-check",
        "--disable-background-networking",
        "--disable-sync",
        "--headless=new",
    ]

    proc = subprocess.Popen(chrome_args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        await wait_for_cdp(port)
        browser_config = BrowserConfig(
            cdp_url=f"http://127.0.0.1:{port}",
            headless=True,
            verbose=True,
        )
        crawler_config = CrawlerRunConfig(
            cache_mode=CacheMode.BYPASS,
            page_timeout=60000,
        )
        async with AsyncWebCrawler(config=browser_config) as crawler:
            result = await crawler.arun(args.url, config=crawler_config)

        output.write_text(result.markdown or "", encoding="utf-8")
        (output.with_suffix(output.suffix + ".status.txt")).write_text(
            f"success={result.success}\n"
            f"url={result.url}\n"
            f"title={(result.metadata or {}).get('title', '')}\n"
            f"markdown_len={len(result.markdown or '')}\n"
            f"error={result.error_message or ''}\n",
            encoding="utf-8",
        )
        return 0 if result.success else 1
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)


if __name__ == "__main__":
    raise SystemExit(asyncio.run(main()))
