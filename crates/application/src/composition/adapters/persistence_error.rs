use super::super::port_registry::PersistenceError;
use crate::ports::{PortError, PortErrorKind};

pub(in crate::composition) fn map_persistence_error(error: PersistenceError) -> PortError {
    let kind = match &error {
        PersistenceError::Serialization(_) => PortErrorKind::Serialization,
        PersistenceError::InvalidState(_) => PortErrorKind::InvalidState,
        PersistenceError::RouteNotFound => PortErrorKind::NotFound,
        PersistenceError::Sqlx(_) | PersistenceError::Migration(_) => PortErrorKind::Infrastructure,
    };
    PortError::new(kind, error.to_string())
}
