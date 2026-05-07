#!/usr/bin/env python3
"""Run Crawl4AI with an existing Playwright-compatible storage state file."""

import argparse
import asyncio
from pathlib import Path

from crawl4ai import AsyncWebCrawler, BrowserConfig, CacheMode, CrawlerRunConfig


async def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", required=True)
    parser.add_argument("--state", required=True, help="Path to storage_state JSON")
    parser.add_argument("--output", required=True)
    parser.add_argument("--channel", default="chrome")
    args = parser.parse_args()

    state_path = Path(args.state).expanduser().resolve()
    output_path = Path(args.output).expanduser().resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)

    browser_config = BrowserConfig(
        headless=True,
        browser_type="chromium",
        channel=args.channel,
        storage_state=str(state_path),
        viewport_width=1920,
        viewport_height=1080,
    )
    crawler_config = CrawlerRunConfig(
        cache_mode=CacheMode.BYPASS,
        remove_overlay_elements=True,
    )

    async with AsyncWebCrawler(config=browser_config) as crawler:
        result = await crawler.arun(url=args.url, config=crawler_config)

    if not result.success:
        raise SystemExit(result.error_message)

    output_path.write_text(result.markdown or "", encoding="utf-8")
    print(f"Wrote {output_path} ({len(result.markdown or '')} chars)")


if __name__ == "__main__":
    asyncio.run(main())
