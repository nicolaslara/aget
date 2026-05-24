use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use aget::{DoctorCheck, DoctorCommand, ErrorCode, ErrorResponse, ENVELOPE_SCHEMA_VERSION};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DoctorResult {
    pub(crate) summary: DoctorSummary,
    pub(crate) checks: Vec<DoctorDiagnostic>,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub(crate) struct DoctorSummary {
    pub(crate) ok: usize,
    pub(crate) warn: usize,
    pub(crate) fail: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DoctorDiagnostic {
    id: String,
    category: String,
    status: DoctorStatus,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remediation: Option<String>,
    sensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DoctorStatus {
    Ok,
    Warn,
    Fail,
}

pub(crate) fn run_doctor(
    command: DoctorCommand,
    json: bool,
    quiet: bool,
    started: Instant,
) -> Result<ExitCode, ErrorResponse> {
    let result = collect_diagnostics(&command);
    if json {
        print_doctor_envelope(&result, started)?;
    } else if !quiet {
        print_human(&result);
    }
    Ok(if result.summary.fail > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn collect_diagnostics(command: &DoctorCommand) -> DoctorResult {
    let selected = command.checks.iter().copied().collect::<BTreeSet<_>>();
    let mut checks = Vec::new();

    if selected.is_empty() || selected.contains(&DoctorCheck::Binary) {
        check_binary(&mut checks);
    }
    if selected.is_empty() || selected.contains(&DoctorCheck::Store) {
        check_store(&mut checks);
    }
    if selected.is_empty() || selected.contains(&DoctorCheck::Artifacts) {
        check_artifacts(&mut checks, command.quick);
    }
    if selected.is_empty() || selected.contains(&DoctorCheck::Chrome) {
        check_chrome(&mut checks);
    }
    if selected.is_empty() || selected.contains(&DoctorCheck::CurrentTab) {
        check_current_tab(&mut checks);
    }
    if selected.is_empty() || selected.contains(&DoctorCheck::Cmux) {
        check_cmux(&mut checks);
    }
    if selected.is_empty() || selected.contains(&DoctorCheck::Opencode) {
        check_opencode(&mut checks);
    }

    let mut summary = DoctorSummary::default();
    for check in &checks {
        match check.status {
            DoctorStatus::Ok => summary.ok += 1,
            DoctorStatus::Warn => summary.warn += 1,
            DoctorStatus::Fail => summary.fail += 1,
        }
    }

    DoctorResult { summary, checks }
}

fn check_binary(checks: &mut Vec<DoctorDiagnostic>) {
    checks.push(ok(
        "binary.version",
        "binary",
        format!("aget {}", env!("CARGO_PKG_VERSION")),
        Some(serde_json::json!({"version": env!("CARGO_PKG_VERSION")})),
    ));
    match env::current_exe() {
        Ok(path) => checks.push(ok(
            "binary.path",
            "binary",
            "current executable path is available",
            Some(serde_json::json!({"path": path})),
        )),
        Err(error) => checks.push(warn(
            "binary.path",
            "binary",
            format!("could not read current executable path: {error}"),
            None,
            None,
        )),
    }
    match env::current_dir() {
        Ok(path) => checks.push(ok(
            "binary.cwd",
            "binary",
            "current working directory is available",
            Some(serde_json::json!({"cwd": path})),
        )),
        Err(error) => checks.push(warn(
            "binary.cwd",
            "binary",
            format!("could not read current working directory: {error}"),
            None,
            None,
        )),
    }
}

fn check_store(checks: &mut Vec<DoctorDiagnostic>) {
    let home = effective_home();
    let Ok(home) = home else {
        checks.push(fail(
            "store.home",
            "store",
            "could not resolve AGET_HOME",
            None,
            Some("Set AGET_HOME or HOME to a local writable directory.".to_string()),
        ));
        return;
    };

    checks.push(ok(
        "store.home",
        "store",
        "resolved AGET_HOME",
        Some(serde_json::json!({"home": home})),
    ));

    let store = aget::SessionStore::new(&home);
    if let Err(error) = store {
        checks.push(fail(
            "store.layout",
            "store",
            format!("could not create or read AGET_HOME layout: {error}"),
            None,
            Some("Choose a writable AGET_HOME directory.".to_string()),
        ));
        return;
    }

    let required = ["sessions", "runs", "cache", "tmp"];
    let missing = required
        .iter()
        .filter(|name| !home.join(name).is_dir())
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        checks.push(ok(
            "store.layout",
            "store",
            "AGET_HOME layout is ready",
            Some(serde_json::json!({"dirs": required})),
        ));
    } else {
        checks.push(fail(
            "store.layout",
            "store",
            format!(
                "AGET_HOME layout is missing directories: {}",
                missing.join(", ")
            ),
            None,
            Some("Run any aget command with a writable AGET_HOME.".to_string()),
        ));
    }

    check_store_permissions(checks, &home);
    check_store_writable(checks, &home);
    check_sessions(checks, &home);
}

fn check_store_permissions(checks: &mut Vec<DoctorDiagnostic>, home: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut loose = Vec::new();
        for path in [home.to_path_buf(), home.join("sessions")] {
            match fs::metadata(&path) {
                Ok(metadata) => {
                    let mode = metadata.permissions().mode() & 0o777;
                    if mode != 0o700 {
                        loose.push(format!("{} is {mode:o}", path.display()));
                    }
                }
                Err(error) => loose.push(format!("{} unreadable: {error}", path.display())),
            }
        }
        if let Ok(entries) = fs::read_dir(home.join("sessions")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(metadata) = fs::metadata(&path) {
                    let mode = metadata.permissions().mode() & 0o777;
                    if mode != 0o600 {
                        loose.push(format!("{} is {mode:o}", path.display()));
                    }
                }
            }
        }
        if loose.is_empty() {
            checks.push(ok(
                "store.permissions",
                "store",
                "AGET_HOME permissions are private",
                None,
            ));
        } else {
            checks.push(warn(
                "store.permissions",
                "store",
                "some AGET_HOME paths are not private",
                Some(serde_json::json!({"paths": loose})),
                Some(
                    "Restrict AGET_HOME directories to 0700 and session files to 0600.".to_string(),
                ),
            ));
        }
    }

    #[cfg(not(unix))]
    {
        let _ = home;
        checks.push(ok(
            "store.permissions",
            "store",
            "permission check is not available on this platform",
            None,
        ));
    }
}

fn check_store_writable(checks: &mut Vec<DoctorDiagnostic>, home: &Path) {
    let probe = home.join("tmp").join("doctor-write-probe.tmp");
    match fs::write(&probe, b"aget doctor\n").and_then(|_| fs::remove_file(&probe)) {
        Ok(()) => checks.push(ok(
            "store.writable",
            "store",
            "AGET_HOME tmp directory is writable",
            None,
        )),
        Err(error) => checks.push(fail(
            "store.writable",
            "store",
            format!("AGET_HOME tmp directory is not writable: {error}"),
            None,
            Some("Fix permissions or set AGET_HOME to a writable directory.".to_string()),
        )),
    }
}

fn check_sessions(checks: &mut Vec<DoctorDiagnostic>, home: &Path) {
    match fs::read_dir(home.join("sessions")) {
        Ok(entries) => {
            let mut count = 0usize;
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|ext| ext.to_str()) == Some("json") {
                    count += 1;
                }
            }
            checks.push(ok(
                "store.sessions",
                "store",
                format!("{count} session file(s) found"),
                Some(serde_json::json!({"count": count})),
            ));
        }
        Err(error) => checks.push(fail(
            "store.sessions",
            "store",
            format!("could not read sessions directory: {error}"),
            None,
            Some("Fix AGET_HOME permissions.".to_string()),
        )),
    }
}

fn check_artifacts(checks: &mut Vec<DoctorDiagnostic>, quick: bool) {
    let Ok(home) = effective_home() else {
        checks.push(fail(
            "artifacts.runs_dir",
            "artifacts",
            "could not resolve AGET_HOME for artifact checks",
            None,
            None,
        ));
        return;
    };
    let runs = home.join("runs");
    let entries = match fs::read_dir(&runs) {
        Ok(entries) => entries.flatten().collect::<Vec<_>>(),
        Err(error) => {
            checks.push(fail(
                "artifacts.runs_dir",
                "artifacts",
                format!("could not read runs directory: {error}"),
                None,
                Some("Run aget with a writable AGET_HOME.".to_string()),
            ));
            return;
        }
    };
    let run_dirs = entries
        .iter()
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();
    checks.push(ok(
        "artifacts.count",
        "artifacts",
        format!("{} run artifact directorie(s) found", run_dirs.len()),
        Some(serde_json::json!({"count": run_dirs.len()})),
    ));

    if !quick {
        let size = dir_size(&runs).unwrap_or(0);
        checks.push(ok(
            "artifacts.size",
            "artifacts",
            format!("run artifacts use {size} bytes"),
            Some(serde_json::json!({"bytes": size})),
        ));
    }

    let malformed = run_dirs
        .iter()
        .take(20)
        .filter_map(|entry| {
            let metadata = entry.path().join("metadata.json");
            if !metadata.exists() {
                return None;
            }
            let text = fs::read_to_string(&metadata).ok()?;
            serde_json::from_str::<Value>(&text)
                .is_err()
                .then(|| entry.file_name().to_string_lossy().to_string())
        })
        .collect::<Vec<_>>();
    if malformed.is_empty() {
        checks.push(ok(
            "artifacts.metadata",
            "artifacts",
            "sampled run metadata is valid JSON",
            None,
        ));
    } else {
        checks.push(warn(
            "artifacts.metadata",
            "artifacts",
            "some sampled run metadata is malformed",
            Some(serde_json::json!({"runs": malformed})),
            Some("Inspect or prune malformed run artifacts.".to_string()),
        ));
    }
}

fn check_chrome(checks: &mut Vec<DoctorDiagnostic>) {
    match chrome_candidate() {
        Some(path) if path.is_file() => checks.push(ok(
            "chrome.command",
            "chrome",
            "Chrome/Chromium candidate found",
            Some(serde_json::json!({"path": path})),
        )),
        Some(path) => checks.push(warn(
            "chrome.command",
            "chrome",
            "configured Chrome command does not point to a file",
            Some(serde_json::json!({"path": path})),
            Some("Set AGET_CHROME_COMMAND to a Chrome/Chromium executable.".to_string()),
        )),
        None => checks.push(warn(
            "chrome.command",
            "chrome",
            "Chrome/Chromium was not found; static fetch still works",
            None,
            Some(
                "Install Chrome/Chromium or set AGET_CHROME_COMMAND for browser-backed flows."
                    .to_string(),
            ),
        )),
    }
}

fn check_current_tab(checks: &mut Vec<DoctorDiagnostic>) {
    checks.push(ok(
        "current_tab.command",
        "current_tab",
        "current-tab command is available and requires explicit local --cdp-port plus --allow-private-content",
        None,
    ));
}

fn check_cmux(checks: &mut Vec<DoctorDiagnostic>) {
    let configured = env::var_os("AGET_CMUX_COMMAND").map(PathBuf::from);
    let candidate = configured.clone().or_else(|| find_on_path("cmux"));
    match candidate {
        Some(path) if path.is_file() => checks.push(ok(
            "cmux.command",
            "cmux",
            "cmux command candidate found",
            Some(serde_json::json!({"path": path})),
        )),
        Some(path) => checks.push(warn(
            "cmux.command",
            "cmux",
            "configured cmux command does not point to a file",
            Some(serde_json::json!({"path": path})),
            Some("Set AGET_CMUX_COMMAND to cmux or remove it to use PATH.".to_string()),
        )),
        None => checks.push(warn(
            "cmux.command",
            "cmux",
            "cmux was not found; only session import cmux is unavailable",
            None,
            Some("Install cmux or set AGET_CMUX_COMMAND to use session import cmux.".to_string()),
        )),
    }
}

fn check_opencode(checks: &mut Vec<DoctorDiagnostic>) {
    let tool_path = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".opencode/tools/aget.ts");
    if tool_path.exists() {
        checks.push(ok(
            "opencode.tool",
            "opencode",
            ".opencode/tools/aget.ts found",
            Some(serde_json::json!({"path": tool_path})),
        ));
    } else {
        checks.push(ok(
            "opencode.tool",
            "opencode",
            "OpenCode aget tool is not present in this working tree",
            None,
        ));
    }

    if let Some(path) = env::var_os("AGET_OPENCODE_BIN").map(PathBuf::from) {
        if path.is_file() {
            checks.push(ok(
                "opencode.binary",
                "opencode",
                "AGET_OPENCODE_BIN points to a file",
                Some(serde_json::json!({"path": path})),
            ));
        } else {
            checks.push(warn(
                "opencode.binary",
                "opencode",
                "AGET_OPENCODE_BIN does not point to a file",
                Some(serde_json::json!({"path": path})),
                Some(
                    "Set AGET_OPENCODE_BIN to the aget binary or unset it to use PATH.".to_string(),
                ),
            ));
        }
    } else if let Some(path) = find_on_path("aget") {
        checks.push(ok(
            "opencode.binary",
            "opencode",
            "OpenCode wrapper can resolve aget from PATH",
            Some(serde_json::json!({"path": path})),
        ));
    } else {
        checks.push(warn(
            "opencode.binary",
            "opencode",
            "OpenCode wrapper may not resolve aget from PATH",
            None,
            Some("Install aget on PATH or set AGET_OPENCODE_BIN.".to_string()),
        ));
    }
}

fn effective_home() -> Result<PathBuf, ()> {
    if let Some(home) = env::var_os("AGET_HOME") {
        return Ok(PathBuf::from(home));
    }
    env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".aget"))
        .ok_or(())
}

fn chrome_candidate() -> Option<PathBuf> {
    if let Some(command) = env::var_os("AGET_CHROME_COMMAND") {
        if !command.is_empty() {
            return Some(PathBuf::from(command));
        }
    }

    for path in platform_chrome_candidates() {
        if path.is_file() {
            return Some(path);
        }
    }

    for command in [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "chrome",
        "brave-browser",
        "brave-browser-stable",
    ] {
        if let Some(path) = find_on_path(command) {
            return Some(path);
        }
    }
    None
}

fn platform_chrome_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    #[cfg(target_os = "macos")]
    {
        candidates.extend([
            PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            PathBuf::from(
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            ),
            PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"),
            PathBuf::from("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
        ]);
        if let Some(home) = env::var_os("HOME") {
            let home = PathBuf::from(home);
            candidates.extend([
                home.join("Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                home.join("Applications/Chromium.app/Contents/MacOS/Chromium"),
                home.join("Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
            ]);
        }
    }
    #[cfg(target_os = "windows")]
    {
        candidates.extend([
            PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
        ]);
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            let local = PathBuf::from(local);
            candidates.extend([
                local.join(r"Google\Chrome\Application\chrome.exe"),
                local.join(r"BraveSoftware\Brave-Browser\Application\brave.exe"),
            ]);
        }
    }
    candidates
}

fn find_on_path(command: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(command))
        .find(|path| path.is_file())
}

fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut size = 0u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            size = size.saturating_add(dir_size(&entry.path())?);
        } else {
            size = size.saturating_add(metadata.len());
        }
    }
    Ok(size)
}

fn print_doctor_envelope(result: &DoctorResult, started: Instant) -> Result<(), ErrorResponse> {
    let envelope = serde_json::json!({
        "ok": result.summary.fail == 0,
        "schema_version": ENVELOPE_SCHEMA_VERSION,
        "command": "doctor",
        "data": result,
        "warnings": [],
        "timing_ms": {"total": started.elapsed().as_millis()},
    });
    println!("{}", serde_json::to_string(&envelope).map_err(io_error)?);
    Ok(())
}

fn print_human(result: &DoctorResult) {
    println!("aget doctor\n");
    for check in &result.checks {
        println!(
            "{:<4} {:<28} {}",
            status_label(check.status),
            check.id,
            check.message
        );
        if let Some(remediation) = &check.remediation {
            println!("     {:<28} {}", "", remediation);
        }
    }
    println!(
        "\nSummary: {} ok, {} warn, {} fail",
        result.summary.ok, result.summary.warn, result.summary.fail
    );
}

fn status_label(status: DoctorStatus) -> &'static str {
    match status {
        DoctorStatus::Ok => "ok",
        DoctorStatus::Warn => "warn",
        DoctorStatus::Fail => "fail",
    }
}

fn ok(
    id: impl Into<String>,
    category: impl Into<String>,
    message: impl Into<String>,
    detail: Option<Value>,
) -> DoctorDiagnostic {
    diagnostic(id, category, DoctorStatus::Ok, message, detail, None)
}

fn warn(
    id: impl Into<String>,
    category: impl Into<String>,
    message: impl Into<String>,
    detail: Option<Value>,
    remediation: Option<String>,
) -> DoctorDiagnostic {
    diagnostic(
        id,
        category,
        DoctorStatus::Warn,
        message,
        detail,
        remediation,
    )
}

fn fail(
    id: impl Into<String>,
    category: impl Into<String>,
    message: impl Into<String>,
    detail: Option<Value>,
    remediation: Option<String>,
) -> DoctorDiagnostic {
    diagnostic(
        id,
        category,
        DoctorStatus::Fail,
        message,
        detail,
        remediation,
    )
}

fn diagnostic(
    id: impl Into<String>,
    category: impl Into<String>,
    status: DoctorStatus,
    message: impl Into<String>,
    detail: Option<Value>,
    remediation: Option<String>,
) -> DoctorDiagnostic {
    DoctorDiagnostic {
        id: id.into(),
        category: category.into(),
        status,
        message: message.into(),
        detail,
        remediation,
        sensitive: false,
    }
}

fn io_error(error: impl std::fmt::Display) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::IoError, error.to_string())
}
