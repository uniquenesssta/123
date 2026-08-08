use thiserror::Error;

pub type PortResult<T> = Result<T, PortError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortErrorKind {
    Unavailable,
    NotFound,
    Conflict,
    InvalidState,
    Serialization,
    Infrastructure,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct PortError {
    pub kind: PortErrorKind,
    pub message: String,
}

impl PortError {
    pub fn new(kind: PortErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}
