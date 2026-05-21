use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::session::Session;

mod cleanup;
mod permissions;
#[cfg(test)]
mod tests;

use cleanup::sweep_orphaned_tmp;
use permissions::{create_private_dir, create_private_file};

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
        sweep_orphaned_tmp(&self.home.join("tmp"), Duration::from_secs(60 * 60))?;
        Ok(())
    }

    pub fn save(&self, session: &Session) -> io::Result<()> {
        let path = self.session_path(&session.name)?;
        let mut file = create_private_file(&path)?;
        serde_json::to_writer_pretty(&mut file, session).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn load(&self, name: &str) -> io::Result<Session> {
        let file = File::open(self.session_path(name)?)?;
        serde_json::from_reader(file).map_err(io::Error::other)
    }

    pub fn exists(&self, name: &str) -> io::Result<bool> {
        Ok(self.session_path(name)?.exists())
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
        let path = self.session_path(name)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn session_path(&self, name: &str) -> io::Result<PathBuf> {
        validate_session_name(name)?;
        Ok(self.sessions_dir().join(format!("{name}.json")))
    }
}

fn validate_session_name(name: &str) -> io::Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid session name '{name}'"),
        ));
    }
    Ok(())
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
