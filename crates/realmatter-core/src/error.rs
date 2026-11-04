//! Error types for REALMATTER.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RealmatterError {
    #[error("invalid world dimensions: {0}x{1}")]
    InvalidDimensions(u32, u32),

    #[error("chunk out of bounds: ({0}, {1})")]
    ChunkOutOfBounds(i32, i32),

    #[error("cell out of bounds: ({0}, {1})")]
    CellOutOfBounds(i32, i32),

    #[error("unknown material id: {0}")]
    UnknownMaterial(u16),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    #[error("io error: {0}")]
    Io(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, RealmatterError>;