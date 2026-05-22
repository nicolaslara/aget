use std::io;

use crate::cli::OutputFormat;
use crate::error::{AgetError, ErrorCode};
use crate::extraction::{BrowserFallbackBackend, ExtractorBackend};
use crate::session::{
    complete_login_session, compose_session, import_cmux_session as import_cmux_state,
    merge_login_session, ChromeImportOptions, CmuxImportOptions, LoginCancelOptions,
    LoginCancelResult, LoginCompleteOptions, LoginFinishOptions, LoginStartOptions,
    LoginStartResult, Session,
};

use super::authorize::evaluate_authorization_predicates;
use super::{
    io_aget_error, AgetWith, AuthorizationState, AuthorizeSessionOptions, AuthorizeSessionResult,
    BrowserAutomationBackend, SessionStoreBackend,
};

impl<E, S, B> AgetWith<E, S, B>
where
    E: ExtractorBackend + Clone,
    S: SessionStoreBackend + Clone,
    B: BrowserAutomationBackend + BrowserFallbackBackend + Clone,
{
    pub fn list_sessions(&self) -> Result<Vec<String>, AgetError> {
        self.session_store.list().map_err(io_aget_error)
    }

    pub fn load_session(&self, name: &str) -> Result<Session, AgetError> {
        self.session_store.load(name).map_err(io_aget_error)
    }

    pub fn delete_session(&self, name: &str) -> Result<bool, AgetError> {
        self.session_store.delete(name).map_err(io_aget_error)
    }

    pub fn import_cmux_session(
        &self,
        surface: impl Into<String>,
        name: impl Into<String>,
        domains: Vec<String>,
    ) -> Result<Session, AgetError> {
        let session = import_cmux_state(CmuxImportOptions {
            surface: surface.into(),
            name: name.into(),
            domains,
            tmp_dir: self.tmp_dir(),
        })?;
        self.session_store.save(&session).map_err(io_aget_error)?;
        Ok(session)
    }

    pub fn import_chrome_session(
        &self,
        profile: impl Into<String>,
        name: impl Into<String>,
        domains: Vec<String>,
    ) -> Result<Session, AgetError> {
        let session = self.browser_backend.import_chrome(ChromeImportOptions {
            profile: profile.into(),
            name: name.into(),
            domains,
            tmp_dir: self.tmp_dir(),
        })?;
        self.session_store.save(&session).map_err(io_aget_error)?;
        Ok(session)
    }

    pub fn authorize_chrome_session(
        &self,
        options: AuthorizeSessionOptions,
    ) -> Result<AuthorizeSessionResult, AgetError> {
        let baseline = self
            .get(options.url.clone())
            .content_format(OutputFormat::Markdown)
            .run()?;
        let session = self.import_chrome_session(
            options.chrome_profile,
            options.name,
            options.allow_domains.clone(),
        )?;
        let mut verification_request = self
            .get(options.url)
            .session(session.name.clone())
            .content_format(OutputFormat::Markdown);
        if let Some(output) = options.output {
            verification_request = verification_request.output(output);
        }
        let verification = verification_request.run()?;
        let predicates = evaluate_authorization_predicates(
            &verification.content,
            &options.must_contain,
            &options.must_not_contain,
        );
        let verified = predicates.iter().all(|predicate| predicate.matched);
        let mut warnings = verification.warnings.clone();
        if !verified {
            warnings.push(
                "verification fetch completed, but one or more caller-supplied predicates failed"
                    .to_string(),
            );
        }
        Ok(AuthorizeSessionResult {
            state: if verified {
                AuthorizationState::Verified
            } else {
                AuthorizationState::VerificationFailed
            },
            name: session.name,
            source: "chrome",
            allowed_domains: options.allow_domains,
            baseline,
            verification,
            predicates,
            warnings,
        })
    }

    pub fn compose_sessions(
        &self,
        name: impl Into<String>,
        source_names: Vec<String>,
    ) -> Result<Session, AgetError> {
        let name = name.into();
        validate_compose_target(&self.session_store, &name, &source_names)?;
        let source_sessions = source_names
            .iter()
            .map(|source| self.session_store.load(source).map_err(io_aget_error))
            .collect::<Result<Vec<_>, _>>()?;
        let session = compose_session(&name, &source_sessions)?;
        self.session_store.save(&session).map_err(io_aget_error)?;
        Ok(session)
    }

    pub fn start_login_session(
        &self,
        name: impl Into<String>,
        profile: Option<String>,
        url: impl Into<String>,
    ) -> Result<LoginStartResult, AgetError> {
        self.browser_backend.start_login(LoginStartOptions {
            name: name.into(),
            profile,
            url: url.into(),
            tmp_dir: self.tmp_dir(),
        })
    }

    pub fn finish_login_session(&self, name: impl Into<String>) -> Result<Session, AgetError> {
        let result = self.browser_backend.finish_login(LoginFinishOptions {
            name: name.into(),
            tmp_dir: self.tmp_dir(),
        })?;
        let session = match self.session_store.load(&result.session.name) {
            Ok(existing) => merge_login_session(existing, result.session),
            Err(error) if error.kind() == io::ErrorKind::NotFound => result.session,
            Err(error) => return Err(io_aget_error(error)),
        };
        self.session_store.save(&session).map_err(io_aget_error)?;
        complete_login_session(LoginCompleteOptions {
            pending: result.pending,
            tmp_dir: self.tmp_dir(),
        })?;
        Ok(session)
    }

    pub fn cancel_login_session(
        &self,
        name: impl Into<String>,
    ) -> Result<LoginCancelResult, AgetError> {
        self.browser_backend.cancel_login(LoginCancelOptions {
            name: name.into(),
            tmp_dir: self.tmp_dir(),
        })
    }
}

fn validate_compose_target(
    store: &impl SessionStoreBackend,
    target: &str,
    sources: &[String],
) -> Result<(), AgetError> {
    if sources.iter().any(|source| source == target) {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("compose target '{target}' must not match a source session"),
        });
    }
    if store.exists(target).map_err(io_aget_error)? {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("session '{target}' already exists"),
        });
    }
    Ok(())
}
