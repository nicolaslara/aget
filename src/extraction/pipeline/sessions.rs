use crate::error::AgetError;
use crate::session::Session;

use crate::extraction::{io_aget_error, ExtractionSessionStore};

pub(in crate::extraction) fn load_selected_sessions(
    store: &impl ExtractionSessionStore,
    session_names: &[String],
) -> Result<Vec<Session>, AgetError> {
    session_names
        .iter()
        .map(|name| store.load(name).map_err(io_aget_error))
        .collect()
}
