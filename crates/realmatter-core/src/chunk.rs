//! Chunk layout and constants (2D, approved chunk-layout contract).
//!
//! A chunk covers `chunk_size x chunk_size` cells in row-major order:
//! `index = ly * chunk_size + lx`. Chunks are stored in a deterministic
//! map keyed by [`ChunkCoord`] (sorted iteration order).

use serde::{Deserialize, Serialize};

use crate::cell::Cell;
use crate::coord::ChunkCoord;

/// Number of cells per chunk edge for fixed-size chunk storage.
///
/// NOTE: the runtime chunk edge comes from `WorldConfig::chunk_size`
/// (default 32). This constant is the compile-time chunk volume used for
/// fixed-size chunk arrays; it matches the default config.
pub const CHUNK_EDGE: usize = 32;

/// Number of cells in a default-sized chunk (`CHUNK_EDGE * CHUNK_EDGE`).
pub const CHUNK_VOLUME: usize = CHUNK_EDGE * CHUNK_EDGE;

/// A chunk of cells (default chunk size).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub coord: ChunkCoord,
    pub cells: Vec<Cell>,
    pub dirty: bool,
}

impl Chunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            cells: vec![Cell::default(); CHUNK_VOLUME],
            dirty: false,
        }
    }

    /// Row-major cell index for local coords in `[0, CHUNK_EDGE)`.
    #[inline]
    pub fn index(lx: usize, ly: usize) -> usize {
        ly * CHUNK_EDGE + lx
    }

    #[inline]
    pub fn get(&self, lx: usize, ly: usize) -> &Cell {
        &self.cells[Self::index(lx, ly)]
    }

    #[inline]
    pub fn get_mut(&mut self, lx: usize, ly: usize) -> &mut Cell {
        &mut self.cells[Self::index(lx, ly)]
    }

    /// Mark the chunk dirty (needs re-render / re-serialize).
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear the dirty flag after the chunk has been consumed.
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
