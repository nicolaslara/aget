use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aget::error::ErrorBody;
use aget::{
    parse_action_plan_json, ActionDefinition, ActionPlan, ActionPlanValidationOptions, ErrorCode,
    ErrorResponse, InteractCommand, SessionStore, ENVELOPE_SCHEMA_VERSION,
};
use serde::Serialize;

pub(super) fn run_interact(
    command: InteractCommand,
    json: bool,
    quiet: bool,
    started: Instant,
) -> Result<ExitCode, ErrorResponse> {
    let executor: Box<dyn InteractExecutor> = if command.dry_run {
        Box::new(DryRunInteractExecutor)
    } else {
        Box::new(UnsupportedInteractExecutor)
    };
    run_interact_with_executor(command, json, quiet, started, executor.as_ref())
}

fn run_interact_with_executor(
    command: InteractCommand,
    json: bool,
    quiet: bool,
    started: Instant,
    executor: &dyn InteractExecutor,
) -> Result<ExitCode, ErrorResponse> {
    if !command.allow_actions {
        return Err(usage_error(
            "interact can mutate browser/page state; pass --allow-actions",
        ));
    }
    validate_source_consent(&command)?;

    let action_text = fs::read_to_string(&command.actions).map_err(super::io_error)?;
    let plan =
        parse_action_plan_json(&action_text).map_err(|error| usage_error(error.message()))?;
    plan.validate(ActionPlanValidationOptions {
        allow_sensitive_input: command.allow_sensitive_input,
        allow_submit: command.allow_submit,
    })
    .map_err(|error| usage_error(error.message()))?;
    validate_capture_consent(&command, &plan)?;

    let store = SessionStore::from_env().map_err(super::io_error)?;
    let run_id = interact_run_id();
    let run_dir = store.home().join("runs").join(&run_id);
    create_private_dir(&run_dir).map_err(super::io_error)?;

    let artifacts = InteractArtifacts {
        metadata: run_dir.join("metadata.json"),
        actions_request: run_dir.join("actions-request.json"),
        actions_result: run_dir.join("actions-result.json"),
    };
    plan.write_redacted_request(&artifacts.actions_request)
        .map_err(|error| usage_error(error.message()))?;

    let sensitive = source_sensitive(&command);
    let execution = executor.execute(&command, &plan);
    let actions = InteractActionSummary::from_results(
        plan.actions.len(),
        artifacts.actions_result.clone(),
        &execution.action_results,
    );
    let source = InteractSource {
        kind: if command.source == "current-tab" {
            "current_tab"
        } else {
            "url"
        },
        url: (command.source != "current-tab").then(|| command.source.clone()),
        cdp_port: command.cdp_port,
        sessions: command.session.clone(),
        sensitive,
    };
    let data = InteractRunData {
        run_id,
        source,
        initial_url: command.source.clone(),
        final_url: execution.final_url.clone(),
        actions,
        artifacts,
        sensitive,
    };

    write_actions_result(&data.artifacts.actions_result, &execution)?;
    write_metadata(
        &data.artifacts.metadata,
        &data,
        &execution,
        started.elapsed().as_millis(),
    )?;

    if let Some(error) = execution.error {
        if json {
            print_interact_failure_envelope(&data, &error, execution.warnings, started)?;
        } else if !quiet {
            eprintln!("Interact failed: {}", error.message);
            eprintln!("Run: {}", data.run_id);
            eprintln!("Actions: {}", data.artifacts.actions_result.display());
        }
        Ok(ExitCode::from(1))
    } else {
        if json {
            super::print_success_envelope(
                "interact",
                serde_json::to_value(&data).map_err(super::io_error)?,
                execution.warnings,
                aget::TimingMs {
                    total: started.elapsed().as_millis(),
                },
            )?;
        } else if !quiet {
            println!("Interact: {} action(s) recorded", data.actions.total);
            println!("Run: {}", data.run_id);
            println!("Actions: {}", data.artifacts.actions_result.display());
        }
        Ok(ExitCode::SUCCESS)
    }
}

trait InteractExecutor {
    fn execute(&self, command: &InteractCommand, plan: &ActionPlan) -> InteractExecution;
}

struct DryRunInteractExecutor;

impl InteractExecutor for DryRunInteractExecutor {
    fn execute(&self, command: &InteractCommand, plan: &ActionPlan) -> InteractExecution {
        InteractExecution {
            final_url: Some(command.source.clone()),
            action_results: plan
                .actions
                .iter()
                .enumerate()
                .map(|(index, action)| InteractActionResult::dry_run(index, action))
                .collect(),
            warnings: vec![
                "interact dry-run validated the action plan but did not execute browser actions"
                    .to_string(),
            ],
            error: None,
        }
    }
}

struct UnsupportedInteractExecutor;

impl InteractExecutor for UnsupportedInteractExecutor {
    fn execute(&self, _command: &InteractCommand, plan: &ActionPlan) -> InteractExecution {
        let error = ErrorBody {
            code: ErrorCode::BackendUnavailable,
            message:
                "interact browser execution is not implemented yet; use --dry-run to validate the plan"
                    .to_string(),
            retry: Some("Rerun with --dry-run, or wait for ACT-004 browser action support.".to_string()),
        };
        let action_results = plan
            .actions
            .first()
            .map(|action| vec![InteractActionResult::failed(0, action, error.clone())])
            .unwrap_or_default();
        InteractExecution {
            final_url: None,
            action_results,
            warnings: Vec::new(),
            error: Some(error),
        }
    }
}

#[derive(Debug)]
struct InteractExecution {
    final_url: Option<String>,
    action_results: Vec<InteractActionResult>,
    warnings: Vec<String>,
    error: Option<ErrorBody>,
}

#[derive(Debug, Clone, Serialize)]
struct InteractRunData {
    run_id: String,
    source: InteractSource,
    initial_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_url: Option<String>,
    actions: InteractActionSummary,
    artifacts: InteractArtifacts,
    sensitive: bool,
}

#[derive(Debug, Clone, Serialize)]
struct InteractSource {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cdp_port: Option<u16>,
    sessions: Vec<String>,
    sensitive: bool,
}

#[derive(Debug, Clone, Serialize)]
struct InteractArtifacts {
    metadata: PathBuf,
    actions_request: PathBuf,
    actions_result: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct InteractActionSummary {
    total: usize,
    succeeded: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    failed_index: Option<usize>,
    result_path: PathBuf,
}

impl InteractActionSummary {
    fn from_results(total: usize, result_path: PathBuf, results: &[InteractActionResult]) -> Self {
        let failed_index = results
            .iter()
            .find(|result| result.status == InteractActionStatus::Failed)
            .map(|result| result.index);
        let succeeded = if failed_index.is_some() {
            results
                .iter()
                .filter(|result| result.status != InteractActionStatus::Failed)
                .count()
        } else {
            total
        };
        Self {
            total,
            succeeded,
            failed_index,
            result_path,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct InteractActionResult {
    index: usize,
    action_type: &'static str,
    status: InteractActionStatus,
    elapsed_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorBody>,
}

impl InteractActionResult {
    fn dry_run(index: usize, action: &ActionDefinition) -> Self {
        Self {
            index,
            action_type: action.kind(),
            status: InteractActionStatus::DryRun,
            elapsed_ms: 0,
            error: None,
        }
    }

    fn failed(index: usize, action: &ActionDefinition, error: ErrorBody) -> Self {
        Self {
            index,
            action_type: action.kind(),
            status: InteractActionStatus::Failed,
            elapsed_ms: 0,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum InteractActionStatus {
    DryRun,
    Failed,
}

fn validate_source_consent(command: &InteractCommand) -> Result<(), ErrorResponse> {
    if command.source == "current-tab" {
        if command.cdp_port.is_none() {
            return Err(usage_error(
                "interact current-tab requires --cdp-port for the selected local browser",
            ));
        }
        if !command.allow_private_content {
            return Err(usage_error(
                "interact current-tab may read private browser content; pass --allow-private-content",
            ));
        }
    } else if command.cdp_port.is_some() {
        return Err(usage_error(
            "--cdp-port is only valid with `aget interact current-tab`",
        ));
    }

    if !command.session.is_empty() && !command.allow_private_content {
        return Err(usage_error(
            "interact with --session may read private content; pass --allow-private-content",
        ));
    }
    Ok(())
}

fn validate_capture_consent(
    command: &InteractCommand,
    plan: &ActionPlan,
) -> Result<(), ErrorResponse> {
    if plan.actions.iter().any(|action| {
        matches!(
            action,
            ActionDefinition::Capture(capture) if capture.screenshot
        )
    }) && !command.capture_screenshot
    {
        return Err(usage_error(
            "capture actions with screenshot=true require --capture-screenshot",
        ));
    }
    Ok(())
}

fn source_sensitive(command: &InteractCommand) -> bool {
    command.source == "current-tab" || !command.session.is_empty()
}

fn interact_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("run-{}-{nanos}", std::process::id())
}

fn write_actions_result(path: &Path, execution: &InteractExecution) -> Result<(), ErrorResponse> {
    let value = serde_json::json!({
        "ok": execution.error.is_none(),
        "actions": execution.action_results,
        "error": execution.error,
        "warnings": execution.warnings,
    });
    write_private_json(path, &value)
}

fn write_metadata(
    path: &Path,
    data: &InteractRunData,
    execution: &InteractExecution,
    elapsed_ms: u128,
) -> Result<(), ErrorResponse> {
    let value = serde_json::json!({
        "ok": execution.error.is_none(),
        "command": "interact",
        "url": data.initial_url,
        "final_url": data.final_url,
        "artifacts": data.artifacts,
        "actions": data.actions,
        "sensitive": data.sensitive,
        "warnings": execution.warnings,
        "timing_ms": {"total": elapsed_ms},
        "error": execution.error,
    });
    write_private_json(path, &value)
}

fn write_private_json(path: &Path, value: &impl Serialize) -> Result<(), ErrorResponse> {
    let bytes = serde_json::to_vec_pretty(value).map_err(super::io_error)?;
    write_private_file(path, &bytes).map_err(super::io_error)
}

fn print_interact_failure_envelope(
    data: &InteractRunData,
    error: &ErrorBody,
    warnings: Vec<String>,
    started: Instant,
) -> Result<(), ErrorResponse> {
    let envelope = serde_json::json!({
        "ok": false,
        "schema_version": ENVELOPE_SCHEMA_VERSION,
        "command": "interact",
        "error": error,
        "data": data,
        "warnings": warnings,
        "timing_ms": {"total": started.elapsed().as_millis()},
    });
    println!(
        "{}",
        serde_json::to_string(&envelope).map_err(super::io_error)?
    );
    Ok(())
}

fn usage_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::UsageError, message)
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    set_private_dir_permissions(path)
}

fn write_private_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        create_private_dir(parent)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> ActionPlan {
        parse_action_plan_json(
            r#"{
              "schema_version":"aget.actions.v1",
              "actions":[
                {"type":"wait","duration_ms":1},
                {"type":"click","selector":"button"}
              ]
            }"#,
        )
        .unwrap()
    }

    fn command() -> InteractCommand {
        InteractCommand {
            source: "https://example.com/app".to_string(),
            actions: PathBuf::from("actions.json"),
            allow_actions: true,
            allow_private_content: false,
            allow_sensitive_input: false,
            allow_submit: false,
            capture_screenshot: false,
            cdp_port: None,
            session: Vec::new(),
            dry_run: true,
        }
    }

    #[test]
    fn dry_run_executor_preserves_action_sequence() {
        let plan = plan();
        let execution = DryRunInteractExecutor.execute(&command(), &plan);

        assert!(execution.error.is_none());
        assert_eq!(execution.action_results.len(), 2);
        assert_eq!(execution.action_results[0].index, 0);
        assert_eq!(execution.action_results[0].action_type, "wait");
        assert_eq!(execution.action_results[1].index, 1);
        assert_eq!(execution.action_results[1].action_type, "click");
    }

    #[test]
    fn fake_executor_result_can_report_timeout_partial_failure() {
        let plan = plan();
        let timeout = ErrorBody {
            code: ErrorCode::Timeout,
            message: "action 1 timed out".to_string(),
            retry: None,
        };
        let results = vec![
            InteractActionResult::dry_run(0, &plan.actions[0]),
            InteractActionResult::failed(1, &plan.actions[1], timeout),
        ];
        let summary = InteractActionSummary::from_results(
            plan.actions.len(),
            PathBuf::from("result"),
            &results,
        );

        assert_eq!(summary.total, 2);
        assert_eq!(summary.succeeded, 1);
        assert_eq!(summary.failed_index, Some(1));
    }
}
