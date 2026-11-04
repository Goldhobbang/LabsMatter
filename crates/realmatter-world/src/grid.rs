//! 2D world grid: bounded cell storage, chunk activation, dirty tracking.
//!
//! Storage is a flat row-major `Vec<Cell>` (`index = y * width + x`) for the
//! bounded default world. Chunk activation and dirty-region tracking are
//! derived from cell coordinates via [`WorldConfig::chunk_size`], so the
//! renderer and serializer can consume dirty chunks without a separate chunk
//! map. Deterministic iteration order is row-major `(y, x)` everywhere.

use std::collections::BTreeSet;
use std::sync::Arc;

use realmatter_core::cell::{cell_flags, Cell};
use realmatter_core::coord::{CellCoord, ChunkCoord};
use realmatter_core::error::{RealmatterError, Result};
use realmatter_core::material::{MaterialId, MaterialRegistry};
use realmatter_core::world_config::WorldConfig;

/// Live 2D simulation world (A2).
pub struct World {
    config: WorldConfig,
    cells: Vec<Cell>,
    materials: Arc<dyn MaterialRegistry>,
    tick_count: u64,
    time_accumulator: f64,
    /// Chunks touched since the last [`World::take_dirty_chunks`].
    dirty_chunks: BTreeSet<ChunkCoord>,
}

impl World {
    /// Allocate a validated world filled with empty cells.
    pub fn new(config: WorldConfig, materials: Arc<dyn MaterialRegistry>) -> Result<Self> {
        config.validate()?;
        let count = config.width as usize * config.height as usize;
        Ok(Self {
            config,
            cells: vec![Cell::default(); count],
            materials,
            tick_count: 0,
            time_accumulator: 0.0,
            dirty_chunks: BTreeSet::new(),
        })
    }

    #[inline]
    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    #[inline]
    pub fn materials(&self) -> &Arc<dyn MaterialRegistry> {
        &self.materials
    }

    #[inline]
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    #[inline]
    pub fn time_accumulator(&self) -> f64 {
        self.time_accumulator
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.config.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.config.height
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Flat row-major index for `(x, y)`, or `None` if out of bounds.
    #[inline]
    pub fn index_of(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.config.width as i32 || y >= self.config.height as i32 {
            return None;
        }
        Some(y as usize * self.config.width as usize + x as usize)
    }

    /// Chunk containing `(x, y)`, or `None` if out of bounds.
    #[inline]
    pub fn chunk_of(&self, x: i32, y: i32) -> Option<ChunkCoord> {
        if x < 0 || y < 0 || x >= self.config.width as i32 || y >= self.config.height as i32 {
            return None;
        }
        let cs = self.config.chunk_size as i32;
        Some(ChunkCoord::new(x.div_euclid(cs), y.div_euclid(cs)))
    }

    /// Chunk containing `coord`, or `None` if out of bounds.
    #[inline]
    pub fn chunk_of_coord(&self, coord: CellCoord) -> Option<ChunkCoord> {
        self.chunk_of(coord.x, coord.y)
    }

    #[inline]
    pub fn cell(&self, x: i32, y: i32) -> Option<&Cell> {
        self.index_of(x, y).map(|i| &self.cells[i])
    }

    #[inline]
    pub fn cell_at(&self, coord: CellCoord) -> Option<&Cell> {
        self.cell(coord.x, coord.y)
    }

    #[inline]
    pub fn cell_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell> {
        let idx = self.index_of(x, y)?;
        let cs = self.config.chunk_size as i32;
        let cc = ChunkCoord::new(x.div_euclid(cs), y.div_euclid(cs));
        self.dirty_chunks.insert(cc);
        Some(&mut self.cells[idx])
    }

    /// Read a cell; out-of-bounds returns `CellOutOfBounds`.
    pub fn get(&self, coord: CellCoord) -> Result<&Cell> {
        self.cell_at(coord)
            .ok_or(RealmatterError::CellOutOfBounds(coord.x, coord.y))
    }

    /// Write a cell and mark its chunk dirty; out-of-bounds is an error.
    pub fn set(&mut self, coord: CellCoord, cell: Cell) -> Result<()> {
        let slot = self
            .cell_mut(coord.x, coord.y)
            .ok_or(RealmatterError::CellOutOfBounds(coord.x, coord.y))?;
        *slot = cell;
        Ok(())
    }

    /// Raw row-major cell slice (y outer, x inner) for module phases.
    #[inline]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Mutable cell slice. Callers must invoke [`World::mark_dirty`] for
    /// regions they modify so dirty tracking stays accurate.
    #[inline]
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }

    /// Mark the chunk containing `(x, y)` dirty (no-op if out of bounds).
    pub fn mark_dirty(&mut self, x: i32, y: i32) {
        if let Some(cc) = self.chunk_of(x, y) {
            self.dirty_chunks.insert(cc);
        }
    }

    /// Mark every chunk intersecting the inclusive rect dirty.
    pub fn mark_rect_dirty(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let cs = self.config.chunk_size as i32;
        let (xa, xb) = (x0.min(x1).max(0), x1.max(x0).min(self.config.width as i32 - 1));
        let (ya, yb) = (y0.min(y1).max(0), y1.max(y0).min(self.config.height as i32 - 1));
        if xa > xb || ya > yb {
            return;
        }
        for cy in (ya.div_euclid(cs))..=(yb.div_euclid(cs)) {
            for cx in (xa.div_euclid(cs))..=(xb.div_euclid(cs)) {
                self.dirty_chunks.insert(ChunkCoord::new(cx, cy));
            }
        }
    }

    /// Drain the dirty-chunk set (sorted). Renderer/serializer consume this.
    pub fn take_dirty_chunks(&mut self) -> Vec<ChunkCoord> {
        std::mem::take(&mut self.dirty_chunks).into_iter().collect()
    }

    /// Peek at dirty chunks without draining.
    pub fn dirty_chunks(&self) -> impl Iterator<Item = &ChunkCoord> {
        self.dirty_chunks.iter()
    }

    /// Clear `UPDATED` flags, age lifetimes, remove dead cells.
    /// Must run once per tick (finalize phase) in row-major order.
    pub fn finalize_tick(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.set_flag(cell_flags::UPDATED, false);
            if cell.lifetime > 0.0 {
                cell.lifetime -= self.config.fixed_dt as f32;
                if cell.lifetime <= 0.0 {
                    *cell = Cell::default();
                }
            }
        }
        self.tick_count += 1;
    }

    /// Advance the time accumulator by `elapsed`, returning the number of
    /// fixed steps to run (capped at `MAX_SUBSTEPS = 4`).
    pub fn accumulate(&mut self, elapsed: f64) -> u32 {
        self.time_accumulator += elapsed;
        let dt = self.config.fixed_dt;
        let mut steps = 0u32;
        while self.time_accumulator >= dt && steps < 4 {
            self.time_accumulator -= dt;
            steps += 1;
        }
        if steps == 4 && self.time_accumulator >= dt {
            // Spiral-of-death guard: drop leftover time.
            self.time_accumulator = 0.0;
        }
        steps
    }

    /// Material lookup helper for movement/physics phases.
    pub fn material_props(&self, id: MaterialId) -> Option<&realmatter_core::material::MaterialProps> {
        self.materials.get(id)
    }

    /// Restore the tick counter (used by persistence load; format v1 header
    /// already stores `tick_count`, this applies it without touching cells).
    pub fn set_tick_count(&mut self, tick: u64) {
        self.tick_count = tick;
    }

    /// Restore the time accumulator (used by persistence load).
    pub fn set_time_accumulator(&mut self, t: f64) {
        self.time_accumulator = if t.is_finite() && t >= 0.0 { t } else { 0.0 };
    }
}
