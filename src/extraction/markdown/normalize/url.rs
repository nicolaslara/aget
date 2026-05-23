use url::Url;

pub(in crate::extraction::markdown) fn is_absolute_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

pub(in crate::extraction) fn resolve_markdown_url(base: &str, raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("mailto:") {
        return raw.to_string();
    }
    Url::parse(base)
        .and_then(|base| base.join(raw))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| raw.to_string())
}
