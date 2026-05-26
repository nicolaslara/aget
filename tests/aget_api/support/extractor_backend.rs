use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use aget::extraction::{ExtractorBackend, ExtractorBackendResult, ExtractorRequest};
use aget::{AgetError, ErrorCode};

#[derive(Clone)]
pub(crate) struct InspectingExtractor;

impl ExtractorBackend for InspectingExtractor {
    fn name(&self) -> &'static str {
        "inspect-extractor"
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        let state: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(request.state_path).unwrap()).unwrap();
        assert_eq!(state["cookies"][0]["value"], "memory-secret");
        assert_eq!(request.state.cookies[0].value, "memory-secret");
        assert_eq!(request.options.selector.as_deref(), Some("main"));
        assert!(request.content_path.starts_with(
            request
                .state_path
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("runs")
        ));
        assert!(request
            .metadata_path
            .starts_with(request.content_path.parent().unwrap()));

        Ok(ExtractorBackendResult {
            ok: true,
            final_url: Some(format!("{}/final", request.url)),
            content: Some("typed extractor content".to_string()),
            page_metadata: Default::default(),
            warnings: vec!["custom extractor".to_string()],
            source_bytes: Some("typed extractor content".len()),
            error: None,
            screenshot_png: None,
        })
    }
}

#[derive(Clone)]
pub(crate) struct FailingExtractor;

impl ExtractorBackend for FailingExtractor {
    fn name(&self) -> &'static str {
        "failing-extractor"
    }

    fn extract(&self, _request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: "primary extractor failed".to_string(),
        })
    }
}

#[derive(Clone)]
pub(crate) struct SecretLeakingExtractor;

impl ExtractorBackend for SecretLeakingExtractor {
    fn name(&self) -> &'static str {
        "secret-leaking-extractor"
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!(
                "primary extractor leaked {}",
                request.state.cookies[0].value
            ),
        })
    }
}

#[derive(Clone)]
pub(crate) struct AuthorizationExtractor {
    baseline_content: String,
    verification_content: String,
    calls: Rc<RefCell<Vec<Vec<String>>>>,
}

impl AuthorizationExtractor {
    pub(crate) fn new(baseline_content: &str, verification_content: &str) -> Self {
        Self {
            baseline_content: baseline_content.to_string(),
            verification_content: verification_content.to_string(),
            calls: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(crate) fn call_count(&self) -> usize {
        self.calls.borrow().len()
    }
}

impl ExtractorBackend for AuthorizationExtractor {
    fn name(&self) -> &'static str {
        "authorization-extractor"
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        self.calls
            .borrow_mut()
            .push(request.options.sessions.clone());
        let content = if request.options.sessions.is_empty() {
            &self.baseline_content
        } else {
            assert_eq!(request.state.cookies[0].value, "chrome-secret");
            &self.verification_content
        };

        Ok(ExtractorBackendResult {
            ok: true,
            final_url: Some(request.url.to_string()),
            content: Some(content.clone()),
            page_metadata: Default::default(),
            warnings: Vec::new(),
            source_bytes: Some(content.len()),
            error: None,
            screenshot_png: None,
        })
    }
}
