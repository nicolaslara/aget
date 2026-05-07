#!/usr/bin/env python3
import asyncio
import sys
from pathlib import Path

from crawl4ai import AsyncWebCrawler, BrowserConfig, CacheMode, CrawlerRunConfig


async def main() -> int:
    if len(sys.argv) != 3:
        print("Usage: crawl4ai-skill-minimal.py <url> <output-dir>", file=sys.stderr)
        return 2

    url = sys.argv[1]
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(parents=True, exist_ok=True)

    browser_config = BrowserConfig(
        headless=True,
        viewport_width=1920,
        viewport_height=1080,
        verbose=True,
    )
    crawler_config = CrawlerRunConfig(
        cache_mode=CacheMode.BYPASS,
        wait_for="css:body",
        page_timeout=60000,
        remove_overlay_elements=True,
    )

    async with AsyncWebCrawler(config=browser_config) as crawler:
        result = await crawler.arun(url=url, config=crawler_config)

    (output_dir / "status.txt").write_text(
        f"success={result.success}\n"
        f"url={result.url}\n"
        f"title={result.metadata.get('title', '')}\n"
        f"markdown_len={len(result.markdown or '')}\n"
        f"html_len={len(result.html or '')}\n"
        f"error={result.error_message or ''}\n",
        encoding="utf-8",
    )
    (output_dir / "output.md").write_text(result.markdown or "", encoding="utf-8")
    (output_dir / "output.html").write_text(result.html or "", encoding="utf-8")
    return 0 if result.success else 1


if __name__ == "__main__":
    raise SystemExit(asyncio.run(main()))
