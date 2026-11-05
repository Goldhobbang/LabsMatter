//! Save format definition and versioning for REALMATTER worlds.
//!
//! Binary layout (little-endian):
//!
//! ```text
//! [magic: 4 bytes "RMT\0"]
//! [format_version: u32]
//! [header_len: u32]
//! [header: bincode(WorldHeader)]
//! [material_count: u32]
//! [materials: bincode Vec<Material>]
//! [chunk_count: u32]
//! for each chunk (sorted by ChunkCoord):
//!   [coord_len: u32][coord: bincode ChunkCoord]
//!   [payload_len: u32][compressed_flag: u8][payload: cells (optionally flate2)]
//!   [dirty: u8]
//! [recovery_len: u32][recovery: bincode RecoveryMetadata]
//! [checksum: u32 (CRC32)]
//! ```

use realmatter_core::cell::Cell;
use realmatter_core::coord::ChunkCoord;
use realmatter_core::material::MaterialProps;
use realmatter_core::world_config::WorldConfig;
use realmatter_world::World;
use serde::{Deserialize, Serialize};

pub const MAGIC: [u8; 4] = *b"RMT\0";
/// Version 2 adds the optional decoration layer (`SaveData::deco`).
pub const FORMAT_VERSION: u32 = 2;
pub const MIN_SUPPORTED_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldHeader {
    pub config: WorldConfig,
    pub tick_count: u64,
    pub time_accumulator: f64,
    pub compressed_chunks: bool,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RecoveryMetadata {
    pub sequence: u64,
    pub saved_at_unix: u64,
    pub chunk_count: u32,
    pub particle_count: u64,
    pub incremental: bool,
    pub parent_sequence: u64,
}

#[derive(Clone, Debug)]
pub struct SaveData {
    pub header: WorldHeader,
    pub materials: Vec<MaterialProps>,
    pub chunks: Vec<(ChunkCoord, ChunkSnapshot)>,
    pub recovery: RecoveryMetadata,
    /// Decoration layer, one ARGB colour per cell (0 = none). Absent in
    /// version 1 saves, which decode with an empty layer.
    pub deco: Option<Vec<u32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkSnapshot {
    pub coord: ChunkCoord,
    pub cells: Vec<Cell>,
    pub dirty: bool,
}

impl SaveData {
    pub fn from_world(
        world: &mut World,
        label: Option<String>,
        sequence: u64,
        incremental: bool,
        parent: u64,
    ) -> Self {
        let dirty = world.take_dirty_chunks();
        let mut chunks: Vec<(ChunkCoord, ChunkSnapshot)> = Vec::new();
        let cs = world.config().chunk_size as i32;
        let w = world.width() as i32;
        let h = world.height() as i32;
        for cy in 0..(world.config().chunks_y() as i32) {
            for cx in 0..(world.config().chunks_x() as i32) {
                let cc = ChunkCoord::new(cx, cy);
                let mut cells: Vec<Cell> = Vec::with_capacity((cs as usize) * (cs as usize));
                for ly in 0..cs {
                    for lx in 0..cs {
                        let gx = cx * cs + lx;
                        let gy = cy * cs + ly;
                        if gx >= w || gy >= h {
                            cells.push(Cell::default());
                        } else if let Some(c) = world.cell(gx, gy) {
                            cells.push(*c);
                        } else {
                            cells.push(Cell::default());
                        }
                    }
                }
                chunks.push((
                    cc,
                    ChunkSnapshot {
                        coord: cc,
                        cells,
                        dirty: dirty.contains(&cc),
                    },
                ));
            }
        }
        chunks.sort_by_key(|(c, _)| *c);

        let particle_count: u64 = world.cells().iter().filter(|cell| !cell.is_empty()).count() as u64;

        let materials: Vec<MaterialProps> = Vec::new();

        Self {
            header: WorldHeader {
                config: world.config().clone(),
                tick_count: world.tick_count(),
                time_accumulator: world.time_accumulator(),
                compressed_chunks: true,
                label,
            },
            materials,
            chunks,
            deco: None,
            recovery: RecoveryMetadata {
                sequence,
                saved_at_unix: 0,
                chunk_count: 0,
                particle_count,
                incremental,
                parent_sequence: parent,
            },
        }
    }
}

pub fn encode(save: &SaveData) -> Result<Vec<u8>, String> {
    use bincode::Options;
    let opts = bincode::DefaultOptions::new()
        .with_little_endian()
        .with_fixint_encoding();

    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&FORMAT_VERSION.to_le_bytes());

    let header_bytes = opts
        .serialize(&save.header)
        .map_err(|e| format!("header serialize: {e}"))?;
    buf.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&header_bytes);

    buf.extend_from_slice(&(save.materials.len() as u32).to_le_bytes());
    let mat_bytes = opts
        .serialize(&save.materials)
        .map_err(|e| format!("materials serialize: {e}"))?;
    buf.extend_from_slice(&(mat_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&mat_bytes);

    let mut recovery = save.recovery.clone();
    recovery.chunk_count = save.chunks.len() as u32;
    buf.extend_from_slice(&(save.chunks.len() as u32).to_le_bytes());

    for (coord, chunk) in &save.chunks {
        let coord_bytes = opts
            .serialize(coord)
            .map_err(|e| format!("coord serialize: {e}"))?;
        buf.extend_from_slice(&(coord_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(&coord_bytes);

        let cell_bytes = opts
            .serialize(&chunk.cells)
            .map_err(|e| format!("cells serialize: {e}"))?;
        let payload = if save.header.compressed_chunks {
            compress_bytes(&cell_bytes)?
        } else {
            cell_bytes
        };
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.push(if save.header.compressed_chunks { 1 } else { 0 });
        buf.extend_from_slice(&payload);
        buf.push(if chunk.dirty { 1 } else { 0 });
    }

    let rec_bytes = opts
        .serialize(&recovery)
        .map_err(|e| format!("recovery serialize: {e}"))?;
    buf.extend_from_slice(&(rec_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&rec_bytes);

    // Decoration layer (version 2+). Written only when non-empty so v1-style
    // worlds stay compact.
    let deco = save.deco.clone().unwrap_or_default();
    let has_deco = deco.iter().any(|c| *c != 0);
    if has_deco {
        let deco_bytes = opts
            .serialize(&deco)
            .map_err(|e| format!("deco serialize: {e}"))?;
        let payload = compress_bytes(&deco_bytes)?;
        buf.push(1);
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&payload);
    } else {
        buf.push(0);
    }

    let crc = crc32(&buf);
    buf.extend_from_slice(&crc.to_le_bytes());

    Ok(buf)
}

pub fn decode(bytes: &[u8]) -> Result<SaveData, String> {
    use bincode::Options;
    let opts = bincode::DefaultOptions::new()
        .with_little_endian()
        .with_fixint_encoding();

    if bytes.len() < 8 {
        return Err("blob too short".into());
    }
    if bytes[0..4] != MAGIC {
        return Err("bad magic: not a REALMATTER save".into());
    }
    let mut pos = 4usize;
    let version = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
    pos += 4;
    if version < MIN_SUPPORTED_VERSION || version > FORMAT_VERSION {
        return Err(format!(
            "unsupported format version {version} (supported {MIN_SUPPORTED_VERSION}..={FORMAT_VERSION})"
        ));
    }

    let data = migrate(bytes, version).map_err(|e| e.to_string())?;
    if data[0..4] != MAGIC {
        return Err("bad magic after migration".into());
    }
    let mut pos = 8;

    if pos + 4 > data.len() {
        return Err("truncated header len".into());
    }
    let header_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if pos + header_len > data.len() {
        return Err("truncated header".into());
    }
    let header: WorldHeader = opts
        .deserialize(&data[pos..pos + header_len])
        .map_err(|e| format!("header deserialize: {e}"))?;
    pos += header_len;

    if pos + 4 > data.len() {
        return Err("truncated material count".into());
    }
    let mat_count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if pos + 4 > data.len() {
        return Err("truncated materials len".into());
    }
    let mat_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if pos + mat_len > data.len() {
        return Err("truncated materials".into());
    }
    let materials: Vec<realmatter_core::material::MaterialProps> = opts
        .deserialize(&data[pos..pos + mat_len])
        .map_err(|e| format!("materials deserialize: {e}"))?;
    if materials.len() != mat_count {
        return Err(format!(
            "material count mismatch: header {mat_count}, actual {}",
            materials.len()
        ));
    }
    pos += mat_len;

    if pos + 4 > data.len() {
        return Err("truncated chunk count".into());
    }
    let chunk_count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    let mut chunks: Vec<(ChunkCoord, ChunkSnapshot)> =
        Vec::with_capacity(chunk_count);
    for _ in 0..chunk_count {
        if pos + 4 > data.len() {
            return Err("truncated chunk coord len".into());
        }
        let coord_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        if pos + coord_len > data.len() {
            return Err("truncated chunk coord".into());
        }
        let coord: ChunkCoord = opts
            .deserialize(&data[pos..pos + coord_len])
            .map_err(|e| format!("coord deserialize: {e}"))?;
        pos += coord_len;

        if pos + 4 + 1 > data.len() {
            return Err("truncated cell payload len".into());
        }
        let payload_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let compressed_flag = data[pos];
        pos += 1;
        if pos + payload_len > data.len() {
            return Err("truncated cell payload".into());
        }
        let payload = &data[pos..pos + payload_len];
        pos += payload_len;

        let cell_bytes = if compressed_flag == 1 {
            decompress_bytes(payload)?
        } else {
            payload.to_vec()
        };
        let cells: Vec<Cell> = opts
            .deserialize(&cell_bytes)
            .map_err(|e| format!("cells deserialize: {e}"))?;

        if pos >= data.len() {
            return Err("truncated dirty flag".into());
        }
        let dirty = data[pos] != 0;
        pos += 1;

        chunks.push((
            coord,
            ChunkSnapshot {
                coord,
                cells,
                dirty,
            },
        ));
    }

    if pos + 4 > data.len() {
        return Err("truncated recovery len".into());
    }
    let rec_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if pos + rec_len > data.len() {
        return Err("truncated recovery".into());
    }
    let recovery: RecoveryMetadata = opts
        .deserialize(&data[pos..pos + rec_len])
        .map_err(|e| format!("recovery deserialize: {e}"))?;
    pos += rec_len;

    // Decoration layer: present in version 2+ saves when non-empty.
    let mut deco: Option<Vec<u32>> = None;
    if version >= 2 && pos < data.len() {
        let present = data[pos];
        pos += 1;
        if present == 1 {
            if pos + 4 > data.len() {
                return Err("truncated deco len".into());
            }
            let deco_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            if pos + deco_len > data.len() {
                return Err("truncated deco payload".into());
            }
            let deco_bytes = decompress_bytes(&data[pos..pos + deco_len])?;
            pos += deco_len;
            let colors: Vec<u32> = opts
                .deserialize(&deco_bytes)
                .map_err(|e| format!("deco deserialize: {e}"))?;
            deco = Some(colors);
        }
    }

    if pos + 4 > data.len() {
        return Err("truncated checksum".into());
    }
    let stored_crc = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
    pos += 4;
    let computed = crc32(&data[..data.len() - 4]);
    if stored_crc != computed {
        return Err(format!(
            "checksum mismatch: stored {stored_crc:#x}, computed {computed:#x}"
        ));
    }

    chunks.sort_by_key(|(c, _)| *c);

    let mut recovery = recovery;
    recovery.chunk_count = chunks.len() as u32;

    Ok(SaveData {
        header,
        materials,
        chunks,
        recovery,
        deco,
    })
}

pub fn to_world(save: &SaveData) -> Result<World, String> {
    use std::sync::Arc;
    use realmatter_core::material::MaterialRegistry;

    let registry: Arc<dyn MaterialRegistry> = Arc::new(crate::SimpleMaterialRegistry::new(save.materials.clone()));
    let mut world = World::new(save.header.config.clone(), registry).map_err(|e| format!("{e:?}"))?;
    // Load cells from chunks (flatten into world grid).
    for (_coord, snap) in &save.chunks {
        let cs = save.header.config.chunk_size as i32;
        for (i, cell) in snap.cells.iter().enumerate() {
            let lx = (i as i32) % cs;
            let ly = (i as i32) / cs;
            let gx = snap.coord.x * cs + lx;
            let gy = snap.coord.y * cs + ly;
            if gx >= 0 && gy >= 0 && (gx as u32) < save.header.config.width && (gy as u32) < save.header.config.height {
                if let Some(slot) = world.cell_mut(gx, gy) {
                    *slot = *cell;
                }
            }
        }
    }
    // tick_count/time_accumulator restored via public setters (v1 header).
    world.set_tick_count(save.header.tick_count);
    world.set_time_accumulator(save.header.time_accumulator);
    Ok(world)
}

pub fn migrate(blob: &[u8], version: u32) -> Result<Vec<u8>, String> {
    if version == FORMAT_VERSION {
        return Ok(blob.to_vec());
    }
    if version < MIN_SUPPORTED_VERSION {
        return Err(format!("version {version} too old to migrate"));
    }
    Err(format!(
        "no migration path from version {version} to {FORMAT_VERSION}"
    ))
}

pub fn compress_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).map_err(|e| e.to_string())?;
    enc.finish().map_err(|e| e.to_string())
}

pub fn decompress_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    let mut dec = ZlibDecoder::new(data);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

pub fn compress_blob(blob: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(4 + blob.len());
    out.extend_from_slice(b"ZLIB");
    out.extend_from_slice(&compress_bytes(blob)?);
    Ok(out)
}

pub fn decompress_blob(blob: &[u8]) -> Result<Vec<u8>, String> {
    if blob.len() < 4 || &blob[0..4] != b"ZLIB" {
        return Ok(blob.to_vec());
    }
    decompress_bytes(&blob[4..])
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &b in data {
        let mut c = crc ^ (b as u32);
        for _ in 0..8 {
            if c & 1 != 0 {
                c = (c >> 1) ^ 0xEDB88320;
            } else {
                c >>= 1;
            }
        }
        crc = c;
    }
    crc ^ 0xFFFFFFFF
}


