pub(super) fn domain_allowed(cookie_domain: &str, allowed_domains: &[String]) -> bool {
    allowed_domains
        .iter()
        .any(|allowed| domain_matches_allowed(cookie_domain, allowed))
}

fn domain_matches_allowed(cookie_domain: &str, allowed_domain: &str) -> bool {
    let cookie = normalize_domain(cookie_domain);
    let allowed = normalize_domain(allowed_domain);
    if cookie.is_empty() || allowed.is_empty() {
        return false;
    }

    if cookie == allowed {
        return true;
    }

    if let Some(cookie_root) = cookie.strip_prefix('.') {
        return cookie_root == allowed;
    }

    is_domain_or_subdomain(&cookie, &allowed)
}

fn is_domain_or_subdomain(candidate: &str, root: &str) -> bool {
    candidate == root || candidate.ends_with(&format!(".{root}"))
}

pub(super) fn normalize_domain(domain: &str) -> String {
    domain.trim().trim_end_matches('.').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_matching_rejects_substring_only_matches() {
        assert!(domain_matches_allowed("example.com", "example.com"));
        assert!(domain_matches_allowed(".example.com", "example.com"));
        assert!(!domain_matches_allowed(".example.com", "docs.example.com"));
        assert!(domain_matches_allowed(
            ".docs.example.com",
            "docs.example.com"
        ));
        assert!(domain_matches_allowed("docs.example.com", "example.com"));
        assert!(!domain_matches_allowed("example.com", "docs.example.com"));
        assert!(!domain_matches_allowed("evil-example.com", "example.com"));
        assert!(!domain_matches_allowed("example.com.evil", "example.com"));
    }
}
