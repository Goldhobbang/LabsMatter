//! Chunk layout and constants.

use serde::{Deserialize, Serialize};

use crate::cell::Cell;
use crate::coord::ChunkCoord;
use crate::coord::CHUNK_SIZE;

pub const CHUNK_VOLUME: usize = (CHUNK_SIZE as usize)
    * (CHUNK_SIZE as usize)
    * (CHUNK_SIZE as usize);

/// A chunk of cells.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub coord: ChunkCoord,
    pub cells: [Cell; CHUNK_VOLUME],
    pub dirty: bool,
}

impl Chunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            cells: [Cell::default(); CHUNK_VOLUME],
            dirty: false,
        }
    }
}
