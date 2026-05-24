pub(crate) fn domain_allowed(candidate_domain: &str, allowed_domains: &[String]) -> bool {
    allowed_domains
        .iter()
        .any(|allowed| domain_matches_allowed(candidate_domain, allowed))
}

pub(crate) fn domain_matches_allowed(candidate_domain: &str, allowed_domain: &str) -> bool {
    let candidate = normalize_domain(candidate_domain);
    let allowed = normalize_domain(allowed_domain);
    if candidate.is_empty() || allowed.is_empty() {
        return false;
    }

    if candidate == allowed {
        return true;
    }

    if let Some(candidate_root) = candidate.strip_prefix('.') {
        return candidate_root == allowed;
    }

    candidate.ends_with(&format!(".{allowed}"))
}

pub(crate) fn origin_host(origin: &str) -> Option<String> {
    let (_, rest) = origin.split_once("://")?;
    let authority = rest.split('/').next().unwrap_or(rest);
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        stripped.split(']').next()?
    } else {
        host_port.split(':').next().unwrap_or(host_port)
    };
    let host = normalize_domain(host);
    (!host.is_empty()).then_some(host)
}

pub(super) fn normalize_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}
