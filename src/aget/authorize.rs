use std::path::PathBuf;

use crate::extraction::GetSuccess;

#[derive(Debug, Clone)]
pub struct AuthorizeSessionOptions {
    pub name: String,
    pub url: String,
    pub chrome_profile: String,
    pub allow_domains: Vec<String>,
    pub must_contain: Vec<String>,
    pub must_not_contain: Vec<String>,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthorizeSessionResult {
    pub state: AuthorizationState,
    pub name: String,
    pub source: &'static str,
    pub allowed_domains: Vec<String>,
    pub baseline: GetSuccess,
    pub verification: GetSuccess,
    pub predicates: Vec<AuthorizationPredicateResult>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationState {
    Verified,
    VerificationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AuthorizationPredicateResult {
    pub kind: &'static str,
    pub value: String,
    pub matched: bool,
}

pub(super) fn evaluate_authorization_predicates(
    content: &str,
    must_contain: &[String],
    must_not_contain: &[String],
) -> Vec<AuthorizationPredicateResult> {
    let mut predicates = must_contain
        .iter()
        .map(|value| AuthorizationPredicateResult {
            kind: "must_contain",
            value: value.clone(),
            matched: content.contains(value),
        })
        .collect::<Vec<_>>();
    predicates.extend(
        must_not_contain
            .iter()
            .map(|value| AuthorizationPredicateResult {
                kind: "must_not_contain",
                value: value.clone(),
                matched: !content.contains(value),
            }),
    );
    predicates
}
