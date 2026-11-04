//! 2D coordinate types (approved 2D grid model, ADR-0003).
//!
//! Canonical definitions of [`CellCoord`] and [`ChunkCoord`]. There is exactly
//! one definition of each (D-002 item 4 / D-003 item 3).

use serde::{Deserialize, Serialize};

/// Default chunk edge length in cells (see `WorldConfig::chunk_size`).
pub const CHUNK_SIZE: i32 = 32;

/// Cell coordinate (world-absolute, in cells). 2D: x right, y up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CellCoord {
    pub x: i32,
    pub y: i32,
}

impl CellCoord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Chunk containing this cell for the given chunk size.
    pub fn to_chunk(self, chunk_size: i32) -> ChunkCoord {
        ChunkCoord {
            x: self.x.div_euclid(chunk_size),
            y: self.y.div_euclid(chunk_size),
        }
    }

    /// Local position within its chunk, each in `[0, chunk_size)`.
    pub fn to_local(self, chunk_size: i32) -> (i32, i32) {
        (
            self.x.rem_euclid(chunk_size),
            self.y.rem_euclid(chunk_size),
        )
    }
}

/// Chunk coordinate (in chunks). 2D.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

impl ChunkCoord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// World-absolute cell coordinate of this chunk's origin (min corner).
    pub fn origin_cell(self, chunk_size: i32) -> CellCoord {
        CellCoord {
            x: self.x * chunk_size,
            y: self.y * chunk_size,
        }
    }
}
