//! World container.

use crate::material::MaterialRegistry;

#[derive(Clone, Debug)]
pub struct WorldConfig {
    pub width: u32,
    pub height: u32,
    pub chunk_size: u32,
    pub fixed_dt: f64,
}

impl WorldConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            chunk_size: 32,
            fixed_dt: 1.0 / 60.0,
        }
    }

    pub fn chunks_x(&self) -> u32 {
        (self.width + self.chunk_size - 1) / self.chunk_size
    }

    pub fn chunks_y(&self) -> u32 {
        (self.height + self.chunk_size - 1) / self.chunk_size
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

pub const DEFAULT_WORLD_WIDTH: u32 = 512;
pub const DEFAULT_WORLD_HEIGHT: u32 = 384;
pub const DEFAULT_CHUNK_SIZE: u32 = 32;

pub struct World {
    pub config: WorldConfig,
    pub materials: std::sync::Arc<dyn MaterialRegistry>,
}

impl World {
    pub fn new(config: WorldConfig, materials: std::sync::Arc<dyn MaterialRegistry>) -> Self {
        Self { config, materials }
    }
}