use crate::error::AgetError;
use crate::extraction::{
    run_owned_extractor_backend, ExtractorBackendResult, ExtractorRequest, OWNED_EXTRACTOR,
};

/// Local extraction engine for URL-to-content behavior.
#[derive(Clone, Debug, Default)]
pub struct AgetExtractor;

impl AgetExtractor {
    pub(crate) fn name(&self) -> &'static str {
        OWNED_EXTRACTOR
    }

    pub(crate) fn extract(
        &self,
        request: ExtractorRequest<'_>,
    ) -> Result<ExtractorBackendResult, AgetError> {
        run_owned_extractor_backend(request)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    use super::*;
    use crate::cli::{ExtractorOption, OutputFormat};
    use crate::extraction::GetOptions;
    use crate::session::PlaywrightState;

    fn serve_once(body: &'static str) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/page", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        (url, server)
    }

    #[test]
    fn extracts_static_page_without_aget_facade() {
        let (url, server) = serve_once(
            r#"<html><body><nav>Skip</nav><main><h1>Engine</h1><p>Direct extraction.</p></main></body></html>"#,
        );

        let temp = tempfile::tempdir().unwrap();
        let state = PlaywrightState {
            cookies: Vec::new(),
            origins: Vec::new(),
        };
        let state_path = temp.path().join("state.json");
        let content_path = temp.path().join("content.md");
        let metadata_path = temp.path().join("metadata.json");
        let options = GetOptions {
            url: url.clone(),
            sessions: Vec::new(),
            output: None,
            home: None,
            timeout: Some(Duration::from_secs(5)),
            content_format: OutputFormat::Markdown,
            selector: Some("main".to_string()),
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            backend_options: Vec::new(),
        };

        let result = AgetExtractor::default()
            .extract(ExtractorRequest {
                url: &url,
                state: &state,
                state_path: &state_path,
                content_path: &content_path,
                metadata_path: &metadata_path,
                options: &options,
                timeout: Duration::from_secs(5),
            })
            .unwrap();

        server.join().unwrap();
        assert_eq!(AgetExtractor::default().name(), "aget-owned-extractor");
        assert_eq!(result.final_url.as_deref(), Some(url.as_str()));
        assert_eq!(
            result.content.as_deref(),
            Some("# Engine\n\nDirect extraction.")
        );
        assert_eq!(
            std::fs::read_to_string(content_path).unwrap(),
            "# Engine\n\nDirect extraction.\n"
        );
        assert!(std::fs::read_to_string(metadata_path)
            .unwrap()
            .contains("\"ok\": true"));
    }

    #[test]
    fn applies_selector_exclusion_and_target_options_without_aget_facade() {
        let (url, server) = serve_once(
            r#"<html><body><main><h1>Feature</h1><p>Keep <a href="/docs">linked text</a>.</p><p class="ad">Remove me.</p><aside>Ignore chrome.</aside></main></body></html>"#,
        );

        let temp = tempfile::tempdir().unwrap();
        let state = PlaywrightState {
            cookies: Vec::new(),
            origins: Vec::new(),
        };
        let state_path = temp.path().join("state.json");
        let content_path = temp.path().join("content.md");
        let metadata_path = temp.path().join("metadata.json");
        let options = GetOptions {
            url: url.clone(),
            sessions: Vec::new(),
            output: None,
            home: None,
            timeout: Some(Duration::from_secs(5)),
            content_format: OutputFormat::Markdown,
            selector: Some("main".to_string()),
            exclude_selector: Some("p.ad".to_string()),
            wait_for_selector: None,
            max_chars: None,
            backend_options: vec![ExtractorOption {
                key: "aget.target_elements".to_string(),
                value: "h1,p".to_string(),
            }],
        };

        let result = AgetExtractor::default()
            .extract(ExtractorRequest {
                url: &url,
                state: &state,
                state_path: &state_path,
                content_path: &content_path,
                metadata_path: &metadata_path,
                options: &options,
                timeout: Duration::from_secs(5),
            })
            .unwrap();

        server.join().unwrap();
        let content = result.content.unwrap();
        assert!(content.contains("Feature"));
        assert!(content.contains("Keep"));
        assert!(content.contains("linked text"));
        assert!(content.contains("[linked text]"));
        assert!(!content.contains("Remove me."));
        assert!(!content.contains("Ignore chrome."));
    }
}
