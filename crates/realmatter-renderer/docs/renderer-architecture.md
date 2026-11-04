# Renderer Architecture (A5) — Design Spec

**Status:** Draft (design-only, per D-004)
**Owner:** A5 (Renderer)
**Dependencies:** realmatter-core (contracts), realmatter-wasm (integration)
**Updated:** 2026-11-04

## Goals
- High-throughput 2D/3D-ready rendering of world grid/particles via WebGPU.
- Deterministic, read-only consumption of simulation render snapshot.
- Chunk-based dirty tracking to minimize redraw cost.
- Graceful fallback to WebGL2 when WebGPU unavailable.
- Offline-first, no network dependencies.
- Low interaction latency; integrate cleanly with UI via adapter interface.

## Non-goals (v1)
- Photorealistic rendering, advanced post-processing.
- Ray tracing, volumetric lighting.
- Direct simulation mutation.

## Architecture overview
- **RenderAdapter** (defined in `web/src/adapters/renderer.js`, mirrors future Rust API): `attach`, `resize`, `render(snapshot, camera, opts)`, `setOverlay`, `dispose`.
- **Snapshot consumer**: Renderer reads `RenderSnapshot { width,height,grid,temp,... }` produced by core (double-buffered, immutable for the frame).
- **Chunk system**: World split into 16×16×16 chunks; renderer tracks dirty chunks per snapshot tick.
- **Camera**: Orthographic (2D) initially; perspective-ready for 3D. Supports pan/zoom/world-to-screen.
- **Pipeline**: WebGPU render passes (clear, grid/particles, overlays, UI/debug). Bind groups for atlas/material colors.

## Data flow
1. Core advances tick → produces snapshot (read-only copy or view).
2. UI loop calls `renderer.render(snapshot, camera, opts)`.
3. Renderer computes visible chunk range from camera.
4. For dirty chunks: upload/update GPU buffers (instance data per particle/cell).
5. Draw calls batched by material/atlas; grid overlay optional.
6. Return timing metrics (render ms, fps) for diagnostics.

## Chunk dirty tracking
- Each chunk has `dirty: bool` + `version: u64`.
- On snapshot change: mark chunks containing modified cells dirty (or all if full reset).
- Renderer caches GPU buffers per chunk; rebuild on dirty.
- Frustum/culling by visible range (AABB).
- Deterministic: chunk order sorted by ChunkCoord.

## WebGPU pipeline design
- **Device/queue**: request adapter/device, handle lost device.
- **Buffers**: vertex buffer (quad), instance buffer (per visible cell: position, color/material, flags).
- **Bind group layout**: camera UBO, material atlas texture/sampler.
- **Shaders (WGSL)**:
  - Vertex: instance transform by cell size/camera.
  - Fragment: sample atlas or flat color; heat overlay tint.
- **Render passes**: opaque cells, overlays (grid/heat).
- **Feature detection**: `navigator.gpu` present; fallback to WebGL2 if absent or init fails.

## WebGL2 fallback strategy
- Detect: try WebGPU first; on failure → create WebGL2 context.
- **Rendering**: instanced drawing (`ANGLE_instanced_arrays`) if available, else per-cell quads (batched).
- **Shaders**: GLSL ES 3.00.
- **Textures**: material color atlas (1D/2D).
- **Parity**: same adapter interface, same snapshot schema, overlays identical.
- **Limitations**: may hit draw call limits on very large visible areas; reduce cell size/zoom or use coarser batching.
- **Telemetry**: log fallback reason to diagnostics.

## Camera & transforms
```text
world(cell) -> screen(px): 
  x = (cell.x + camera.x) * cellSize * zoom
  y = (cell.y + camera.y) * cellSize * zoom
```
- Orthographic projection.
- Zoom range [0.25, 6.0].
- Pan in world/screen coords.
- Cell picking for UI tools.

## Overlays
- **Grid**: line drawing between cells (WebGPU line primitive or thin quads).
- **Heat**: remap temperature to color gradient (blue→red) per cell.
- Toggle via UI; cheap to compute.

## Performance considerations
- **Batching**: draw all cells in chunk with single instanced draw call.
- **Culling**: only visible chunks/cells.
- **Updates**: GPU buffer updates (writeBuffer) only for dirty chunks.
- **Avoid allocations** in hot loop.
- **Target**: render ≤ 8 ms @ 1M particles (per architecture).
- **FPS**: track frame time, expose via diagnostics.

## Error handling & resilience
- **Adapter init failure**: show loading/error overlay with retry; never block UI.
- **Device loss**: attempt re-init once; fallback to WebGL2.
- **Context loss**: restore on `webglcontextlost`/WebGPU lost.
- **Graceful degradation**: reduce quality (no heat overlay, coarser grid) on low perf.

## Integration points
- **Rust crate**: `crates/realmatter-renderer/` will implement against core traits (`World`/snapshot). 
- **WASM bridge**: `realmatter-wasm` exposes snapshot/camera API to JS.
- **UI**: uses `RendererAdapter` only (no internal access).
- **Contracts**: pending A1 WASM boundary contract (REQ-A5-001) for snapshot schema.

## Testing (recommended, A8)
- WebGPU available: instanced rendering, chunk dirty updates, overlays.
- WebGPU unavailable: WebGL2 fallback, same visuals.
- Large world culling: only visible drawn.
- Camera pan/zoom smooth at 60 FPS.
- Device loss recovery.

## TODO (when contracts ready)
- Define `RenderSnapshot` struct in Rust/WASM bindings.
- Implement WebGPU backend.
- Implement WebGL2 fallback.
- Chunk upload cache.
- Material atlas generation from registry.
