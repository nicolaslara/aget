use crate::browser_cdp::{render_attached_page, BrowserAttachedPageRenderRequest};
use crate::error::AgetError;
use crate::extraction::{BrowserFallbackRequest, BrowserFallbackResult};
use crate::session::{
    cancel_owned_login_session, finish_owned_login_session, import_owned_chrome_session,
    start_owned_login_session, ChromeImportOptions, LoginCancelOptions, LoginCancelResult,
    LoginFinishOptions, LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
};

use std::time::Duration;

#[derive(Debug, Clone)]
// Current-tab wiring intentionally stops at the engine seam until public
// consent and endpoint-discovery UX are chosen.
#[allow(dead_code)]
pub(crate) struct AttachedPageRequest<'a> {
    pub(crate) ws_url: &'a str,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_for_images: bool,
    pub(crate) flatten_shadow_dom: bool,
    pub(crate) settle_delay: Duration,
    pub(crate) page_timeout: Duration,
    pub(crate) wait_for_timeout: Option<Duration>,
    pub(crate) timeout: Duration,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AttachedPageResult {
    pub(crate) final_url: String,
    pub(crate) html: String,
    pub(crate) warnings: Vec<String>,
}

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

    #[allow(dead_code)]
    pub(crate) fn render_attached_page(
        &self,
        request: AttachedPageRequest<'_>,
    ) -> Result<AttachedPageResult, AgetError> {
        let rendered = render_attached_page(BrowserAttachedPageRenderRequest {
            ws_url: request.ws_url,
            wait_for_selector: request.wait_for_selector,
            wait_for_images: request.wait_for_images,
            flatten_shadow_dom: request.flatten_shadow_dom,
            settle_delay: request.settle_delay,
            page_timeout: request.page_timeout,
            wait_for_timeout: request.wait_for_timeout,
            timeout: request.timeout,
        })?;
        Ok(AttachedPageResult {
            final_url: rendered.final_url,
            html: rendered.html,
            warnings: rendered.warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::Duration;

    use serde_json::{json, Value};
    use tungstenite::{Message, WebSocket};

    use super::*;
    use crate::session::login::PendingLogin;

    fn read_cdp_request(websocket: &mut WebSocket<TcpStream>) -> Value {
        let Message::Text(text) = websocket.read().unwrap() else {
            panic!("expected text CDP command");
        };
        serde_json::from_str(text.as_ref()).unwrap()
    }

    fn reply_ok(websocket: &mut WebSocket<TcpStream>, request: &Value, result: Value) {
        let id = request
            .get("id")
            .and_then(serde_json::Value::as_u64)
            .unwrap();
        websocket
            .send(Message::Text(
                json!({ "id": id, "result": result }).to_string().into(),
            ))
            .unwrap();
    }

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

    #[test]
    fn renders_attached_page_without_aget_facade_or_command_backend() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut websocket = tungstenite::accept(stream).unwrap();

            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Target.setDiscoverTargets");
            reply_ok(&mut websocket, &request, json!({}));

            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Target.getTargets");
            reply_ok(
                &mut websocket,
                &request,
                json!({
                    "targetInfos": [
                        {
                            "targetId": "page-1",
                            "type": "page",
                            "url": "https://example.com/current",
                            "title": "Current"
                        }
                    ]
                }),
            );

            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Target.attachToTarget");
            assert_eq!(request["params"]["targetId"], "page-1");
            reply_ok(
                &mut websocket,
                &request,
                json!({ "sessionId": "session-1" }),
            );

            for expected_method in [
                "Page.enable",
                "Runtime.enable",
                "Runtime.runIfWaitingForDebugger",
                "Network.enable",
            ] {
                let request = read_cdp_request(&mut websocket);
                assert_eq!(request["method"], expected_method);
                assert_eq!(request["sessionId"], "session-1");
                reply_ok(&mut websocket, &request, json!({}));
            }

            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Target.setAutoAttach");
            assert_eq!(request["sessionId"], "session-1");
            reply_ok(&mut websocket, &request, json!({}));

            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Runtime.evaluate");
            assert_eq!(request["sessionId"], "session-1");
            assert!(request["params"]["expression"]
                .as_str()
                .unwrap()
                .contains("querySelectorAll"));
            reply_ok(
                &mut websocket,
                &request,
                json!({ "result": { "type": "undefined" } }),
            );

            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Runtime.evaluate");
            assert_eq!(request["sessionId"], "session-1");
            assert_eq!(request["params"]["expression"], "location.href");
            reply_ok(
                &mut websocket,
                &request,
                json!({ "result": { "type": "string", "value": "https://example.com/current" } }),
            );

            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Runtime.evaluate");
            assert_eq!(request["sessionId"], "session-1");
            assert_eq!(
                request["params"]["expression"],
                "document.documentElement.outerHTML || ''"
            );
            reply_ok(
                &mut websocket,
                &request,
                json!({ "result": { "type": "string", "value": "<html><body>Current engine page</body></html>" } }),
            );

            let _ = websocket.close(None);
        });

        let rendered = AgetBrowser::default()
            .render_attached_page(AttachedPageRequest {
                ws_url: &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
                wait_for_selector: None,
                wait_for_images: false,
                flatten_shadow_dom: false,
                settle_delay: Duration::ZERO,
                page_timeout: Duration::from_secs(2),
                wait_for_timeout: None,
                timeout: Duration::from_secs(2),
            })
            .unwrap();

        assert_eq!(rendered.final_url, "https://example.com/current");
        assert_eq!(
            rendered.html,
            "<html><body>Current engine page</body></html>"
        );
        assert!(rendered.warnings.is_empty());
        handle.join().unwrap();
    }
}
