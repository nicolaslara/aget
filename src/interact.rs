use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use scraper::Selector;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::cli::OutputFormat;
use crate::error::{AgetError, ErrorBody};

pub const ACTIONS_SCHEMA_VERSION: &str = "aget.actions.v1";
const MAX_ACTIONS: usize = 50;
const MAX_CAPTURE_NAME_CHARS: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlanError {
    message: String,
}

impl ActionPlanError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for ActionPlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ActionPlanError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActionPlanValidationOptions {
    pub allow_sensitive_input: bool,
    pub allow_submit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserActionExecution {
    pub final_url: Option<String>,
    pub action_results: Vec<BrowserActionResult>,
    pub warnings: Vec<String>,
    pub error: Option<ErrorBody>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserActionResult {
    pub index: usize,
    pub action_type: &'static str,
    pub status: BrowserActionStatus,
    pub elapsed_ms: u128,
    pub error: Option<ErrorBody>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserActionStatus {
    Ok,
    DryRun,
    Failed,
}

pub enum BrowserActionRunSource<'a> {
    Url { url: &'a str, tmp_dir: &'a Path },
    CurrentTab { port: u16 },
}

pub struct BrowserActionRunOptions {
    pub timeout: Duration,
    pub default_action_timeout: Duration,
    pub page_timeout: Duration,
}

pub fn execute_browser_action_plan(
    plan: &ActionPlan,
    source: BrowserActionRunSource<'_>,
    options: BrowserActionRunOptions,
) -> Result<BrowserActionExecution, AgetError> {
    let source = match source {
        BrowserActionRunSource::Url { url, tmp_dir } => {
            crate::browser_cdp::BrowserActionSource::Url { url, tmp_dir }
        }
        BrowserActionRunSource::CurrentTab { port } => {
            crate::browser_cdp::BrowserActionSource::CurrentTab { port }
        }
    };
    crate::browser_cdp::execute_browser_action_plan(crate::browser_cdp::BrowserActionPlanRequest {
        source,
        plan,
        timeout: options.timeout,
        default_action_timeout: options.default_action_timeout,
        page_timeout: options.page_timeout,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlan {
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "ActionPlanDefaults::is_empty")]
    pub defaults: ActionPlanDefaults,
    pub actions: Vec<ActionDefinition>,
}

impl ActionPlan {
    pub fn validate(&self, options: ActionPlanValidationOptions) -> Result<(), ActionPlanError> {
        validate_action_plan(self, options)
    }

    pub fn redacted_request_json(&self) -> Value {
        redacted_action_request(self)
    }

    pub fn write_redacted_request(&self, path: &Path) -> Result<(), ActionPlanError> {
        write_redacted_action_request(path, self)
    }
}

impl ActionDefinition {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Wait(_) => "wait",
            Self::Click(_) => "click",
            Self::Type(_) => "type",
            Self::Select(_) => "select",
            Self::Submit(_) => "submit",
            Self::Capture(_) => "capture",
            Self::Extract(_) => "extract",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation_timeout_ms: Option<u64>,
}

impl ActionPlanDefaults {
    fn is_empty(&self) -> bool {
        self.action_timeout_ms.is_none() && self.navigation_timeout_ms.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionDefinition {
    Wait(WaitAction),
    Click(ClickAction),
    Type(TypeAction),
    Select(SelectAction),
    Submit(SubmitAction),
    Capture(CaptureAction),
    Extract(ExtractAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaitAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_state: Option<WaitLoadState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_mode: Option<SelectorMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClickAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub selector: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub selector: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sensitive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub selector: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub selector: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub confirm: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureAction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub screenshot: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub html: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sensitive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_mode: Option<SelectorMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtractAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(default = "default_content_format")]
    pub content_format: OutputFormat,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sensitive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_mode: Option<SelectorMatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitLoadState {
    DomContentLoaded,
    Load,
    NetworkIdle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectorMatch {
    Unique,
    First,
}

pub fn parse_action_plan_json(input: &str) -> Result<ActionPlan, ActionPlanError> {
    serde_json::from_str::<ActionPlan>(input)
        .map_err(|error| ActionPlanError::new(format!("invalid action plan JSON: {error}")))
}

pub fn validate_action_plan(
    plan: &ActionPlan,
    options: ActionPlanValidationOptions,
) -> Result<(), ActionPlanError> {
    if plan.schema_version != ACTIONS_SCHEMA_VERSION {
        return Err(ActionPlanError::new(format!(
            "unsupported action schema_version '{}'; expected '{ACTIONS_SCHEMA_VERSION}'",
            plan.schema_version
        )));
    }
    if plan.actions.is_empty() {
        return Err(ActionPlanError::new(
            "action plan must contain at least one action",
        ));
    }
    if plan.actions.len() > MAX_ACTIONS {
        return Err(ActionPlanError::new(format!(
            "action plan contains {} actions; maximum is {MAX_ACTIONS}",
            plan.actions.len()
        )));
    }

    let mut capture_names = HashSet::new();
    for (index, action) in plan.actions.iter().enumerate() {
        validate_action(index, action, options, &mut capture_names)?;
    }
    Ok(())
}

pub fn redacted_action_request(plan: &ActionPlan) -> Value {
    let actions = plan.actions.iter().map(redacted_action).collect::<Vec<_>>();
    let mut object = serde_json::Map::new();
    object.insert("schema_version".to_string(), json!(plan.schema_version));
    if !plan.defaults.is_empty() {
        object.insert("defaults".to_string(), json!(plan.defaults));
    }
    object.insert("actions".to_string(), Value::Array(actions));
    Value::Object(object)
}

pub fn write_redacted_action_request(
    path: &Path,
    plan: &ActionPlan,
) -> Result<(), ActionPlanError> {
    let bytes = serde_json::to_vec_pretty(&redacted_action_request(plan)).map_err(|error| {
        ActionPlanError::new(format!("serialize redacted action plan: {error}"))
    })?;
    write_private_file(path, &bytes).map_err(|error| {
        ActionPlanError::new(format!(
            "write redacted action request '{}': {error}",
            path.display()
        ))
    })
}

fn validate_action(
    index: usize,
    action: &ActionDefinition,
    options: ActionPlanValidationOptions,
    capture_names: &mut HashSet<String>,
) -> Result<(), ActionPlanError> {
    match action {
        ActionDefinition::Wait(action) => {
            validate_wait(index, action)?;
            if let Some(selector) = &action.selector {
                validate_selector(index, "wait selector", selector)?;
            }
        }
        ActionDefinition::Click(action) => {
            validate_selector(index, "click selector", &action.selector)?;
        }
        ActionDefinition::Type(action) => {
            validate_selector(index, "type selector", &action.selector)?;
            if action.sensitive && !options.allow_sensitive_input {
                return Err(ActionPlanError::new(format!(
                    "action {index} type marks text sensitive; pass --allow-sensitive-input"
                )));
            }
        }
        ActionDefinition::Select(action) => {
            validate_selector(index, "select selector", &action.selector)?;
            validate_one_select_choice(index, action)?;
        }
        ActionDefinition::Submit(action) => {
            validate_selector(index, "submit selector", &action.selector)?;
            if !action.confirm || !options.allow_submit {
                return Err(ActionPlanError::new(format!(
                    "action {index} submit requires confirm=true and --allow-submit"
                )));
            }
        }
        ActionDefinition::Capture(action) => {
            validate_capture_name(index, &action.name)?;
            if !action.screenshot && !action.html {
                return Err(ActionPlanError::new(format!(
                    "action {index} capture must request screenshot or html"
                )));
            }
            if !capture_names.insert(action.name.clone()) {
                return Err(ActionPlanError::new(format!(
                    "action {index} capture name '{}' is duplicated",
                    action.name
                )));
            }
            if let Some(selector) = &action.selector {
                validate_selector(index, "capture selector", selector)?;
            }
        }
        ActionDefinition::Extract(action) => {
            if let Some(selector) = &action.selector {
                validate_selector(index, "extract selector", selector)?;
            }
        }
    }
    Ok(())
}

fn validate_wait(index: usize, action: &WaitAction) -> Result<(), ActionPlanError> {
    let mut modes = 0;
    if action.selector.is_some() {
        modes += 1;
    }
    if action.load_state.is_some() {
        modes += 1;
    }
    if action.duration_ms.is_some() {
        modes += 1;
    }
    if modes != 1 {
        return Err(ActionPlanError::new(format!(
            "action {index} wait must specify exactly one of selector, load_state, or duration_ms"
        )));
    }
    Ok(())
}

fn validate_one_select_choice(index: usize, action: &SelectAction) -> Result<(), ActionPlanError> {
    let mut choices = 0;
    if action.value.is_some() {
        choices += 1;
    }
    if action.label.is_some() {
        choices += 1;
    }
    if action.index.is_some() {
        choices += 1;
    }
    if choices != 1 {
        return Err(ActionPlanError::new(format!(
            "action {index} select must specify exactly one of value, label, or index"
        )));
    }
    Ok(())
}

fn validate_selector(index: usize, label: &str, selector: &str) -> Result<(), ActionPlanError> {
    Selector::parse(selector).map_err(|error| {
        ActionPlanError::new(format!("action {index} {label} is invalid: {error}"))
    })?;
    Ok(())
}

fn validate_capture_name(index: usize, name: &str) -> Result<(), ActionPlanError> {
    if name.is_empty() {
        return Err(ActionPlanError::new(format!(
            "action {index} capture name must not be empty"
        )));
    }
    if name.chars().count() > MAX_CAPTURE_NAME_CHARS {
        return Err(ActionPlanError::new(format!(
            "action {index} capture name exceeds {MAX_CAPTURE_NAME_CHARS} characters"
        )));
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(ActionPlanError::new(format!(
            "action {index} capture name must contain only ASCII letters, digits, dash, underscore, or dot"
        )));
    }
    Ok(())
}

fn redacted_action(action: &ActionDefinition) -> Value {
    let mut value = serde_json::to_value(action).expect("action serializes");
    if let (ActionDefinition::Type(action), Value::Object(object)) = (action, &mut value) {
        if action.sensitive {
            object.insert("text".to_string(), Value::String("<redacted>".to_string()));
            object.insert("text_redacted".to_string(), Value::Bool(true));
            object.insert("text_chars".to_string(), json!(action.text.chars().count()));
        }
    }
    value
}

fn write_private_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        set_private_dir_permissions(parent)?;
    }
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    set_private_file_mode(&mut options);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.write_all(b"\n")?;
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

fn default_content_format() -> OutputFormat {
    OutputFormat::Markdown
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> ActionPlan {
        parse_action_plan_json(input).expect("plan parses")
    }

    fn validate(input: &str, options: ActionPlanValidationOptions) -> Result<(), ActionPlanError> {
        parse(input).validate(options)
    }

    #[test]
    fn parses_and_validates_all_supported_actions() {
        let plan = parse(
            r##"{
              "schema_version": "aget.actions.v1",
              "defaults": {"action_timeout_ms": 5000, "navigation_timeout_ms": 15000},
              "actions": [
                {"type": "wait", "selector": "main"},
                {"type": "click", "selector": "button.search"},
                {"type": "type", "selector": "input[name=q]", "text": "install instructions"},
                {"type": "select", "selector": "select[name=version]", "value": "stable"},
                {"type": "submit", "selector": "form#search", "confirm": true},
                {"type": "capture", "name": "after-search", "screenshot": true, "html": true},
                {"type": "extract", "selector": "main", "content_format": "markdown"}
              ]
            }"##,
        );

        plan.validate(ActionPlanValidationOptions {
            allow_sensitive_input: false,
            allow_submit: true,
        })
        .unwrap();
    }

    #[test]
    fn rejects_unknown_top_level_and_action_fields() {
        let error = parse_action_plan_json(
            r#"{"schema_version":"aget.actions.v1","extra":true,"actions":[]}"#,
        )
        .unwrap_err();
        assert!(error.message().contains("unknown field"));

        let error = parse_action_plan_json(
            r#"{"schema_version":"aget.actions.v1","actions":[{"type":"click","selector":"button","extra":true}]}"#,
        )
        .unwrap_err();
        assert!(error.message().contains("unknown field"));
    }

    #[test]
    fn rejects_missing_required_fields() {
        let error = parse_action_plan_json(
            r#"{"schema_version":"aget.actions.v1","actions":[{"type":"click"}]}"#,
        )
        .unwrap_err();

        assert!(error.message().contains("missing field `selector`"));
    }

    #[test]
    fn rejects_invalid_selectors_and_capture_names() {
        let error = validate(
            r#"{"schema_version":"aget.actions.v1","actions":[{"type":"click","selector":"["}]}"#,
            ActionPlanValidationOptions::default(),
        )
        .unwrap_err();
        assert!(error.message().contains("click selector is invalid"));

        let error = validate(
            r#"{"schema_version":"aget.actions.v1","actions":[{"type":"capture","name":"bad/name","html":true}]}"#,
            ActionPlanValidationOptions::default(),
        )
        .unwrap_err();
        assert!(error.message().contains("capture name must contain only"));
    }

    #[test]
    fn rejects_duplicate_capture_names() {
        let error = validate(
            r#"{
              "schema_version":"aget.actions.v1",
              "actions":[
                {"type":"capture","name":"same","html":true},
                {"type":"capture","name":"same","screenshot":true}
              ]
            }"#,
            ActionPlanValidationOptions::default(),
        )
        .unwrap_err();

        assert!(error.message().contains("duplicated"));
    }

    #[test]
    fn rejects_invalid_wait_and_select_shapes() {
        let error = validate(
            r#"{"schema_version":"aget.actions.v1","actions":[{"type":"wait","selector":"main","duration_ms":10}]}"#,
            ActionPlanValidationOptions::default(),
        )
        .unwrap_err();
        assert!(error.message().contains("exactly one of selector"));

        let error = validate(
            r#"{"schema_version":"aget.actions.v1","actions":[{"type":"select","selector":"select","value":"a","label":"A"}]}"#,
            ActionPlanValidationOptions::default(),
        )
        .unwrap_err();
        assert!(error.message().contains("exactly one of value"));
    }

    #[test]
    fn rejects_too_many_actions() {
        let actions = (0..=MAX_ACTIONS)
            .map(|_| r#"{"type":"wait","duration_ms":1}"#)
            .collect::<Vec<_>>()
            .join(",");
        let input = format!(r#"{{"schema_version":"aget.actions.v1","actions":[{actions}]}}"#);

        let error = validate(&input, ActionPlanValidationOptions::default()).unwrap_err();

        assert!(error.message().contains("maximum is"));
    }

    #[test]
    fn rejects_sensitive_typing_without_consent() {
        let input = r#"{
          "schema_version":"aget.actions.v1",
          "actions":[{"type":"type","selector":"input","text":"secret-token","sensitive":true}]
        }"#;

        let error = validate(input, ActionPlanValidationOptions::default()).unwrap_err();
        assert!(error.message().contains("--allow-sensitive-input"));

        validate(
            input,
            ActionPlanValidationOptions {
                allow_sensitive_input: true,
                allow_submit: false,
            },
        )
        .unwrap();
    }

    #[test]
    fn rejects_submit_without_confirmation_and_consent() {
        let input = r#"{
          "schema_version":"aget.actions.v1",
          "actions":[{"type":"submit","selector":"form","confirm":true}]
        }"#;

        let error = validate(input, ActionPlanValidationOptions::default()).unwrap_err();
        assert!(error.message().contains("--allow-submit"));

        validate(
            input,
            ActionPlanValidationOptions {
                allow_sensitive_input: false,
                allow_submit: true,
            },
        )
        .unwrap();
    }

    #[test]
    fn redacts_sensitive_typed_text() {
        let plan = parse(
            r#"{
              "schema_version":"aget.actions.v1",
              "actions":[
                {"type":"type","selector":"input.public","text":"public"},
                {"type":"type","selector":"input.secret","text":"secret-token","sensitive":true}
              ]
            }"#,
        );

        let redacted = plan.redacted_request_json();
        let encoded = serde_json::to_string(&redacted).unwrap();

        assert!(encoded.contains("public"));
        assert!(!encoded.contains("secret-token"));
        assert_eq!(redacted["actions"][1]["text"], "<redacted>");
        assert_eq!(redacted["actions"][1]["text_redacted"], true);
        assert_eq!(redacted["actions"][1]["text_chars"], 12);
    }

    #[test]
    fn writes_redacted_request_artifact() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("runs/run-1/actions-request.json");
        let plan = parse(
            r#"{
              "schema_version":"aget.actions.v1",
              "actions":[{"type":"type","selector":"input","text":"secret-token","sensitive":true}]
            }"#,
        );

        plan.write_redacted_request(&path).unwrap();

        let text = std::fs::read_to_string(path).unwrap();
        assert!(!text.contains("secret-token"));
        assert!(text.contains("<redacted>"));
    }
}
