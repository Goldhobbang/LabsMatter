import pathlib

content = '''//! World grid, chunk coordinates, cell coordinates, and configuration.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::chunk::Chunk;
use crate::coord::{CellCoord, ChunkCoord};
use crate::error::Result;
use crate::material::MaterialRegistry;

/// World configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldConfig {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub chunk_size: u32,
    pub fixed_dt: f64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            depth: 64,
            chunk_size: 8,
            fixed_dt: 1.0 / 60.0,
        }
    }
}

impl WorldConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            depth: 1,
            chunk_size: 8,
            fixed_dt: 1.0 / 60.0,
        }
    }

    pub fn chunks_x(&self) -> u32 {
        (self.width + self.chunk_size - 1) / self.chunk_size
    }

    pub fn chunks_y(&self) -> u32 {
        (self.height + self.chunk_size - 1) / self.chunk_size
    }

    pub fn chunks_z(&self) -> u32 {
        (self.depth + self.chunk_size - 1) / self.chunk_size
    }

    pub fn validate(&self) -> Result<()> {
        if self.width == 0 || self.height == 0 || self.depth == 0 {
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

/// The world: a grid of chunks.
#[derive(Clone, Debug)]
pub struct World {
    pub config: WorldConfig,
    pub tick_count: u64,
    pub time_accumulator: f64,
    pub chunks: BTreeMap<ChunkCoord, Chunk>,
    pub materials: MaterialRegistry,
}

impl World {
    pub fn new(config: WorldConfig, materials: MaterialRegistry) -> Self {
        let cs = config.chunk_size as i32;
        let cx = (config.width as i32 + cs - 1) / cs;
        let cy = (config.height as i32 + cs - 1) / cs;
        let cz = (config.depth as i32 + cs - 1) / cs;
        let mut chunks = BTreeMap::new();
        for z in 0..cz {
            for y in 0..cy {
                for x in 0..cx {
                    let coord = ChunkCoord::new(x, y, z);
                    chunks.insert(coord, Chunk::new(coord));
                }
            }
        }
        Self {
            config,
            tick_count: 0,
            time_accumulator: 0.0,
            chunks,
            materials,
        }
    }

    pub fn chunks(&self) -> &BTreeMap<ChunkCoord, Chunk> {
        &self.chunks
    }

    pub fn chunks_mut(&mut self) -> &mut BTreeMap<ChunkCoord, Chunk> {
        &mut self.chunks
    }

    fn chunk_coord_of(&self, coord: CellCoord) -> ChunkCoord {
        let cs = self.config.chunk_size as i32;
        ChunkCoord::new(
            coord.x.div_euclid(cs),
            coord.y.div_euclid(cs),
            coord.z.div_euclid(cs),
        )
    }

    fn local_index(&self, coord: CellCoord) -> usize {
        let cs = self.config.chunk_size as usize;
        let lx = coord.x.rem_euclid(self.config.chunk_size as i32) as usize;
        let ly = coord.y.rem_euclid(self.config.chunk_size as i32) as usize;
        let lz = coord.z.rem_euclid(self.config.chunk_size as i32) as usize;
        (lz * cs + ly) * cs + lx
    }

    pub fn cell(&self, coord: CellCoord) -> Option<&crate::cell::Cell> {
        let cc = self.chunk_coord_of(coord);
        let chunk = self.chunks.get(&cc)?;
        Some(&chunk.cells[self.local_index(coord)])
    }

    pub fn cell_mut(&mut self, coord: CellCoord) -> &mut crate::cell::Cell {
        let cc = self.chunk_coord_of(coord);
        let idx = self.local_index(coord);
        &mut self.chunks.get_mut(&cc).expect("chunk missing").cells[idx]
    }

    pub fn get_or_create_chunk(&mut self, coord: ChunkCoord) -> &mut Chunk {
        if !self.chunks.contains_key(&coord) {
            self.chunks.insert(coord, Chunk::new(coord));
        }
        self.chunks.get_mut(&coord).unwrap()
    }
}
'''
pathlib.Path('crates/realmatter-core/src/world.rs').write_text(content, encoding='utf-8')
print("world.rs written")
