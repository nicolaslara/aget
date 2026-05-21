use std::io;
use std::path::{Path, PathBuf};

use crate::extraction::ExtractionSessionStore;
use crate::session::{Session, SessionStore};

pub trait SessionStoreBackend {
    // The default implementation is filesystem-backed and local-first, but `Aget`
    // only needs this persistence contract.
    fn home(&self) -> &Path;
    fn list(&self) -> io::Result<Vec<String>>;
    fn load(&self, name: &str) -> io::Result<Session>;
    fn save(&self, session: &Session) -> io::Result<()>;
    fn delete(&self, name: &str) -> io::Result<bool>;
    fn exists(&self, name: &str) -> io::Result<bool>;
}

#[derive(Clone)]
pub struct FilesystemSessionStoreBackend {
    home: PathBuf,
}

impl FilesystemSessionStoreBackend {
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }

    fn store(&self) -> io::Result<SessionStore> {
        SessionStore::new(&self.home)
    }
}

impl SessionStoreBackend for FilesystemSessionStoreBackend {
    fn home(&self) -> &Path {
        &self.home
    }

    fn list(&self) -> io::Result<Vec<String>> {
        self.store()?.list()
    }

    fn load(&self, name: &str) -> io::Result<Session> {
        self.store()?.load(name)
    }

    fn save(&self, session: &Session) -> io::Result<()> {
        self.store()?.save(session)
    }

    fn delete(&self, name: &str) -> io::Result<bool> {
        self.store()?.delete(name)
    }

    fn exists(&self, name: &str) -> io::Result<bool> {
        self.store()?.exists(name)
    }
}

impl<T> ExtractionSessionStore for T
where
    T: SessionStoreBackend,
{
    fn home(&self) -> &Path {
        SessionStoreBackend::home(self)
    }

    fn load(&self, name: &str) -> io::Result<Session> {
        SessionStoreBackend::load(self, name)
    }
}
