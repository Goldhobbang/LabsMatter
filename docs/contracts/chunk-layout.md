# Contract: Chunk Layout

**Version:** 1.0.0
**Owner:** A1 (spec) / A2 (impl)

## Chunk Size

- Default `chunk_size = 32` cells per edge.
- Chunk covers `chunk_size x chunk_size` cells.

## Storage

- Each chunk stores cells in row-major order: index = `ly * chunk_size + lx`
  where `lx, ly` are local coordinates `[0, chunk_size)`.
- Storage may be SoA (Structure of Arrays) internally for cache efficiency;
  logical access is via `Cell`.
- Chunks are stored in a deterministic map keyed by `ChunkCoord` (sorted).

## Chunk Lifecycle

- Chunks are allocated lazily on first access (infinite world) OR eagerly for
  bounded worlds (default). Default world is bounded (512x384).
- Empty chunks may be deallocated to save memory (optional, A2 decides).

## Determinism

- Chunk iteration order: row-major by `(cy, cx)`.
- Cell iteration within chunk: row-major by `(ly, lx)`.
