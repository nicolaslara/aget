use crate::session::agent_browser::{domain_allowed, domain_matches_allowed};
use crate::session::Session;

pub fn merge_login_session(mut existing: Session, mut fresh: Session) -> Session {
    let authorized_domains = fresh.allowed_cookie_domains.clone();
    let authorized_origins = fresh.allowed_storage_origins.clone();

    existing
        .cookies
        .retain(|cookie| !domain_allowed(&cookie.domain, &authorized_domains));
    existing.origins.retain(|origin| {
        !authorized_origins
            .iter()
            .any(|allowed| allowed == &origin.origin)
    });
    existing.allowed_cookie_domains.retain(|domain| {
        !authorized_domains
            .iter()
            .any(|allowed| domain_matches_allowed(domain, allowed))
    });
    existing
        .allowed_storage_origins
        .retain(|origin| !authorized_origins.iter().any(|allowed| allowed == origin));

    existing.source = fresh.source;
    existing.sensitive = existing.sensitive || fresh.sensitive;
    existing
        .allowed_cookie_domains
        .append(&mut fresh.allowed_cookie_domains);
    existing.allowed_cookie_domains.sort();
    existing.allowed_cookie_domains.dedup();
    existing
        .allowed_storage_origins
        .append(&mut fresh.allowed_storage_origins);
    existing.allowed_storage_origins.sort();
    existing.allowed_storage_origins.dedup();
    existing.cookies.append(&mut fresh.cookies);
    existing.origins.append(&mut fresh.origins);
    existing
}
