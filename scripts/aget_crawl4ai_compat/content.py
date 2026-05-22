import html
import json
from html.parser import HTMLParser


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
        if tag in {
            "p",
            "div",
            "section",
            "article",
            "main",
            "br",
            "li",
            "tr",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
        }:
            self.parts.append("\n")

    def handle_endtag(self, tag):
        if tag in {"script", "style", "noscript"} and self.skip_depth > 0:
            self.skip_depth -= 1
        if tag in {
            "p",
            "div",
            "section",
            "article",
            "main",
            "li",
            "tr",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
        }:
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
