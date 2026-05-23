use url::Url;

pub(super) fn is_crawl4ai_like_external_url(
    raw_url: &str,
    base_url: &str,
    base_domain: &str,
) -> bool {
    let raw_url = raw_url.trim();
    if is_crawl4ai_special_url(raw_url) {
        return true;
    }
    let Ok(url) = Url::parse(raw_url)
        .or_else(|_| Url::parse(base_url).and_then(|base_url| base_url.join(raw_url)))
    else {
        return false;
    };
    let Some(url_host) = url.host_str() else {
        return false;
    };
    let url_domain = normalize_crawl4ai_domain(url_host);
    !url_domain.ends_with(base_domain)
}

pub(super) fn crawl4ai_like_base_domain(raw_url: &str) -> String {
    Url::parse(raw_url)
        .ok()
        .and_then(|url| url.host_str().map(normalize_crawl4ai_domain))
        .map(|domain| {
            let parts = domain.split('.').collect::<Vec<_>>();
            if parts.len() > 2
                && matches!(
                    parts[parts.len() - 2],
                    "co" | "com"
                        | "org"
                        | "gov"
                        | "edu"
                        | "net"
                        | "mil"
                        | "int"
                        | "ac"
                        | "ad"
                        | "ae"
                        | "af"
                        | "ag"
                )
            {
                parts[parts.len() - 3..].join(".")
            } else if parts.len() >= 2 {
                parts[parts.len() - 2..].join(".")
            } else {
                domain
            }
        })
        .unwrap_or_default()
}

pub(super) fn crawl4ai_like_url_base_domain(raw_url: &str, base_url: &str) -> Option<String> {
    Url::parse(raw_url)
        .or_else(|_| Url::parse(base_url).and_then(|base_url| base_url.join(raw_url)))
        .ok()
        .map(|url| crawl4ai_like_base_domain(url.as_str()))
        .filter(|domain| !domain.is_empty())
}

pub(super) fn crawl4ai_like_domain_from_option(raw_domain: &str) -> Option<String> {
    let raw_domain = raw_domain.trim();
    if raw_domain.is_empty() {
        return None;
    }
    if let Ok(url) = Url::parse(raw_domain) {
        return Some(crawl4ai_like_base_domain(url.as_str())).filter(|domain| !domain.is_empty());
    }
    let domain = raw_domain
        .split('/')
        .next()
        .unwrap_or(raw_domain)
        .split(':')
        .next()
        .unwrap_or(raw_domain);
    Some(crawl4ai_like_base_domain(&format!("https://{domain}")))
        .filter(|domain| !domain.is_empty())
}

fn is_crawl4ai_special_url(raw_url: &str) -> bool {
    let lower = raw_url.to_ascii_lowercase();
    ["mailto:", "tel:", "ftp:", "file:", "data:", "javascript:"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

fn normalize_crawl4ai_domain(host: &str) -> String {
    let domain = host.trim_end_matches('.').to_ascii_lowercase();
    domain.strip_prefix("www.").unwrap_or(&domain).to_string()
}
