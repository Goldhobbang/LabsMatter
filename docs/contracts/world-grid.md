# Contract: World Grid

**Version:** 0.1.0
**Owner:** A1

## ChunkCoord

```rust
pub struct ChunkCoord { pub x: i32, pub y: i32 }
```

## WorldConfig

```rust
pub struct WorldConfig {
    pub width: u32,        // cells
    pub height: u32,       // cells
    pub chunk_size: u32,   // cells per chunk edge (default 32)
    pub fixed_dt: f64,     // seconds
}
```

Methods: `chunks_x()`, `chunks_y()`, `validate()`.

Defaults: width=512, height=384, chunk_size=32, fixed_dt=1/60.

## Validation

`validate()` returns `Err(InvalidDimensions)` if width or height is 0; `Err(Internal)` if chunk_size is 0.