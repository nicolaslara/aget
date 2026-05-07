use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::session::Session;

#[derive(Debug, Clone)]
pub struct SessionStore {
    home: PathBuf,
}

impl SessionStore {
    pub fn from_env() -> io::Result<Self> {
        let home = match env::var_os("AGET_HOME") {
            Some(home) => PathBuf::from(home),
            None => default_home()?,
        };

        let store = Self { home };
        store.ensure_layout()?;
        Ok(store)
    }

    pub fn new(home: impl Into<PathBuf>) -> io::Result<Self> {
        let store = Self { home: home.into() };
        store.ensure_layout()?;
        Ok(store)
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.home.join("sessions")
    }

    pub fn ensure_layout(&self) -> io::Result<()> {
        create_private_dir(&self.home)?;
        for dir in ["sessions", "runs", "cache", "tmp"] {
            create_private_dir(&self.home.join(dir))?;
        }
        Ok(())
    }

    pub fn save(&self, session: &Session) -> io::Result<()> {
        let path = self.session_path(&session.name);
        let mut file = create_private_file(&path)?;
        serde_json::to_writer_pretty(&mut file, session).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn load(&self, name: &str) -> io::Result<Session> {
        let file = File::open(self.session_path(name))?;
        serde_json::from_reader(file).map_err(io::Error::other)
    }

    pub fn list(&self) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(self.sessions_dir())? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                    names.push(stem.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    pub fn delete(&self, name: &str) -> io::Result<bool> {
        let path = self.session_path(name);
        match fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn session_path(&self, name: &str) -> PathBuf {
        self.sessions_dir().join(format!("{name}.json"))
    }
}

fn default_home() -> io::Result<PathBuf> {
    let home = env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set and AGET_HOME was not provided",
        )
    })?;
    Ok(PathBuf::from(home).join(".aget"))
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    set_private_dir_permissions(path)
}

fn create_private_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    set_private_file_mode(&mut options);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
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
        let session_mode = fs::metadata(store.session_path("demo"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;

        assert_eq!(home_mode, 0o700);
        assert_eq!(sessions_mode, 0o700);
        assert_eq!(session_mode, 0o600);
    }
}
