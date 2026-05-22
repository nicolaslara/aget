import inspect


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
    "body_width": ("markdown", "int"),
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
    "mark_code": ("markdown", "bool"),
    "open_quote": ("markdown", "str"),
    "protect_links": ("markdown", "bool"),
    "single_line_break": ("markdown", "bool"),
    "skip_internal_links": ("markdown", "bool"),
    "strong_mark": ("markdown", "str"),
    "ul_item_mark": ("markdown", "str"),
    "unicode_snob": ("markdown", "bool"),
    "use_automatic_links": ("markdown", "bool"),
}

JS_WAIT_MARKERS = ("=>", "function(", "return ", ";")


def parse_extractor_options(values: list[str]) -> dict[str, dict[str, object]]:
    parsed = {"browser": {}, "crawler": {}, "markdown": {}}
    for value in values:
        if "=" not in value:
            raise ValueError(f"extractor option must use key=value form: {value}")
        key, raw_value = value.split("=", 1)
        namespace = "crawl4ai."
        if not key.startswith(namespace):
            raise ValueError(f"extractor option '{key}' must use the crawl4ai.<key> namespace")
        key = key[len(namespace) :]
        if key not in EXTRACTOR_OPTION_TYPES:
            allowed = ", ".join(sorted(EXTRACTOR_OPTION_TYPES))
            raise ValueError(f"unsupported extractor option '{key}'; supported keys: {allowed}")
        target, value_type = EXTRACTOR_OPTION_TYPES[key]
        parsed[target][key] = parse_extractor_value(key, raw_value, value_type)
    return parsed


def validate_wait_for(value: str) -> None:
    normalized = value.strip().lower()
    if normalized.startswith("js:") or any(marker in normalized for marker in JS_WAIT_MARKERS):
        raise ValueError(
            "--wait-for-selector only supports CSS selectors in v1; "
            "JavaScript wait conditions are not allowed"
        )


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


def apply_compatible_options(
    target_kwargs: dict,
    other_kwargs: dict,
    options: dict,
    config_type,
) -> None:
    if not options:
        return
    signature = inspect.signature(config_type)
    accepted = set(signature.parameters)
    for key, value in options.items():
        if key in accepted:
            target_kwargs[key] = value
        elif key not in other_kwargs:
            raise ValueError(
                f"extractor option '{key}' is not supported by installed Crawl4AI "
                f"{config_type.__name__}"
            )
