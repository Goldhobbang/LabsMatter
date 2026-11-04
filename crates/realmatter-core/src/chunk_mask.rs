//! Active-chunk mask for simulation phase skipping.
//!
//! An [`ActiveMask`] marks which chunks of the world need processing in a
//! given tick: every chunk containing at least one non-empty cell, plus its
//! 8-neighbors (cross-boundary movement, diffusion, heat transfer, and fire
//! spread can reach adjacent chunks). The mask is rebuilt every tick from
//! the live cell slice, so regions can never get permanently stuck inactive:
//! any cell written into an inactive chunk shows up in the next rebuild.
//!
//! Determinism: iteration order stays row-major everywhere; the mask only
//! *skips* cells whose processing is a proven no-op (empty cells in
//! all-empty neighborhoods). Skipped work must not change results — callers
//! must only skip source cells that are empty, and must always allow writes
//! into inactive chunks (e.g. gas diffusing into an empty chunk).

/// Chunk activity bitmask for one tick.
#[derive(Clone, Debug, Default)]
pub struct ActiveMask {
    words: Vec<u64>,
    chunks_x: u32,
    chunks_y: u32,
    chunk_size: u32,
}

impl ActiveMask {
    /// All chunks active (no skipping). Used when no mask was computed.
    pub fn all(chunks_x: u32, chunks_y: u32, chunk_size: u32) -> Self {
        let n = (chunks_x as usize) * (chunks_y as usize);
        Self {
            words: vec![u64::MAX; n.div_ceil(64)],
            chunks_x,
            chunks_y,
            chunk_size: chunk_size.max(1),
        }
    }

    /// Empty mask (nothing active).
    pub fn none(chunks_x: u32, chunks_y: u32, chunk_size: u32) -> Self {
        let n = (chunks_x as usize) * (chunks_y as usize);
        Self {
            words: vec![0; n.div_ceil(64)],
            chunks_x,
            chunks_y,
            chunk_size: chunk_size.max(1),
        }
    }

    /// Build from a row-major cell-material slice: chunks containing a
    /// non-`empty_id` cell plus 8-neighbors are marked active.
    pub fn build(
        is_empty: impl Fn(usize) -> bool,
        width: u32,
        height: u32,
        chunk_size: u32,
    ) -> Self {
        let cs = chunk_size.max(1);
        let cx = width.div_ceil(cs);
        let cy = height.div_ceil(cs);
        let mut mask = Self::none(cx, cy, cs);
        let w = width as usize;
        let h = height as usize;
        for y in 0..h {
            for x in 0..w {
                if is_empty(y * w + x) {
                    continue;
                }
                let ccx = (x as u32) / cs;
                let ccy = (y as u32) / cs;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = ccx as i32 + dx;
                        let ny = ccy as i32 + dy;
                        if nx >= 0 && ny >= 0 && (nx as u32) < cx && (ny as u32) < cy {
                            mask.set(nx as u32, ny as u32);
                        }
                    }
                }
            }
        }
        mask
    }

    fn set(&mut self, cx: u32, cy: u32) {
        let idx = (cy * self.chunks_x + cx) as usize;
        self.words[idx / 64] |= 1u64 << (idx % 64);
    }

    /// Build from a live cell slice. Faster than [`ActiveMask::build`]: no
    /// per-cell closure call, and chunk coordinates come from a linear
    /// counter (bit shift when `chunk_size` is a power of two, which covers
    /// the default 32).
    pub fn build_from_cells(
        cells: &[crate::cell::Cell],
        width: u32,
        height: u32,
        chunk_size: u32,
    ) -> Self {
        let cs = chunk_size.max(1);
        let cx = width.div_ceil(cs);
        let cy = height.div_ceil(cs);
        let mut mask = Self::none(cx, cy, cs);
        let shift = if cs.is_power_of_two() {
            Some(cs.trailing_zeros())
        } else {
            None
        };
        let w = width as usize;
        let mut x = 0usize;
        let mut y = 0usize;
        for cell in cells.iter() {
            if !cell.is_empty() {
                let (ccx, ccy) = match shift {
                    Some(s) => ((x >> s) as u32, (y >> s) as u32),
                    None => ((x as u32) / cs, (y as u32) / cs),
                };
                let x0 = ccx.saturating_sub(1);
                let y0 = ccy.saturating_sub(1);
                let x1 = (ccx + 1).min(cx.saturating_sub(1));
                let y1 = (ccy + 1).min(cy.saturating_sub(1));
                for ny in y0..=y1 {
                    for nx in x0..=x1 {
                        mask.set(nx, ny);
                    }
                }
            }
            x += 1;
            if x >= w {
                x = 0;
                y += 1;
            }
        }
        mask
    }

    /// Test whether chunk `(cx, cy)` is active. Out-of-range reads false.
    #[inline]
    pub fn is_active(&self, cx: u32, cy: u32) -> bool {
        if cx >= self.chunks_x || cy >= self.chunks_y {
            return false;
        }
        let idx = (cy * self.chunks_x + cx) as usize;
        (self.words[idx / 64] >> (idx % 64)) & 1 == 1
    }

    /// Test whether the cell `(x, y)` is in an active chunk.
    #[inline]
    pub fn cell_active(&self, x: usize, y: usize) -> bool {
        let cs = self.chunk_size as usize;
        self.is_active((x / cs) as u32, (y / cs) as u32)
    }

    /// Fraction of chunks marked active (for diagnostics).
    pub fn active_fraction(&self) -> f32 {
        let total = (self.chunks_x as usize) * (self.chunks_y as usize);
        if total == 0 {
            return 0.0;
        }
        let bits: u32 = self.words.iter().map(|w| w.count_ones()).sum();
        bits as f32 / total as f32
    }
}
