#!/usr/bin/env python3
"""Extract one URL through Crawl4AI using a Playwright storage-state file."""

import argparse
import asyncio
import contextlib
import json
import sys
from pathlib import Path

from aget_crawl4ai_compat.content import select_content
from aget_crawl4ai_compat.files import write_private_text
from aget_crawl4ai_compat.options import (
    apply_compatible_options,
    parse_extractor_options,
    validate_wait_for,
)


async def run() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", required=True)
    parser.add_argument("--state", required=True, help="Path to Playwright storage_state JSON")
    parser.add_argument("--output", required=True, help="Markdown output path")
    parser.add_argument("--metadata", required=True, help="Backend metadata output path")
    parser.add_argument("--format", choices=["markdown", "html", "text", "json"], default="markdown")
    parser.add_argument("--selector", default=None)
    parser.add_argument("--exclude-selector", default=None)
    parser.add_argument("--wait-for", default="css:body")
    parser.add_argument("--extractor-option", action="append", default=[])
    parser.add_argument("--channel", default=None)
    args = parser.parse_args()

    state_path = Path(args.state).expanduser().resolve()
    output_path = Path(args.output).expanduser().resolve()
    metadata_path = Path(args.metadata).expanduser().resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    metadata_path.parent.mkdir(parents=True, exist_ok=True)

    try:
        extractor_options = parse_extractor_options(args.extractor_option)
        validate_wait_for(args.wait_for)
    except ValueError as error:
        return fail(metadata_path, str(error))

    try:
        from crawl4ai import (
            AsyncWebCrawler,
            BrowserConfig,
            CacheMode,
            CrawlerRunConfig,
            DefaultMarkdownGenerator,
        )
    except Exception as error:
        return fail(metadata_path, f"Crawl4AI import failed: {error}")

    browser_kwargs = {
        "headless": True,
        "browser_type": "chromium",
        "storage_state": str(state_path),
        "viewport_width": 1920,
        "viewport_height": 1080,
    }
    if args.channel:
        browser_kwargs["channel"] = args.channel
    try:
        apply_compatible_options(browser_kwargs, {}, extractor_options["browser"], BrowserConfig)
    except ValueError as error:
        return fail(metadata_path, str(error))

    browser_config = BrowserConfig(**browser_kwargs)
    crawler_kwargs = {
        "cache_mode": CacheMode.BYPASS,
        "wait_for": args.wait_for,
        "remove_overlay_elements": True,
    }
    if args.selector:
        crawler_kwargs["css_selector"] = args.selector
    if args.exclude_selector:
        crawler_kwargs["excluded_selector"] = args.exclude_selector
    try:
        apply_compatible_options(
            crawler_kwargs,
            browser_kwargs,
            extractor_options["crawler"],
            CrawlerRunConfig,
        )
    except ValueError as error:
        return fail(metadata_path, str(error))
    if extractor_options["markdown"]:
        crawler_kwargs["markdown_generator"] = DefaultMarkdownGenerator(options=extractor_options["markdown"])
    crawler_config = CrawlerRunConfig(**crawler_kwargs)

    with contextlib.redirect_stdout(sys.stderr):
        async with AsyncWebCrawler(config=browser_config) as crawler:
            result = await crawler.arun(url=args.url, config=crawler_config)

    content = select_content(result, args.format)
    final_url = getattr(result, "url", None) or args.url
    response = {
        "ok": bool(result.success),
        "final_url": final_url,
        "content": content if result.success else None,
        "page_metadata": getattr(result, "metadata", None) or {},
        "warnings": [],
        "error": None if result.success else (result.error_message or "Crawl4AI extraction failed"),
    }

    if result.success:
        write_private_text(output_path, content)

    write_private_text(metadata_path, json.dumps(response, indent=2, sort_keys=True) + "\n")
    print(json.dumps(response, sort_keys=True))
    return 0 if result.success else 1


def fail(metadata_path: Path, message: str) -> int:
    response = {
        "ok": False,
        "final_url": None,
        "content": None,
        "page_metadata": {},
        "warnings": [],
        "error": message,
    }
    write_private_text(metadata_path, json.dumps(response, indent=2, sort_keys=True) + "\n")
    print(json.dumps(response, sort_keys=True))
    return 1


if __name__ == "__main__":
    raise SystemExit(asyncio.run(run()))
