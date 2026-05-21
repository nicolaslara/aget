use crate::error::AgetError;
use crate::extraction::{BrowserFallbackRequest, BrowserFallbackResult};
use crate::session::{
    cancel_owned_login_session, finish_owned_login_session, import_owned_chrome_session,
    start_owned_login_session, ChromeImportOptions, LoginCancelOptions, LoginCancelResult,
    LoginFinishOptions, LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
};

/// Local browser/CDP engine for the `agent-browser`-like behavior that `aget`
/// now owns. The methods currently delegate to the migrated implementation
/// slices while the engine boundary is introduced mechanically.
#[derive(Clone, Debug, Default)]
pub struct AgetBrowser;

impl AgetBrowser {
    pub(crate) fn import_chrome_session(
        &self,
        options: ChromeImportOptions,
    ) -> Result<Session, AgetError> {
        import_owned_chrome_session(options)
    }

    pub(crate) fn start_login_session(
        &self,
        options: LoginStartOptions,
    ) -> Result<LoginStartResult, AgetError> {
        start_owned_login_session(options)
    }

    pub(crate) fn finish_login_session(
        &self,
        options: LoginFinishOptions,
    ) -> Result<LoginFinishResult, AgetError> {
        finish_owned_login_session(options)
    }

    pub(crate) fn cancel_login_session(
        &self,
        options: LoginCancelOptions,
    ) -> Result<LoginCancelResult, AgetError> {
        cancel_owned_login_session(options)
    }

    pub(crate) fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        crate::extraction::run_owned_browser_fallback(request)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::session::login::PendingLogin;

    #[test]
    fn cancels_pending_login_without_aget_facade_or_command_backend() {
        let temp = tempfile::tempdir().unwrap();
        let tmp_dir = temp.path().join("tmp");
        let profile = tmp_dir.join("owned-login/aget-docs");
        fs::create_dir_all(&profile).unwrap();
        fs::write(
            tmp_dir.join("login-docs.json"),
            serde_json::to_vec_pretty(&PendingLogin {
                name: "docs".to_string(),
                profile: profile.to_string_lossy().into_owned(),
                agent_session: "aget-login-docs".to_string(),
                url: "https://example.com/login".to_string(),
                allowed_domains: vec!["example.com".to_string()],
                browser_pid: None,
            })
            .unwrap(),
        )
        .unwrap();

        let cancelled = AgetBrowser::default()
            .cancel_login_session(LoginCancelOptions {
                name: "docs".to_string(),
                tmp_dir: tmp_dir.clone(),
            })
            .unwrap();

        assert_eq!(cancelled.pending.name, "docs");
        assert!(!profile.exists());
        assert!(!tmp_dir.join("login-docs.json").exists());
    }
}
