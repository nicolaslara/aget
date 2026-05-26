use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::chrome_process::ChromeProcess;
use super::client::{CdpClient, PageSession, ScreenshotClip};
use super::{discover_cdp_ws_url, CHROME_SHUTDOWN_WAIT};
use crate::cli::{CachePolicy, OutputFormat};
use crate::error::{AgetError, ErrorCode, ErrorResponse};
use crate::extraction::{extract_owned_rendered_html, DebugCaptureOptions, GetOptions};
use crate::interact::{
    ActionDefinition, ActionPlan, BrowserActionArtifact, BrowserActionExecution,
    BrowserActionResult, BrowserActionStatus, CaptureAction, ClickAction, ExtractAction,
    SelectAction, SelectorMatch, SubmitAction, TypeAction, WaitAction, WaitLoadState,
};

const ACTION_BOUNDARY_SETTLE: Duration = Duration::from_millis(1_000);

pub(crate) enum BrowserActionSource<'a> {
    Url { url: &'a str, tmp_dir: &'a Path },
    CurrentTab { port: u16 },
}

pub(crate) struct BrowserActionPlanRequest<'a> {
    pub(crate) source: BrowserActionSource<'a>,
    pub(crate) plan: &'a ActionPlan,
    pub(crate) timeout: Duration,
    pub(crate) default_action_timeout: Duration,
    pub(crate) page_timeout: Duration,
    pub(crate) artifact_dir: &'a Path,
    pub(crate) sensitive: bool,
}

pub(crate) fn execute_browser_action_plan(
    request: BrowserActionPlanRequest<'_>,
) -> Result<BrowserActionExecution, AgetError> {
    match request.source {
        BrowserActionSource::CurrentTab { port } => execute_current_tab(port, request),
        BrowserActionSource::Url { url, tmp_dir } => execute_url(url, tmp_dir, request),
    }
}

fn execute_current_tab(
    port: u16,
    request: BrowserActionPlanRequest<'_>,
) -> Result<BrowserActionExecution, AgetError> {
    let ws_url = discover_cdp_ws_url(port, request.timeout)?;
    let mut client = CdpClient::connect(&ws_url, request.timeout)?;
    let page = client
        .attach_existing_page(request.page_timeout)?
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: "owned browser interact CDP attach found no page targets".to_string(),
        })?;
    client.enable_page_domains(&page.session_id, request.page_timeout)?;
    let warnings = install_interact_event_guards(&mut client, &page, request.page_timeout);
    execute_attached_actions(&mut client, &page, request, warnings)
}

fn execute_url(
    url: &str,
    tmp_dir: &Path,
    request: BrowserActionPlanRequest<'_>,
) -> Result<BrowserActionExecution, AgetError> {
    let mut chrome =
        ChromeProcess::launch_temp(tmp_dir, request.timeout, "owned interact browser")?;
    let mut client = CdpClient::connect(&chrome.ws_url, request.timeout)?;
    let page = client.create_page(request.page_timeout)?;
    client.enable_page_domains(&page.session_id, request.page_timeout)?;
    client.navigate_and_wait(
        &page.session_id,
        url,
        super::PageWaitUntil::DomContentLoaded,
        request.page_timeout,
    )?;
    let warnings = install_interact_event_guards(&mut client, &page, request.page_timeout);
    let execution = execute_attached_actions(&mut client, &page, request, warnings);
    let _ = client.close_page(&page, Duration::from_secs(1));
    let _ = client.close_browser(Duration::from_secs(1));
    chrome.wait_or_kill(CHROME_SHUTDOWN_WAIT);
    execution
}

fn execute_attached_actions(
    client: &mut CdpClient,
    page: &PageSession,
    request: BrowserActionPlanRequest<'_>,
    warnings: Vec<String>,
) -> Result<BrowserActionExecution, AgetError> {
    let initial_url =
        client.evaluate_string(&page.session_id, "location.href", request.page_timeout)?;
    let mut final_url = initial_url.clone();
    let mut action_results = Vec::new();
    let mut error = None;

    for (index, action) in request.plan.actions.iter().enumerate() {
        let timeout = action_timeout(action, &request);
        let started = Instant::now();
        match execute_action(
            client,
            page,
            index,
            action,
            timeout,
            request.page_timeout,
            &initial_url,
            request.artifact_dir,
            request.sensitive,
        ) {
            Ok(artifacts) => {
                if let Err(body) = fail_on_interact_hazard_events(
                    &client.take_hazard_events(),
                    &initial_url,
                    &page.target_id,
                ) {
                    error = Some(body.clone());
                    action_results.push(BrowserActionResult {
                        index,
                        action_type: action.kind(),
                        status: BrowserActionStatus::Failed,
                        elapsed_ms: started.elapsed().as_millis(),
                        artifacts,
                        error: Some(body),
                    });
                    break;
                }
                match client.evaluate_string(
                    &page.session_id,
                    "location.href",
                    request.page_timeout,
                ) {
                    Ok(url) => {
                        if let Err(body) = fail_on_interact_hazard_events(
                            &client.take_hazard_events(),
                            &initial_url,
                            &page.target_id,
                        ) {
                            error = Some(body.clone());
                            action_results.push(BrowserActionResult {
                                index,
                                action_type: action.kind(),
                                status: BrowserActionStatus::Failed,
                                elapsed_ms: started.elapsed().as_millis(),
                                artifacts,
                                error: Some(body),
                            });
                            break;
                        }
                        final_url = url;
                        action_results.push(BrowserActionResult {
                            index,
                            action_type: action.kind(),
                            status: BrowserActionStatus::Ok,
                            elapsed_ms: started.elapsed().as_millis(),
                            artifacts,
                            error: None,
                        });
                    }
                    Err(cdp_error) => {
                        let body = error_body_from_aget(cdp_error);
                        error = Some(body.clone());
                        action_results.push(BrowserActionResult {
                            index,
                            action_type: action.kind(),
                            status: BrowserActionStatus::Failed,
                            elapsed_ms: started.elapsed().as_millis(),
                            artifacts,
                            error: Some(body),
                        });
                        break;
                    }
                }
            }
            Err(body) => {
                error = Some(body.clone());
                action_results.push(BrowserActionResult {
                    index,
                    action_type: action.kind(),
                    status: BrowserActionStatus::Failed,
                    elapsed_ms: started.elapsed().as_millis(),
                    artifacts: Vec::new(),
                    error: Some(body),
                });
                break;
            }
        }
    }

    Ok(BrowserActionExecution {
        final_url: Some(final_url),
        action_results,
        warnings,
        error,
    })
}

fn install_interact_event_guards(
    client: &mut CdpClient,
    page: &PageSession,
    timeout: Duration,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let guard_timeout = timeout.min(Duration::from_secs(1));
    if let Err(error) = client.send(
        "Target.setDiscoverTargets",
        Some(json!({ "discover": true })),
        None,
        guard_timeout,
    ) {
        warnings.push(format!("interact target event guard unavailable: {error}"));
    }
    if let Err(error) = client.send(
        "Browser.setDownloadBehavior",
        Some(json!({
            "behavior": "deny",
            "eventsEnabled": true,
        })),
        None,
        guard_timeout,
    ) {
        warnings.push(format!(
            "interact download event guard unavailable: {error}"
        ));
    }
    if let Err(error) = client.send(
        "Page.setInterceptFileChooserDialog",
        Some(json!({ "enabled": true })),
        session_param(page),
        guard_timeout,
    ) {
        warnings.push(format!("interact file chooser guard unavailable: {error}"));
    }
    let _ = client.take_hazard_events();
    warnings
}

fn execute_action(
    client: &mut CdpClient,
    page: &PageSession,
    action_index: usize,
    action: &ActionDefinition,
    timeout: Duration,
    page_timeout: Duration,
    initial_url: &str,
    artifact_dir: &Path,
    sensitive: bool,
) -> Result<Vec<BrowserActionArtifact>, crate::error::ErrorBody> {
    match action {
        ActionDefinition::Wait(action) => {
            execute_wait(client, page, action, timeout)?;
            Ok(Vec::new())
        }
        ActionDefinition::Click(action) => {
            run_action_script(client, page, &click_script(action), timeout)?;
            enforce_interact_boundary(client, page, initial_url, page_timeout)?;
            Ok(Vec::new())
        }
        ActionDefinition::Type(action) => {
            run_action_script(client, page, &type_script(action), timeout)?;
            enforce_interact_boundary(client, page, initial_url, page_timeout)?;
            Ok(Vec::new())
        }
        ActionDefinition::Select(action) => {
            run_action_script(client, page, &select_script(action), timeout)?;
            enforce_interact_boundary(client, page, initial_url, page_timeout)?;
            Ok(Vec::new())
        }
        ActionDefinition::Submit(action) => {
            run_action_script(client, page, &submit_script(action), timeout)?;
            enforce_interact_boundary(client, page, initial_url, page_timeout)?;
            Ok(Vec::new())
        }
        ActionDefinition::Capture(action) => {
            execute_capture(client, page, action, page_timeout, artifact_dir, sensitive)
        }
        ActionDefinition::Extract(action) => execute_extract(
            client,
            page,
            action_index,
            action,
            page_timeout,
            artifact_dir,
            sensitive,
        ),
    }
}

fn execute_wait(
    client: &mut CdpClient,
    page: &PageSession,
    action: &WaitAction,
    timeout: Duration,
) -> Result<(), crate::error::ErrorBody> {
    if let Some(selector) = &action.selector {
        return client
            .wait_for_selector(&page.session_id, selector, timeout)
            .map_err(error_body_from_aget);
    }
    if let Some(duration_ms) = action.duration_ms {
        thread::sleep(Duration::from_millis(duration_ms));
        return Ok(());
    }
    if let Some(load_state) = action.load_state {
        return wait_for_load_state(client, &page.session_id, load_state, timeout);
    }
    Ok(())
}

fn execute_capture(
    client: &mut CdpClient,
    page: &PageSession,
    action: &CaptureAction,
    page_timeout: Duration,
    artifact_dir: &Path,
    source_sensitive: bool,
) -> Result<Vec<BrowserActionArtifact>, crate::error::ErrorBody> {
    let capture_dir = artifact_dir.join("captures");
    let sensitive = source_sensitive || action.sensitive;
    let mut pending = Vec::new();
    if action.html {
        let html = capture_html_for_action(
            client,
            &page.session_id,
            action.selector.as_deref(),
            action.match_mode,
            page_timeout,
        )?;
        let path = capture_dir.join(format!("{}.html", action.name));
        let artifact = BrowserActionArtifact {
            name: action.name.clone(),
            kind: "capture-html",
            path,
            media_type: "text/html",
            sensitive,
        };
        pending.push((artifact, html.into_bytes()));
    }
    if action.screenshot {
        let clip = if action.selector.is_some() {
            Some(capture_clip_for_action(
                client,
                &page.session_id,
                action.selector.as_deref(),
                action.match_mode,
                page_timeout,
            )?)
        } else {
            None
        };
        let bytes = client
            .capture_screenshot_png_with_clip(&page.session_id, clip, page_timeout)
            .map_err(error_body_from_aget)?;
        let path = capture_dir.join(format!("{}.png", action.name));
        let artifact = BrowserActionArtifact {
            name: action.name.clone(),
            kind: "capture-screenshot",
            path,
            media_type: "image/png",
            sensitive,
        };
        pending.push((artifact, bytes));
    }

    let mut artifacts = Vec::new();
    for (artifact, bytes) in pending {
        write_private_bytes(&artifact.path, &bytes).map_err(error_body_from_io)?;
        artifacts.push(artifact);
    }
    Ok(artifacts)
}

fn execute_extract(
    client: &mut CdpClient,
    page: &PageSession,
    action_index: usize,
    action: &ExtractAction,
    page_timeout: Duration,
    artifact_dir: &Path,
    source_sensitive: bool,
) -> Result<Vec<BrowserActionArtifact>, crate::error::ErrorBody> {
    let final_url = client
        .evaluate_string(&page.session_id, "location.href", page_timeout)
        .map_err(error_body_from_aget)?;
    let html = extract_html_for_action(client, page, action, page_timeout)?;
    let options = extraction_options(&final_url, action);
    let extraction =
        extract_owned_rendered_html(final_url, html, &options).map_err(error_body_from_aget)?;
    let name = safe_artifact_name(action.name.as_deref(), &format!("extract-{action_index}"));
    let file_stem = format!("{}-{name}", action_index);
    let path = artifact_dir.join("extracts").join(format!(
        "{file_stem}.{}",
        extension_for_format(action.content_format)
    ));
    write_private_bytes(&path, extraction.content.as_bytes()).map_err(error_body_from_io)?;
    Ok(vec![BrowserActionArtifact {
        name,
        kind: "extract",
        path,
        media_type: media_type_for_format(action.content_format),
        sensitive: source_sensitive || action.sensitive,
    }])
}

fn extraction_options(final_url: &str, action: &ExtractAction) -> GetOptions {
    GetOptions {
        url: final_url.to_string(),
        sessions: Vec::new(),
        output: None,
        home: None,
        timeout: None,
        content_format: action.content_format,
        selector: None,
        exclude_selector: None,
        wait_for_selector: None,
        max_chars: None,
        cache_policy: CachePolicy::Off,
        cache_ttl: Duration::from_secs(0),
        debug: DebugCaptureOptions::default(),
        backend_options: Vec::new(),
    }
}

fn extract_html_for_action(
    client: &mut CdpClient,
    page: &PageSession,
    action: &ExtractAction,
    page_timeout: Duration,
) -> Result<String, crate::error::ErrorBody> {
    if let Some(selector) = action.selector.as_deref() {
        return capture_html_for_action(
            client,
            &page.session_id,
            Some(selector),
            action.match_mode,
            page_timeout,
        );
    }
    client
        .evaluate_string(
            &page.session_id,
            "document.documentElement.outerHTML || ''",
            page_timeout,
        )
        .map_err(error_body_from_aget)
}

fn capture_html_for_action(
    client: &mut CdpClient,
    session_id: &str,
    selector: Option<&str>,
    match_mode: Option<SelectorMatch>,
    timeout: Duration,
) -> Result<String, crate::error::ErrorBody> {
    let Some(selector) = selector else {
        return client
            .evaluate_string(
                session_id,
                "document.documentElement.outerHTML || ''",
                timeout,
            )
            .map_err(error_body_from_aget);
    };
    let value = evaluate_action_value(
        client,
        session_id,
        &capture_html_script(selector, match_mode),
        timeout,
    )?;
    value
        .get("html")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| {
            error_body(
                ErrorCode::ExtractionFailed,
                "capture html action returned no html",
            )
        })
}

fn capture_clip_for_action(
    client: &mut CdpClient,
    session_id: &str,
    selector: Option<&str>,
    match_mode: Option<SelectorMatch>,
    timeout: Duration,
) -> Result<ScreenshotClip, crate::error::ErrorBody> {
    let Some(selector) = selector else {
        return Err(error_body(
            ErrorCode::ExtractionFailed,
            "capture screenshot clip requires a selector",
        ));
    };
    let value = evaluate_action_value(
        client,
        session_id,
        &capture_html_script(selector, match_mode),
        timeout,
    )?;
    screenshot_clip_from_value(&value)
}

fn screenshot_clip_from_value(value: &Value) -> Result<ScreenshotClip, crate::error::ErrorBody> {
    let clip = value.get("clip").ok_or_else(|| {
        error_body(
            ErrorCode::ExtractionFailed,
            "capture screenshot action returned no clip",
        )
    })?;
    let x = clip.get("x").and_then(Value::as_f64).unwrap_or(0.0);
    let y = clip.get("y").and_then(Value::as_f64).unwrap_or(0.0);
    let width = clip.get("width").and_then(Value::as_f64).unwrap_or(0.0);
    let height = clip.get("height").and_then(Value::as_f64).unwrap_or(0.0);
    let scale = clip.get("scale").and_then(Value::as_f64).unwrap_or(1.0);
    if !x.is_finite()
        || !y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || !scale.is_finite()
        || width <= 0.0
        || height <= 0.0
        || scale <= 0.0
    {
        return Err(error_body(
            ErrorCode::ExtractionFailed,
            "capture screenshot selector has no visible box",
        ));
    }
    Ok(ScreenshotClip {
        x,
        y,
        width,
        height,
        scale,
    })
}

fn evaluate_action_value(
    client: &mut CdpClient,
    session_id: &str,
    expression: &str,
    timeout: Duration,
) -> Result<Value, crate::error::ErrorBody> {
    let raw = client
        .evaluate_string_await(session_id, expression, timeout)
        .map_err(error_body_from_aget)?;
    let value: Value = serde_json::from_str(&raw).map_err(|error| {
        error_body(
            ErrorCode::ExtractionFailed,
            format!("interact action returned invalid JSON: {error}"),
        )
    })?;
    if value.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Ok(value);
    }
    let code = value
        .get("code")
        .and_then(Value::as_str)
        .map(action_error_code)
        .unwrap_or(ErrorCode::ExtractionFailed);
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("interact action failed");
    Err(error_body(code, message))
}

fn wait_for_load_state(
    client: &mut CdpClient,
    session_id: &str,
    load_state: WaitLoadState,
    timeout: Duration,
) -> Result<(), crate::error::ErrorBody> {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(error_body(
                ErrorCode::ActionTimeout,
                format!("timed out waiting for {load_state:?}"),
            ));
        }
        let state = client
            .evaluate_string(session_id, "document.readyState", remaining)
            .map_err(error_body_from_aget)?;
        let ready = match load_state {
            WaitLoadState::DomContentLoaded => state == "interactive" || state == "complete",
            WaitLoadState::Load | WaitLoadState::NetworkIdle => state == "complete",
        };
        if ready {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn run_action_script(
    client: &mut CdpClient,
    page: &PageSession,
    expression: &str,
    timeout: Duration,
) -> Result<(), crate::error::ErrorBody> {
    let raw = client
        .evaluate_string_await(&page.session_id, expression, timeout)
        .map_err(error_body_from_aget)?;
    let value: Value = serde_json::from_str(&raw).map_err(|error| {
        error_body(
            ErrorCode::ExtractionFailed,
            format!("interact action returned invalid JSON: {error}"),
        )
    })?;
    if value.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Ok(());
    }
    let code = value
        .get("code")
        .and_then(Value::as_str)
        .map(action_error_code)
        .unwrap_or(ErrorCode::ExtractionFailed);
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("interact action failed");
    Err(error_body(code, message))
}

fn enforce_interact_boundary(
    client: &mut CdpClient,
    page: &PageSession,
    initial_url: &str,
    timeout: Duration,
) -> Result<(), crate::error::ErrorBody> {
    let deadline = Instant::now() + ACTION_BOUNDARY_SETTLE.min(timeout);
    loop {
        fail_on_interact_hazard_events(&client.take_hazard_events(), initial_url, &page.target_id)?;
        let final_url = client
            .evaluate_string(
                &page.session_id,
                "location.href",
                timeout.min(Duration::from_secs(1)),
            )
            .map_err(error_body_from_aget)?;
        fail_on_interact_hazard_events(&client.take_hazard_events(), initial_url, &page.target_id)?;
        if !origins_match(initial_url, &final_url) {
            return Err(error_body(
                ErrorCode::NavigationBlocked,
                format!(
                    "interact action navigated across origin from {initial_url} to {final_url}"
                ),
            ));
        }
        if Instant::now() >= deadline {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn fail_on_interact_hazard_events(
    events: &[Value],
    initial_url: &str,
    current_target_id: &str,
) -> Result<(), crate::error::ErrorBody> {
    if let Some(body) = interact_hazard_error(events, initial_url, current_target_id) {
        return Err(body);
    }
    Ok(())
}

fn interact_hazard_error(
    events: &[Value],
    initial_url: &str,
    current_target_id: &str,
) -> Option<crate::error::ErrorBody> {
    for event in events {
        let method = event
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let params = event.get("params").unwrap_or(&Value::Null);
        match method {
            "Browser.downloadWillBegin" | "Browser.downloadProgress" => {
                return Some(error_body(
                    ErrorCode::RequiresConfirmation,
                    "interact blocked browser download event",
                ));
            }
            "Page.fileChooserOpened" => {
                return Some(error_body(
                    ErrorCode::RequiresConfirmation,
                    "interact blocked file chooser event",
                ));
            }
            "Page.javascriptDialogOpening" => {
                return Some(error_body(
                    ErrorCode::RequiresConfirmation,
                    "interact blocked browser dialog event",
                ));
            }
            "Page.windowOpen" => {
                return Some(error_body(
                    ErrorCode::RequiresConfirmation,
                    "interact blocked popup/new window event",
                ));
            }
            "Page.frameScheduledNavigation" => {
                let url = params
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if !url.is_empty() && !origins_match(initial_url, url) {
                    return Some(error_body(
                        ErrorCode::NavigationBlocked,
                        format!("interact blocked delayed cross-origin navigation to {url}"),
                    ));
                }
            }
            "Target.targetCreated" | "Target.attachedToTarget" => {
                let target_info = params.get("targetInfo").unwrap_or(params);
                let target_type = target_info
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let target_id = target_info
                    .get("targetId")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if target_type == "page" && !target_id.is_empty() && target_id != current_target_id
                {
                    return Some(error_body(
                        ErrorCode::RequiresConfirmation,
                        "interact blocked popup/new page target event",
                    ));
                }
            }
            _ => {}
        }
    }
    None
}

fn session_param(page: &PageSession) -> Option<&str> {
    if page.session_id.is_empty() {
        None
    } else {
        Some(page.session_id.as_str())
    }
}

fn origins_match(left: &str, right: &str) -> bool {
    let Ok(left) = url::Url::parse(left) else {
        return true;
    };
    let Ok(right) = url::Url::parse(right) else {
        return true;
    };
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn click_script(action: &ClickAction) -> String {
    with_prelude(format!(
        r#"(() => {{
          const result = agetResolveUnique({}, "click");
          if (!result.ok) return JSON.stringify(result);
          const element = result.element;
          const unsafeResult = agetUnsafeElement(element);
          if (unsafeResult) return JSON.stringify(unsafeResult);
          const unsafeForm = agetUnsafeContainingForm(element);
          if (unsafeForm) return JSON.stringify(unsafeForm);
          const blocked = agetBlockedNavigation(element);
          if (blocked) return JSON.stringify(blocked);
          return agetWithBoundary(() => {{ element.click(); }});
        }})()"#,
        js_string(&action.selector)
    ))
}

fn type_script(action: &TypeAction) -> String {
    with_prelude(format!(
        r#"(() => {{
          const result = agetResolveUnique({}, "type");
          if (!result.ok) return JSON.stringify(result);
          const element = result.element;
          const unsafeResult = agetUnsafeElement(element);
          if (unsafeResult) return JSON.stringify(unsafeResult);
          return agetWithBoundary(() => {{
            element.focus && element.focus();
            if ("value" in element) {{
              element.value = {};
            }} else if (element.isContentEditable) {{
              element.textContent = {};
            }} else {{
              return {{ ok: false, code: "unsafe_action", message: "type target is not editable" }};
            }}
            element.dispatchEvent(new Event("input", {{ bubbles: true }}));
            element.dispatchEvent(new Event("change", {{ bubbles: true }}));
          }});
        }})()"#,
        js_string(&action.selector),
        js_string(&action.text),
        js_string(&action.text)
    ))
}

fn select_script(action: &SelectAction) -> String {
    let choice = if let Some(value) = &action.value {
        json!({"kind": "value", "value": value})
    } else if let Some(label) = &action.label {
        json!({"kind": "label", "value": label})
    } else {
        json!({"kind": "index", "value": action.index.unwrap_or_default()})
    };
    with_prelude(format!(
        r#"(() => {{
          const result = agetResolveUnique({}, "select");
          if (!result.ok) return JSON.stringify(result);
          const element = result.element;
          if (!(element instanceof HTMLSelectElement)) {{
            return JSON.stringify({{ ok: false, code: "unsafe_action", message: "select target is not a select element" }});
          }}
          const choice = {};
          return agetWithBoundary(() => {{
            let option = null;
            if (choice.kind === "value") {{
              option = Array.from(element.options).find((candidate) => candidate.value === choice.value);
            }} else if (choice.kind === "label") {{
              option = Array.from(element.options).find((candidate) => candidate.label === choice.value || candidate.text === choice.value);
            }} else {{
              option = element.options[choice.value];
            }}
            if (!option) {{
              return {{ ok: false, code: "selector_not_found", message: "select option was not found" }};
            }}
            element.value = option.value;
            element.dispatchEvent(new Event("input", {{ bubbles: true }}));
            element.dispatchEvent(new Event("change", {{ bubbles: true }}));
          }});
        }})()"#,
        js_string(&action.selector),
        choice
    ))
}

fn submit_script(action: &SubmitAction) -> String {
    with_prelude(format!(
        r#"(() => {{
          const result = agetResolveUnique({}, "submit");
          if (!result.ok) return JSON.stringify(result);
          const element = result.element;
          const unsafeResult = agetUnsafeElement(element);
          if (unsafeResult) return JSON.stringify(unsafeResult);
          const form = element instanceof HTMLFormElement ? element : (element.form || element.closest("form"));
          const unsafeForm = agetUnsafeForm(form);
          if (unsafeForm) return JSON.stringify(unsafeForm);
          const blocked = agetBlockedNavigation(form || element);
          if (blocked) return JSON.stringify(blocked);
          return agetWithBoundary(() => {{
            if (form && form.requestSubmit) {{
              if (element !== form && element.type === "submit") {{
                form.requestSubmit(element);
              }} else {{
                form.requestSubmit();
              }}
            }} else if (form) {{
              form.submit();
            }} else {{
              element.click();
            }}
          }});
        }})()"#,
        js_string(&action.selector)
    ))
}

fn capture_html_script(selector: &str, match_mode: Option<SelectorMatch>) -> String {
    let resolver = if matches!(match_mode, Some(SelectorMatch::First)) {
        format!(
            r#"
          const element = document.querySelector({});
          if (!element) return JSON.stringify({{ ok: false, code: "selector_not_found", message: "capture selector matched no elements" }});
        "#,
            js_string(selector)
        )
    } else {
        format!(
            r#"
          const result = agetResolveUnique({}, "capture");
          if (!result.ok) return JSON.stringify(result);
          const element = result.element;
        "#,
            js_string(selector)
        )
    };
    with_prelude(format!(
        r#"(() => {{
          {}
          const rect = element.getBoundingClientRect();
          return JSON.stringify({{
            ok: true,
            html: element.outerHTML || "",
            clip: {{
              x: Math.max(0, rect.left + window.scrollX),
              y: Math.max(0, rect.top + window.scrollY),
              width: rect.width,
              height: rect.height,
              scale: 1
            }}
          }});
        }})()"#,
        resolver
    ))
}

fn action_timeout(action: &ActionDefinition, request: &BrowserActionPlanRequest<'_>) -> Duration {
    let millis = match action {
        ActionDefinition::Wait(action) => action.timeout_ms,
        ActionDefinition::Click(action) => action.timeout_ms,
        ActionDefinition::Type(action) => action.timeout_ms,
        ActionDefinition::Select(action) => action.timeout_ms,
        ActionDefinition::Submit(action) => action.timeout_ms,
        ActionDefinition::Capture(action) => action.timeout_ms,
        ActionDefinition::Extract(action) => action.timeout_ms,
    }
    .or(request.plan.defaults.action_timeout_ms);
    millis
        .map(Duration::from_millis)
        .unwrap_or(request.default_action_timeout)
}

fn action_error_code(code: &str) -> ErrorCode {
    match code {
        "selector_not_found" => ErrorCode::SelectorNotFound,
        "selector_ambiguous" => ErrorCode::SelectorAmbiguous,
        "requires_confirmation" => ErrorCode::RequiresConfirmation,
        "unsafe_action" => ErrorCode::UnsafeAction,
        "navigation_blocked" => ErrorCode::NavigationBlocked,
        "action_not_supported" => ErrorCode::ActionNotSupported,
        _ => ErrorCode::ExtractionFailed,
    }
}

fn error_body_from_aget(error: AgetError) -> crate::error::ErrorBody {
    match error {
        AgetError::Stable { code, message } => {
            let code = if code == ErrorCode::Timeout {
                ErrorCode::ActionTimeout
            } else {
                code
            };
            error_body(code, message)
        }
    }
}

fn error_body_from_io(error: io::Error) -> crate::error::ErrorBody {
    error_body(
        ErrorCode::IoError,
        format!("interact artifact write failed: {error}"),
    )
}

fn error_body(code: ErrorCode, message: impl Into<String>) -> crate::error::ErrorBody {
    ErrorResponse::new(code, message).error
}

fn js_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serializes")
}

fn action_prelude() -> &'static str {
    r#"
function agetResolveUnique(selector, actionType) {
  const elements = Array.from(document.querySelectorAll(selector));
  if (elements.length === 0) {
    return { ok: false, code: "selector_not_found", message: `${actionType} selector matched no elements` };
  }
  if (elements.length > 1) {
    return { ok: false, code: "selector_ambiguous", message: `${actionType} selector matched ${elements.length} elements` };
  }
  return { ok: true, element: elements[0] };
}
function agetUnsafeElement(element) {
  if (!element) return { ok: false, code: "selector_not_found", message: "element was not found" };
  const input = element.closest ? element.closest("input,textarea,[contenteditable=true]") : element;
  const type = (input && input.getAttribute && (input.getAttribute("type") || "")).toLowerCase();
  const autocomplete = (input && input.getAttribute && (input.getAttribute("autocomplete") || "")).toLowerCase();
  if (type === "password" || autocomplete === "current-password" || autocomplete === "one-time-code") {
    return { ok: false, code: "unsafe_action", message: "interact blocked credential-equivalent input target" };
  }
  if (type === "file") {
    return { ok: false, code: "requires_confirmation", message: "interact blocked file chooser target" };
  }
  return null;
}
function agetUnsafeForm(form) {
  if (!form || !form.querySelector) return null;
  const credential = form.querySelector('input[type="password"], [autocomplete="current-password"], [autocomplete="one-time-code"]');
  if (credential) {
    return { ok: false, code: "unsafe_action", message: "interact blocked credential-equivalent form submission" };
  }
  const fileInput = form.querySelector('input[type="file"]');
  if (fileInput) {
    return { ok: false, code: "requires_confirmation", message: "interact blocked file chooser form submission" };
  }
  return null;
}
function agetUnsafeContainingForm(element) {
  const form = element && (element.form || (element.closest && element.closest("form")));
  return agetUnsafeForm(form);
}
function agetBlockedNavigation(element) {
  if (!element) return null;
  const link = element.closest && element.closest("a[href]");
  if (link && link.hasAttribute("download")) {
    return { ok: false, code: "requires_confirmation", message: "interact blocked download link" };
  }
  const url = (link && link.href) || (element.action || null);
  if (!url) return null;
  try {
    const next = new URL(url, location.href);
    if (next.origin !== location.origin) {
      return { ok: false, code: "navigation_blocked", message: `interact blocked cross-origin navigation to ${next.origin}` };
    }
  } catch (_error) {}
  return null;
}
async function agetWithBoundary(callback) {
  const oldOpen = window.open;
  const oldAlert = window.alert;
  const oldPrompt = window.prompt;
  const oldConfirm = window.confirm;
  const oldAnchorClick = HTMLAnchorElement.prototype.click;
  let boundaryError = null;
  const rememberBoundary = (result, marker) => {
    boundaryError = result;
    throw new Error(marker);
  };
  window.open = () => rememberBoundary({ ok: false, code: "requires_confirmation", message: "interact blocked popup/new window" }, "__AGET_POPUP_BLOCKED__");
  window.alert = () => rememberBoundary({ ok: false, code: "requires_confirmation", message: "interact blocked browser dialog" }, "__AGET_ALERT_BLOCKED__");
  window.prompt = () => rememberBoundary({ ok: false, code: "requires_confirmation", message: "interact blocked browser dialog" }, "__AGET_PROMPT_BLOCKED__");
  window.confirm = () => rememberBoundary({ ok: false, code: "requires_confirmation", message: "interact blocked browser dialog" }, "__AGET_CONFIRM_BLOCKED__");
  HTMLAnchorElement.prototype.click = function(...args) {
    const blocked = agetBlockedNavigation(this);
    if (blocked) return rememberBoundary(blocked, "__AGET_ANCHOR_CLICK_BLOCKED__");
    return oldAnchorClick.apply(this, args);
  };
  try {
    const result = callback();
    if (result && typeof result.then === "function") await result;
    await new Promise((resolve) => setTimeout(resolve, 1000));
    if (boundaryError) return JSON.stringify(boundaryError);
    if (result && result.ok === false) return JSON.stringify(result);
    return JSON.stringify({ ok: true });
  } catch (error) {
    if (boundaryError) return JSON.stringify(boundaryError);
    const message = String(error && error.message || error);
    if (message.includes("__AGET_POPUP_BLOCKED__")) {
      return JSON.stringify({ ok: false, code: "requires_confirmation", message: "interact blocked popup/new window" });
    }
    if (message.includes("__AGET_PROMPT_BLOCKED__") || message.includes("__AGET_CONFIRM_BLOCKED__") || message.includes("__AGET_ALERT_BLOCKED__")) {
      return JSON.stringify({ ok: false, code: "requires_confirmation", message: "interact blocked browser dialog" });
    }
    return JSON.stringify({ ok: false, code: "unsafe_action", message });
  } finally {
    window.open = oldOpen;
    window.alert = oldAlert;
    window.prompt = oldPrompt;
    window.confirm = oldConfirm;
    HTMLAnchorElement.prototype.click = oldAnchorClick;
  }
}
"#
}

fn with_prelude(body: String) -> String {
    format!("(() => {{{}\nreturn {};}})()", action_prelude(), body)
}

fn extension_for_format(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Markdown => "md",
        OutputFormat::Html => "html",
        OutputFormat::Text => "txt",
        OutputFormat::Json => "json",
    }
}

fn media_type_for_format(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Markdown => "text/markdown",
        OutputFormat::Html => "text/html",
        OutputFormat::Text => "text/plain",
        OutputFormat::Json => "application/json",
    }
}

fn safe_artifact_name(name: Option<&str>, fallback: &str) -> String {
    let raw = name.unwrap_or(fallback);
    let mut safe = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if safe.is_empty() {
        safe.push_str(fallback);
    }
    safe.truncate(80);
    safe
}

fn write_private_bytes(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        set_private_dir_permissions(parent)?;
    }
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    set_private_file_mode(&mut options);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    set_private_file_permissions(path)
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file_mode(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_private_file_mode(_options: &mut OpenOptions) {}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_scripts_include_safety_boundaries() {
        let click = click_script(&ClickAction {
            name: None,
            selector: "button#open".to_string(),
            timeout_ms: None,
        });
        let typed = type_script(&TypeAction {
            name: None,
            selector: "input[name=q]".to_string(),
            text: "docs".to_string(),
            sensitive: false,
            timeout_ms: None,
        });
        let selected = select_script(&SelectAction {
            name: None,
            selector: "select[name=version]".to_string(),
            value: Some("stable".to_string()),
            label: None,
            index: None,
            timeout_ms: None,
        });
        let submitted = submit_script(&SubmitAction {
            name: None,
            selector: "form#search".to_string(),
            confirm: true,
            timeout_ms: None,
        });
        let combined = [click, typed, selected, submitted].join("\n");

        assert!(combined.contains("document.querySelectorAll(selector)"));
        assert!(combined.contains("selector_ambiguous"));
        assert!(combined.contains("type === \"password\""));
        assert!(combined.contains("autocomplete === \"current-password\""));
        assert!(combined.contains("autocomplete === \"one-time-code\""));
        assert!(combined.contains("type === \"file\""));
        assert!(combined.contains("agetUnsafeForm"));
        assert!(combined.contains("input[type=\"password\"]"));
        assert!(combined.contains("hasAttribute(\"download\")"));
        assert!(combined.contains("HTMLAnchorElement.prototype.click"));
        assert!(combined.contains("element.form ||"));
        assert!(combined.contains("setTimeout(resolve, 1000)"));
        assert!(combined.contains("next.origin !== location.origin"));
        assert!(combined.contains("window.open"));
        assert!(combined.contains("window.alert"));
        assert!(combined.contains("window.prompt"));
        assert!(combined.contains("window.confirm"));
        assert!(combined.contains("requestSubmit"));
        assert!(combined.contains("finally"));
        assert!(combined.contains("window.open = oldOpen"));
        assert!(combined.contains("HTMLAnchorElement.prototype.click = oldAnchorClick"));
    }

    #[test]
    fn submit_script_uses_external_form_owner_for_form_controls() {
        let script = submit_script(&SubmitAction {
            name: None,
            selector: "button[form=\"search\"]".to_string(),
            confirm: true,
            timeout_ms: None,
        });

        assert!(script.contains("element.form ||"));
        assert!(script.contains("agetUnsafeForm(form)"));
        assert!(script.contains("agetBlockedNavigation(form || element)"));
        assert!(script.contains("form.requestSubmit(element)"));
    }

    #[test]
    fn capture_script_resolves_selector_clip_for_screenshots() {
        let script = capture_html_script(".result", Some(SelectorMatch::First));

        assert!(script.contains("document.querySelector"));
        assert!(script.contains("getBoundingClientRect"));
        assert!(script.contains("window.scrollX"));
        assert!(script.contains("window.scrollY"));
        assert!(script.contains("clip"));
    }

    #[test]
    fn action_error_codes_map_to_stable_error_codes() {
        assert_eq!(
            action_error_code("selector_not_found"),
            ErrorCode::SelectorNotFound
        );
        assert_eq!(
            action_error_code("selector_ambiguous"),
            ErrorCode::SelectorAmbiguous
        );
        assert_eq!(
            action_error_code("requires_confirmation"),
            ErrorCode::RequiresConfirmation
        );
        assert_eq!(action_error_code("unsafe_action"), ErrorCode::UnsafeAction);
        assert_eq!(
            action_error_code("navigation_blocked"),
            ErrorCode::NavigationBlocked
        );
        assert_eq!(
            action_error_code("action_not_supported"),
            ErrorCode::ActionNotSupported
        );
        assert_eq!(action_error_code("unknown"), ErrorCode::ExtractionFailed);
    }

    #[test]
    fn origin_comparison_blocks_cross_origin_navigation() {
        assert!(origins_match(
            "https://example.com/app",
            "https://example.com/search?q=docs"
        ));
        assert!(!origins_match(
            "https://example.com/app",
            "https://other.example/app"
        ));
    }

    #[test]
    fn interact_hazard_events_map_to_action_errors() {
        let download = serde_json::json!({"method": "Browser.downloadWillBegin", "params": {"url": "https://example.com/file"}});
        assert_eq!(
            interact_hazard_error(&[download], "https://example.com/app", "page-1")
                .unwrap()
                .code,
            ErrorCode::RequiresConfirmation
        );

        let popup = serde_json::json!({"method": "Target.targetCreated", "params": {"targetInfo": {"targetId": "page-2", "type": "page"}}});
        assert_eq!(
            interact_hazard_error(&[popup], "https://example.com/app", "page-1")
                .unwrap()
                .code,
            ErrorCode::RequiresConfirmation
        );

        let same_target = serde_json::json!({"method": "Target.targetCreated", "params": {"targetInfo": {"targetId": "page-1", "type": "page"}}});
        assert!(
            interact_hazard_error(&[same_target], "https://example.com/app", "page-1").is_none()
        );

        let cross_origin = serde_json::json!({"method": "Page.frameScheduledNavigation", "params": {"url": "https://other.example/app"}});
        assert_eq!(
            interact_hazard_error(&[cross_origin], "https://example.com/app", "page-1")
                .unwrap()
                .code,
            ErrorCode::NavigationBlocked
        );

        let same_origin = serde_json::json!({"method": "Page.frameScheduledNavigation", "params": {"url": "https://example.com/next"}});
        assert!(
            interact_hazard_error(&[same_origin], "https://example.com/app", "page-1").is_none()
        );
    }
}
