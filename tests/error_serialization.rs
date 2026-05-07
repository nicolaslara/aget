use aget::{ErrorCode, ErrorResponse};

#[test]
fn all_error_codes_serialize_to_stable_strings() {
    let cases = [
        (ErrorCode::UsageError, "usage_error"),
        (ErrorCode::BackendUnavailable, "backend_unavailable"),
        (ErrorCode::Timeout, "timeout"),
        (ErrorCode::RequiresUserAction, "requires_user_action"),
        (ErrorCode::AuthFailed, "auth_failed"),
        (ErrorCode::ExtractionFailed, "extraction_failed"),
        (ErrorCode::SessionConflict, "session_conflict"),
        (ErrorCode::PrivacyPolicyBlocked, "privacy_policy_blocked"),
    ];

    for (code, expected) in cases {
        let serialized = serde_json::to_value(code).unwrap();
        assert_eq!(serialized, expected);
    }
}

#[test]
fn error_response_has_stable_shape() {
    let response = ErrorResponse::new(ErrorCode::Timeout, "command timed out");
    let value = serde_json::to_value(response).unwrap();

    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "timeout");
    assert_eq!(value["error"]["message"], "command timed out");
}
