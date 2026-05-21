use aget::{AuthorizeSessionResult, ErrorResponse, InlineContent};
use serde_json::Value;

pub(super) fn authorize_envelope_data(
    result: &AuthorizeSessionResult,
) -> Result<Value, ErrorResponse> {
    Ok(serde_json::json!({
        "state": result.state,
        "name": result.name,
        "source": result.source,
        "allowed_domains": result.allowed_domains,
        "baseline": super::super::get_envelope_data(&result.baseline, InlineContent::Never)?,
        "verification": super::super::get_envelope_data(&result.verification, InlineContent::Never)?,
        "verification_sensitive": result.verification.sensitive,
        "verification_content_inlined": false,
        "predicates": result.predicates,
        "next_command": if result.state == aget::AuthorizationState::VerificationFailed {
            Some(serde_json::json!([
                "aget",
                "session",
                "authorize",
                result.name,
                "--url",
                result.verification.url,
                "--browser-profile",
                "<profile>",
                "--allow-domain",
                "<domain>",
            ]))
        } else {
            None
        },
    }))
}
