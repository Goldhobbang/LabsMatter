//! World configuration (2D, approved world-grid contract).
use serde::{Deserialize, Serialize};

/// World configuration: bounded 2D grid of cells.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldConfig {
    pub width: u32,
    pub height: u32,
    pub chunk_size: u32,
    pub fixed_dt: f64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 512,
            height: 384,
            chunk_size: 32,
            fixed_dt: 1.0 / 60.0,
        }
    }
}

impl WorldConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Self::default()
        }
    }

    pub fn chunks_x(&self) -> u32 {
        self.width.div_ceil(self.chunk_size.max(1))
    }

    pub fn chunks_y(&self) -> u32 {
        self.height.div_ceil(self.chunk_size.max(1))
    }

    pub fn validate(&self) -> crate::error::Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(crate::error::RealmatterError::InvalidDimensions(
                self.width, self.height,
            ));
        }
        if self.chunk_size == 0 {
            return Err(crate::error::RealmatterError::Internal(
                "chunk_size must be > 0".into(),
            ));
        }
        Ok(())
    }
}
