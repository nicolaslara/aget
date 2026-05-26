use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::chrome_process::ChromeProcess;
use super::client::{CdpClient, PageSession};
use super::{discover_cdp_ws_url, CHROME_SHUTDOWN_WAIT};
use crate::error::{AgetError, ErrorCode, ErrorResponse};
use crate::interact::{
    ActionDefinition, ActionPlan, BrowserActionExecution, BrowserActionResult, BrowserActionStatus,
    ClickAction, SelectAction, SubmitAction, TypeAction, WaitAction, WaitLoadState,
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
    execute_attached_actions(&mut client, &page, request)
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
    let execution = execute_attached_actions(&mut client, &page, request);
    let _ = client.close_page(&page, Duration::from_secs(1));
    let _ = client.close_browser(Duration::from_secs(1));
    chrome.wait_or_kill(CHROME_SHUTDOWN_WAIT);
    execution
}

fn execute_attached_actions(
    client: &mut CdpClient,
    page: &PageSession,
    request: BrowserActionPlanRequest<'_>,
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
            action,
            timeout,
            request.page_timeout,
            &initial_url,
        ) {
            Ok(()) => {
                match client.evaluate_string(
                    &page.session_id,
                    "location.href",
                    request.page_timeout,
                ) {
                    Ok(url) => {
                        final_url = url;
                        action_results.push(BrowserActionResult {
                            index,
                            action_type: action.kind(),
                            status: BrowserActionStatus::Ok,
                            elapsed_ms: started.elapsed().as_millis(),
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
                    error: Some(body),
                });
                break;
            }
        }
    }

    Ok(BrowserActionExecution {
        final_url: Some(final_url),
        action_results,
        warnings: Vec::new(),
        error,
    })
}

fn execute_action(
    client: &mut CdpClient,
    page: &PageSession,
    action: &ActionDefinition,
    timeout: Duration,
    page_timeout: Duration,
    initial_url: &str,
) -> Result<(), crate::error::ErrorBody> {
    match action {
        ActionDefinition::Wait(action) => execute_wait(client, page, action, timeout),
        ActionDefinition::Click(action) => {
            run_action_script(client, page, &click_script(action), timeout)?;
            enforce_same_origin(client, page, initial_url, page_timeout)
        }
        ActionDefinition::Type(action) => {
            run_action_script(client, page, &type_script(action), timeout)?;
            enforce_same_origin(client, page, initial_url, page_timeout)
        }
        ActionDefinition::Select(action) => {
            run_action_script(client, page, &select_script(action), timeout)?;
            enforce_same_origin(client, page, initial_url, page_timeout)
        }
        ActionDefinition::Submit(action) => {
            run_action_script(client, page, &submit_script(action), timeout)?;
            enforce_same_origin(client, page, initial_url, page_timeout)
        }
        ActionDefinition::Capture(_) | ActionDefinition::Extract(_) => Err(error_body(
            ErrorCode::ActionNotSupported,
            format!("{} actions are implemented in ACT-005", action.kind()),
        )),
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

fn enforce_same_origin(
    client: &mut CdpClient,
    page: &PageSession,
    initial_url: &str,
    timeout: Duration,
) -> Result<(), crate::error::ErrorBody> {
    let deadline = Instant::now() + ACTION_BOUNDARY_SETTLE.min(timeout);
    loop {
        let final_url = client
            .evaluate_string(
                &page.session_id,
                "location.href",
                timeout.min(Duration::from_secs(1)),
            )
            .map_err(error_body_from_aget)?;
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
}
