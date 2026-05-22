#!/usr/bin/env python3
"""Extract one URL through Crawl4AI using a Playwright storage-state file."""

import argparse
import asyncio
import contextlib
import html
import inspect
import json
import os
import sys
from html.parser import HTMLParser
from pathlib import Path


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
        from crawl4ai import AsyncWebCrawler, BrowserConfig, CacheMode, CrawlerRunConfig, DefaultMarkdownGenerator
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
        apply_compatible_options(crawler_kwargs, browser_kwargs, extractor_options["crawler"], CrawlerRunConfig)
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


EXTRACTOR_OPTION_TYPES = {
    "base_url": ("crawler", "str"),
    "target_elements": ("crawler", "list"),
    "excluded_tags": ("crawler", "list"),
    "exclude_all_images": ("crawler", "bool"),
    "exclude_domains": ("crawler", "list"),
    "exclude_external_images": ("crawler", "bool"),
    "exclude_external_links": ("crawler", "bool"),
    "exclude_internal_links": ("crawler", "bool"),
    "exclude_social_media_domains": ("crawler", "list"),
    "exclude_social_media_links": ("crawler", "bool"),
    "only_text": ("crawler", "bool"),
    "process_iframes": ("crawler", "bool"),
    "remove_forms": ("crawler", "bool"),
    "remove_overlay_elements": ("crawler", "bool"),
    "keep_data_attributes": ("crawler", "bool"),
    "word_count_threshold": ("crawler", "int"),
    "wait_until": ("crawler", "str"),
    "page_timeout": ("crawler", "int"),
    "wait_for_timeout": ("crawler", "int"),
    "delay_before_return_html": ("crawler", "float"),
    "wait_for_images": ("crawler", "bool"),
    "scan_full_page": ("crawler", "bool"),
    "scroll_delay": ("crawler", "float"),
    "max_scroll_steps": ("crawler", "int"),
    "flatten_shadow_dom": ("crawler", "bool"),
    "bypass_tables": ("markdown", "bool"),
    "close_quote": ("markdown", "str"),
    "default_image_alt": ("markdown", "str"),
    "emphasis_mark": ("markdown", "str"),
    "escape_snob": ("markdown", "bool"),
    "ignore_emphasis": ("markdown", "bool"),
    "ignore_images": ("markdown", "bool"),
    "images_as_html": ("markdown", "bool"),
    "images_to_alt": ("markdown", "bool"),
    "images_with_size": ("markdown", "bool"),
    "ignore_links": ("markdown", "bool"),
    "ignore_mailto_links": ("markdown", "bool"),
    "ignore_tables": ("markdown", "bool"),
    "include_sup_sub": ("markdown", "bool"),
    "open_quote": ("markdown", "str"),
    "protect_links": ("markdown", "bool"),
    "skip_internal_links": ("markdown", "bool"),
    "strong_mark": ("markdown", "str"),
    "ul_item_mark": ("markdown", "str"),
    "use_automatic_links": ("markdown", "bool"),
}

JS_WAIT_MARKERS = ("=>", "function(", "return ", ";")


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


def parse_extractor_options(values: list[str]) -> dict[str, dict[str, object]]:
    parsed = {"browser": {}, "crawler": {}, "markdown": {}}
    for value in values:
        if "=" not in value:
            raise ValueError(f"extractor option must use key=value form: {value}")
        key, raw_value = value.split("=", 1)
        namespace = "crawl4ai."
        if not key.startswith(namespace):
            raise ValueError(f"extractor option '{key}' must use the crawl4ai.<key> namespace")
        key = key[len(namespace):]
        if key not in EXTRACTOR_OPTION_TYPES:
            allowed = ", ".join(sorted(EXTRACTOR_OPTION_TYPES))
            raise ValueError(f"unsupported extractor option '{key}'; supported keys: {allowed}")
        target, value_type = EXTRACTOR_OPTION_TYPES[key]
        parsed[target][key] = parse_extractor_value(key, raw_value, value_type)
    return parsed


def validate_wait_for(value: str) -> None:
    normalized = value.strip().lower()
    if normalized.startswith("js:") or any(marker in normalized for marker in JS_WAIT_MARKERS):
        raise ValueError("--wait-for-selector only supports CSS selectors in v1; JavaScript wait conditions are not allowed")


def parse_extractor_value(key: str, value: str, value_type: str):
    if value_type == "list":
        return [item.strip() for item in value.split(",") if item.strip()]
    if value_type == "bool":
        normalized = value.strip().lower()
        if normalized in {"true", "1", "yes", "on"}:
            return True
        if normalized in {"false", "0", "no", "off"}:
            return False
        raise ValueError(f"extractor option '{key}' expects a boolean value")
    if value_type == "int":
        try:
            return int(value)
        except ValueError as error:
            raise ValueError(f"extractor option '{key}' expects an integer value") from error
    if value_type == "float":
        try:
            return float(value)
        except ValueError as error:
            raise ValueError(f"extractor option '{key}' expects a numeric value") from error
    return value


def apply_compatible_options(target_kwargs: dict, other_kwargs: dict, options: dict, config_type) -> None:
    if not options:
        return
    signature = inspect.signature(config_type)
    accepted = set(signature.parameters)
    for key, value in options.items():
        if key in accepted:
            target_kwargs[key] = value
        elif key not in other_kwargs:
            raise ValueError(f"extractor option '{key}' is not supported by installed Crawl4AI {config_type.__name__}")


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


def select_content(result, output_format: str) -> str:
    if output_format == "markdown":
        markdown = getattr(result, "markdown", None)
        raw_markdown = getattr(markdown, "raw_markdown", None)
        if raw_markdown is not None:
            return str(raw_markdown)
        return "" if markdown is None else str(markdown)
    if output_format == "html":
        return str(getattr(result, "cleaned_html", None) or getattr(result, "html", None) or "")
    if output_format == "text":
        extracted = getattr(result, "extracted_content", None)
        if extracted is not None:
            return str(extracted)
        html_content = getattr(result, "cleaned_html", None) or getattr(result, "html", None)
        if html_content:
            return html_to_text(str(html_content))
        markdown = getattr(result, "markdown", None)
        raw_markdown = getattr(markdown, "raw_markdown", None)
        return str(raw_markdown or markdown or "")
    if output_format == "json":
        extracted = getattr(result, "extracted_content", None)
        if extracted is not None:
            return str(extracted)
        return json.dumps(
            {
                "url": getattr(result, "url", None),
                "markdown": select_content(result, "markdown"),
            },
            sort_keys=True,
        )
    raise ValueError(f"unsupported output format: {output_format}")


class TextExtractor(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=False)
        self.parts = []
        self.skip_depth = 0

    def handle_starttag(self, tag, attrs):
        if tag in {"script", "style", "noscript"}:
            self.skip_depth += 1
        if tag in {"p", "div", "section", "article", "main", "br", "li", "tr", "h1", "h2", "h3", "h4", "h5", "h6"}:
            self.parts.append("\n")

    def handle_endtag(self, tag):
        if tag in {"script", "style", "noscript"} and self.skip_depth > 0:
            self.skip_depth -= 1
        if tag in {"p", "div", "section", "article", "main", "li", "tr", "h1", "h2", "h3", "h4", "h5", "h6"}:
            self.parts.append("\n")

    def handle_data(self, data):
        if self.skip_depth == 0:
            self.parts.append(data)

    def handle_entityref(self, name):
        if self.skip_depth == 0:
            self.parts.append(html.unescape(f"&{name};"))

    def handle_charref(self, name):
        if self.skip_depth == 0:
            self.parts.append(html.unescape(f"&#{name};"))


def html_to_text(html_content: str) -> str:
    parser = TextExtractor()
    parser.feed(html_content)
    parser.close()
    lines = [" ".join(line.split()) for line in "".join(parser.parts).splitlines()]
    return "\n".join(line for line in lines if line)


if __name__ == "__main__":
    raise SystemExit(asyncio.run(run()))
