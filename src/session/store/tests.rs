use std::fs;
use std::io;
use std::time::{Duration, SystemTime};

use crate::session::Session;

use super::cleanup::{
    is_orphanable_tmp_file, sweep_orphaned_owned_chrome_import_profiles,
    sweep_orphaned_owned_chrome_profiles, sweep_orphaned_owned_login_profiles,
};
use super::*;

#[test]
fn creates_expected_layout() {
    let temp = tempfile::tempdir().unwrap();
    let store = SessionStore::new(temp.path().join("aget-home")).unwrap();

    assert!(store.home().is_dir());
    assert!(store.sessions_dir().is_dir());
    assert!(store.home().join("runs").is_dir());
    assert!(store.home().join("cache").is_dir());
    assert!(store.home().join("tmp").is_dir());
}

#[test]
fn saves_lists_loads_and_deletes_session() {
    let temp = tempfile::tempdir().unwrap();
    let store = SessionStore::new(temp.path().join("aget-home")).unwrap();
    let session = Session::new("demo");

    store.save(&session).unwrap();
    assert_eq!(store.list().unwrap(), vec!["demo".to_string()]);
    assert_eq!(store.load("demo").unwrap(), session);
    assert!(store.delete("demo").unwrap());
    assert!(!store.delete("demo").unwrap());
}

#[cfg(unix)]
#[test]
fn uses_private_unix_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let store = SessionStore::new(temp.path().join("aget-home")).unwrap();
    let session = Session::new("demo");
    store.save(&session).unwrap();

    let home_mode = fs::metadata(store.home()).unwrap().permissions().mode() & 0o777;
    let sessions_mode = fs::metadata(store.sessions_dir())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    let session_mode = fs::metadata(store.session_path("demo").unwrap())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(home_mode, 0o700);
    assert_eq!(sessions_mode, 0o700);
    assert_eq!(session_mode, 0o600);
}

#[test]
fn rejects_path_like_session_names() {
    let temp = tempfile::tempdir().unwrap();
    let store = SessionStore::new(temp.path().join("aget-home")).unwrap();

    for name in [
        "",
        ".",
        "..",
        "../secret",
        "nested/name",
        "nested\\name",
        "C:secret",
    ] {
        assert_eq!(
            store.load(name).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }
}

#[test]
fn orphan_sweep_recognizes_import_and_extractor_temp_files() {
    assert!(is_orphanable_tmp_file("login-raw-state-123.json"));
    assert!(is_orphanable_tmp_file("playwright-state-123.json"));
    assert!(!is_orphanable_tmp_file("owned-chrome"));
    assert!(!is_orphanable_tmp_file("aget-chrome-123"));
    assert!(!is_orphanable_tmp_file("login-active.json"));
}

#[test]
fn orphan_sweep_removes_owned_chrome_profiles() {
    let temp = tempfile::tempdir().unwrap();
    let owned_chrome = temp.path().join("owned-chrome");
    let profile = owned_chrome.join("aget-chrome-123");
    let unrelated = owned_chrome.join("keep-me");
    fs::create_dir_all(&profile).unwrap();
    fs::create_dir_all(&unrelated).unwrap();

    sweep_orphaned_owned_chrome_profiles(
        &owned_chrome,
        Duration::from_secs(0),
        SystemTime::now() + Duration::from_secs(1),
    )
    .unwrap();

    assert!(!profile.exists());
    assert!(unrelated.exists());
}

#[test]
fn orphan_sweep_removes_owned_chrome_import_profiles() {
    let temp = tempfile::tempdir().unwrap();
    let owned_chrome_import = temp.path().join("owned-chrome-import");
    let profile = owned_chrome_import.join("aget-profile-123");
    let unrelated = owned_chrome_import.join("keep-me");
    fs::create_dir_all(&profile).unwrap();
    fs::create_dir_all(&unrelated).unwrap();

    sweep_orphaned_owned_chrome_import_profiles(
        &owned_chrome_import,
        Duration::from_secs(0),
        SystemTime::now() + Duration::from_secs(1),
    )
    .unwrap();

    assert!(!profile.exists());
    assert!(unrelated.exists());
}

#[test]
fn orphan_sweep_removes_only_unreferenced_owned_login_profiles() {
    let temp = tempfile::tempdir().unwrap();
    let tmp_dir = temp.path();
    let owned_login = tmp_dir.join("owned-login");
    let orphaned = owned_login.join("aget-orphaned");
    let active = owned_login.join("aget-active");
    let unrelated = owned_login.join("keep-me");
    fs::create_dir_all(&orphaned).unwrap();
    fs::create_dir_all(&active).unwrap();
    fs::create_dir_all(&unrelated).unwrap();
    fs::write(tmp_dir.join("login-active.json"), "{}").unwrap();

    sweep_orphaned_owned_login_profiles(
        tmp_dir,
        &owned_login,
        Duration::from_secs(0),
        SystemTime::now() + Duration::from_secs(1),
    )
    .unwrap();

    assert!(!orphaned.exists());
    assert!(active.exists());
    assert!(unrelated.exists());
}
