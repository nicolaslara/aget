use std::collections::{BTreeMap, BTreeSet};

use crate::error::{AgetError, ErrorCode};
use crate::session::{SessionOrigin, StorageEntry};

#[derive(Debug)]
pub(super) struct ComposedOrigin {
    origin: String,
    pub(super) local_storage: BTreeMap<String, StorageEntry>,
    pub(super) session_storage: BTreeMap<String, StorageEntry>,
    sources: BTreeSet<String>,
}

impl ComposedOrigin {
    pub(super) fn new(origin: &str) -> Self {
        Self {
            origin: origin.to_string(),
            local_storage: BTreeMap::new(),
            session_storage: BTreeMap::new(),
            sources: BTreeSet::new(),
        }
    }

    pub(super) fn insert_source(&mut self, source: String) {
        self.sources.insert(source);
    }

    pub(super) fn into_session_origin(self) -> SessionOrigin {
        let source_session = if self.sources.len() == 1 {
            self.sources.into_iter().next()
        } else {
            None
        };
        SessionOrigin {
            origin: self.origin,
            local_storage: self.local_storage.into_values().collect(),
            session_storage: self.session_storage.into_values().collect(),
            source_session,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct OriginStorage {
    pub(super) local_storage: BTreeMap<String, StorageEntry>,
    pub(super) session_storage: BTreeMap<String, StorageEntry>,
}

pub(super) fn merge_storage_entries(
    origin: &str,
    storage_kind: &str,
    entries_by_name: &mut BTreeMap<String, StorageEntry>,
    entries: &[StorageEntry],
) -> Result<(), AgetError> {
    for entry in entries {
        match entries_by_name.get(&entry.name) {
            Some(existing) if existing.value != entry.value => {
                return Err(AgetError::Stable {
                    code: ErrorCode::SessionConflict,
                    message: format!(
                        "conflicting {storage_kind} key '{}' for origin '{}'",
                        entry.name, origin
                    ),
                });
            }
            Some(_) => {}
            None => {
                entries_by_name.insert(entry.name.clone(), entry.clone());
            }
        }
    }
    Ok(())
}
