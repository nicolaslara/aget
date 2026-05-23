use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use aget::aget::SessionStoreBackend;
use aget::Session;

#[derive(Clone)]
pub(crate) struct MemorySessionStore {
    home: PathBuf,
    sessions: Rc<RefCell<BTreeMap<String, Session>>>,
}

impl MemorySessionStore {
    pub(crate) fn new(home: impl Into<PathBuf>) -> Self {
        Self {
            home: home.into(),
            sessions: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub(crate) fn load(&self, name: &str) -> io::Result<Session> {
        self.sessions
            .borrow()
            .get(name)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "session not found"))
    }

    pub(crate) fn save(&self, session: &Session) -> io::Result<()> {
        self.sessions
            .borrow_mut()
            .insert(session.name.clone(), session.clone());
        Ok(())
    }
}

impl SessionStoreBackend for MemorySessionStore {
    fn home(&self) -> &Path {
        &self.home
    }

    fn list(&self) -> io::Result<Vec<String>> {
        Ok(self.sessions.borrow().keys().cloned().collect())
    }

    fn load(&self, name: &str) -> io::Result<Session> {
        self.load(name)
    }

    fn save(&self, session: &Session) -> io::Result<()> {
        self.save(session)
    }

    fn delete(&self, name: &str) -> io::Result<bool> {
        Ok(self.sessions.borrow_mut().remove(name).is_some())
    }

    fn exists(&self, name: &str) -> io::Result<bool> {
        Ok(self.sessions.borrow().contains_key(name))
    }
}
