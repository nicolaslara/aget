use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

pub fn mock_backend_command(dir: &Path, config: Value) -> String {
    shell_quote(&configured_tool(dir, "aget-mock-backend", config).to_string_lossy())
}

pub fn mock_agent_browser(dir: &Path, config: Value) -> PathBuf {
    configured_tool(dir, "aget-mock-agent-browser", config)
}

#[allow(dead_code)]
pub fn mock_cmux(dir: &Path, config: Value) -> PathBuf {
    configured_tool(dir, "aget-mock-cmux", config)
}

fn configured_tool(dir: &Path, bin: &str, config: Value) -> PathBuf {
    let source = mock_tool_bin(bin);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let exe_suffix = std::env::consts::EXE_SUFFIX;
    let configured = dir.join(format!("{bin}-{}-{nanos}{exe_suffix}", std::process::id()));
    fs::copy(&source, &configured).unwrap_or_else(|error| {
        panic!(
            "copy mock tool {} to {}: {error}",
            source.display(),
            configured.display()
        )
    });
    fs::write(
        configured.with_extension("json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    make_executable(&configured);
    configured
}

fn mock_tool_bin(bin: &str) -> PathBuf {
    static BUILD_ONCE: OnceLock<()> = OnceLock::new();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_manifest = manifest_dir.join("tests/fixtures/mock-tools/Cargo.toml");
    BUILD_ONCE.get_or_init(|| {
        let status = Command::new("cargo")
            .args([
                "build",
                "--quiet",
                "--manifest-path",
                fixture_manifest.to_str().unwrap(),
                "--bins",
            ])
            .status()
            .expect("cargo must be available to build mock tools");
        assert!(status.success(), "mock tool build failed with {status}");
    });

    let mut path = manifest_dir
        .join("tests/fixtures/mock-tools/target/debug")
        .join(bin);
    path.set_extension(std::env::consts::EXE_EXTENSION);
    if std::env::consts::EXE_EXTENSION.is_empty() {
        path.set_extension("");
    }
    assert!(
        path.exists(),
        "mock tool binary missing: {}",
        path.display()
    );
    path
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
