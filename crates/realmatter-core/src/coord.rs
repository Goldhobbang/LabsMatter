//! Coordinate types.

use serde::{Deserialize, Serialize};

/// Cell coordinate (world-absolute, in cells).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CellCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Chunk coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub const CHUNK_SIZE: i32 = 16;

impl CellCoord {
    pub fn to_chunk(self) -> ChunkCoord {
        ChunkCoord {
            x: self.x.div_euclid(CHUNK_SIZE),
            y: self.y.div_euclid(CHUNK_SIZE),
            z: self.z.div_euclid(CHUNK_SIZE),
        }
    }
    pub fn to_local(self) -> (u8, u8, u8) {
        (
            self.x.rem_euclid(CHUNK_SIZE) as u8,
            self.y.rem_euclid(CHUNK_SIZE) as u8,
            self.z.rem_euclid(CHUNK_SIZE) as u8,
        )
    }
}

impl ChunkCoord {
    pub fn origin_cell(self) -> CellCoord {
        CellCoord {
            x: self.x * CHUNK_SIZE,
            y: self.y * CHUNK_SIZE,
            z: self.z * CHUNK_SIZE,
        }
    }
}
