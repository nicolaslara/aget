use std::fs;
use std::path::Path;

use super::profile::{copy_chrome_profile, explicit_profile_dir, resolve_chrome_profile};
use crate::error::ErrorCode;

#[test]
fn resolves_chrome_profile_by_directory_display_and_case_insensitive_directory() {
    let temp = tempfile::tempdir().unwrap();
    write_local_state(
        temp.path(),
        &[("Default", "Person 1"), ("Profile 1", "Work")],
    );

    assert_eq!(
        resolve_chrome_profile(temp.path(), "Default").unwrap(),
        "Default"
    );
    assert_eq!(
        resolve_chrome_profile(temp.path(), "work").unwrap(),
        "Profile 1"
    );
    assert_eq!(
        resolve_chrome_profile(temp.path(), "profile 1").unwrap(),
        "Profile 1"
    );
}

#[test]
fn resolving_chrome_profile_reports_ambiguous_display_name() {
    let temp = tempfile::tempdir().unwrap();
    write_local_state(temp.path(), &[("Default", "Work"), ("Profile 1", "Work")]);

    let error = resolve_chrome_profile(temp.path(), "work").unwrap_err();

    assert_eq!(error.code(), ErrorCode::RequiresUserAction);
    assert!(error.to_string().contains("ambiguous profile name"));
    assert!(error.to_string().contains("Default"));
    assert!(error.to_string().contains("Profile 1"));
}

#[test]
fn resolving_chrome_profile_reports_available_profiles_when_missing() {
    let temp = tempfile::tempdir().unwrap();
    write_local_state(temp.path(), &[("Default", "Person 1")]);

    let error = resolve_chrome_profile(temp.path(), "Missing").unwrap_err();

    assert_eq!(error.code(), ErrorCode::RequiresUserAction);
    assert!(error.to_string().contains("could not find profile"));
    assert!(error.to_string().contains("Default"));
}

#[test]
fn owned_import_requires_existing_profile_path() {
    let missing = std::env::temp_dir().join(format!("aget-missing-profile-{}", std::process::id()));
    let error = explicit_profile_dir(&missing.to_string_lossy()).unwrap_err();

    assert_eq!(error.code(), ErrorCode::RequiresUserAction);
}

#[test]
fn owned_import_accepts_existing_profile_path() {
    let temp = tempfile::tempdir().unwrap();
    let profile = explicit_profile_dir(&temp.path().to_string_lossy()).unwrap();

    assert_eq!(profile, temp.path());
}

#[test]
fn copy_chrome_profile_copies_local_state_and_excludes_cache_and_locks() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let tmp = temp.path().join("tmp");
    write_local_state(&source, &[("Default", "Person 1")]);
    fs::create_dir_all(source.join("Default/Local Storage/leveldb")).unwrap();
    fs::write(source.join("Default/Cookies"), "cookies").unwrap();
    fs::write(source.join("Default/SingletonLock"), "lock").unwrap();
    fs::create_dir_all(source.join("Default/Cache")).unwrap();
    fs::write(source.join("Default/Cache/data_0"), "cache").unwrap();
    fs::write(
        source.join("Default/Local Storage/leveldb/CURRENT"),
        "storage",
    )
    .unwrap();

    let copied = copy_chrome_profile(&source, "Default", &tmp).unwrap();
    let copied_path = copied.path.clone();

    assert!(copied_path.join("Local State").is_file());
    assert_eq!(
        fs::read_to_string(copied_path.join("Default/Cookies")).unwrap(),
        "cookies"
    );
    assert!(copied_path
        .join("Default/Local Storage/leveldb/CURRENT")
        .is_file());
    assert!(!copied_path.join("Default/SingletonLock").exists());
    assert!(!copied_path.join("Default/Cache").exists());
    drop(copied);
    assert!(!copied_path.exists());
}

#[cfg(unix)]
#[test]
fn copied_chrome_profile_parent_uses_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let tmp = temp.path().join("tmp");
    write_local_state(&source, &[("Default", "Person 1")]);
    fs::create_dir_all(source.join("Default")).unwrap();

    let copied = copy_chrome_profile(&source, "Default", &tmp).unwrap();
    let mode = fs::metadata(&copied.path).unwrap().permissions().mode() & 0o777;

    assert_eq!(mode, 0o700);
}

fn write_local_state(user_data_dir: &Path, profiles: &[(&str, &str)]) {
    fs::create_dir_all(user_data_dir).unwrap();
    let info_cache = profiles
        .iter()
        .map(|(directory, name)| {
            (
                directory.to_string(),
                serde_json::json!({
                    "name": name,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let local_state = serde_json::json!({
        "profile": {
            "info_cache": info_cache,
        }
    });
    fs::write(
        user_data_dir.join("Local State"),
        serde_json::to_string(&local_state).unwrap(),
    )
    .unwrap();
}
