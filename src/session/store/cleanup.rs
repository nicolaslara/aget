use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};

pub(super) fn sweep_orphaned_tmp(tmp_dir: &Path, min_age: Duration) -> io::Result<()> {
    let now = SystemTime::now();
    for entry in fs::read_dir(tmp_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if file_name == "owned-chrome" {
            sweep_orphaned_owned_chrome_profiles(&path, min_age, now)?;
            continue;
        }
        if file_name == "owned-chrome-import" {
            sweep_orphaned_owned_chrome_import_profiles(&path, min_age, now)?;
            continue;
        }
        if file_name == "owned-login" {
            sweep_orphaned_owned_login_profiles(tmp_dir, &path, min_age, now)?;
            continue;
        }
        if is_orphanable_tmp_file(file_name) && is_older_than(&path, min_age, now) {
            let _ = fs::remove_file(path);
        }
    }
    Ok(())
}

pub(super) fn sweep_orphaned_owned_chrome_profiles(
    dir: &Path,
    min_age: Duration,
    now: SystemTime,
) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if file_name.starts_with("aget-chrome-") && is_older_than(&path, min_age, now) {
            let _ = fs::remove_dir_all(path);
        }
    }
    Ok(())
}

pub(super) fn sweep_orphaned_owned_chrome_import_profiles(
    dir: &Path,
    min_age: Duration,
    now: SystemTime,
) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if file_name.starts_with("aget-profile-") && is_older_than(&path, min_age, now) {
            let _ = fs::remove_dir_all(path);
        }
    }
    Ok(())
}

pub(super) fn sweep_orphaned_owned_login_profiles(
    tmp_dir: &Path,
    dir: &Path,
    min_age: Duration,
    now: SystemTime,
) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let Some(name) = file_name.strip_prefix("aget-") else {
            continue;
        };
        let pending_path = tmp_dir.join(format!("login-{name}.json"));
        if !pending_path.exists() && is_older_than(&path, min_age, now) {
            let _ = fs::remove_dir_all(path);
        }
    }
    Ok(())
}

pub(super) fn is_orphanable_tmp_file(file_name: &str) -> bool {
    (file_name.starts_with("playwright-state-") && file_name.ends_with(".json"))
        || (file_name.starts_with("login-raw-state-") && file_name.ends_with(".json"))
        || (file_name.starts_with("cmux-stdout-") && file_name.ends_with(".txt"))
        || (file_name.starts_with("cmux-stderr-") && file_name.ends_with(".txt"))
}

fn is_older_than(path: &Path, min_age: Duration, now: SystemTime) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| now.duration_since(modified).ok())
        .is_some_and(|age| age >= min_age)
}
