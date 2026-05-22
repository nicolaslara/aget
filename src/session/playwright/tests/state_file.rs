use super::super::{compose_playwright_state, TempStateFile};

#[test]
fn temp_state_file_is_removed_on_drop() {
    let temp = tempfile::tempdir().unwrap();
    let state = compose_playwright_state(&[]).unwrap();
    let temp_state = TempStateFile::write(temp.path(), &state).unwrap();
    let path = temp_state.path().to_path_buf();

    assert!(path.exists());
    drop(temp_state);
    assert!(!path.exists());
}

#[cfg(unix)]
#[test]
fn temp_state_file_uses_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let state = compose_playwright_state(&[]).unwrap();
    let temp_state = TempStateFile::write(temp.path(), &state).unwrap();
    let mode = std::fs::metadata(temp_state.path())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(mode, 0o600);
}
