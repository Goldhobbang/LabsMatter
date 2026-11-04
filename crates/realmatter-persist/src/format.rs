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

use realmatter_core::chunk::CHUNK_VOLUME;
use realmatter_core::coord::ChunkCoord;
use realmatter_core::material::Material;
use realmatter_core::world::World;
use realmatter_core::world::WorldConfig;
use serde::{Deserialize, Serialize};

pub const MAGIC: [u8; 4] = *b"RMT\0";
pub const FORMAT_VERSION: u32 = 1;
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
    pub materials: Vec<Material>,
    pub chunks: Vec<(ChunkCoord, realmatter_core::chunk::Chunk)>,
    pub recovery: RecoveryMetadata,
}

impl SaveData {
    pub fn from_world(
        world: &World,
        label: Option<String>,
        sequence: u64,
        incremental: bool,
        parent: u64,
    ) -> Self {
        let mut chunks: Vec<(ChunkCoord, realmatter_core::chunk::Chunk)> = world
            .chunks()
            .iter()
            .map(|(c, ch)| (*c, ch.clone()))
            .collect();
        chunks.sort_by_key(|(c, _)| *c);

        let particle_count: u64 = chunks
            .iter()
            .flat_map(|(_, ch)| ch.cells.iter())
            .filter(|cell| !cell.is_empty())
            .count() as u64;

        let materials: Vec<Material> = world.materials.iter().cloned().collect();

        Self {
            header: WorldHeader {
                config: world.config.clone(),
                tick_count: world.tick_count,
                time_accumulator: world.time_accumulator,
                compressed_chunks: true,
                label,
            },
            materials,
            chunks,
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

    let bytes = migrate(bytes, version)?;
    let bytes = bytes.as_slice();
    if bytes[0..4] != MAGIC {
        return Err("bad magic after migration".into());
    }
    pos = 4;
    let _version = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
    pos += 4;

    if pos + 4 > bytes.len() {
        return Err("truncated header len".into());
    }
    let header_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if pos + header_len > bytes.len() {
        return Err("truncated header".into());
    }
    let header: WorldHeader = opts
        .deserialize(&bytes[pos..pos + header_len])
        .map_err(|e| format!("header deserialize: {e}"))?;
    pos += header_len;

    if pos + 4 > bytes.len() {
        return Err("truncated material count".into());
    }
    let mat_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let materials: Vec<Material> = opts
        .deserialize(&bytes[pos..])
        .map_err(|e| format!("materials deserialize: {e}"))?;
    let mat_bytes = opts
        .serialize(&materials)
        .map_err(|e| format!("materials reserialize: {e}"))?;
    if materials.len() != mat_count {
        return Err(format!(
            "material count mismatch: header {mat_count}, actual {}",
            materials.len()
        ));
    }
    pos += mat_bytes.len();

    if pos + 4 > bytes.len() {
        return Err("truncated chunk count".into());
    }
    let chunk_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    let mut chunks: Vec<(ChunkCoord, realmatter_core::chunk::Chunk)> =
        Vec::with_capacity(chunk_count);
    for _ in 0..chunk_count {
        if pos + 4 > bytes.len() {
            return Err("truncated chunk coord len".into());
        }
        let coord_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        if pos + coord_len > bytes.len() {
            return Err("truncated chunk coord".into());
        }
        let coord: ChunkCoord = opts
            .deserialize(&bytes[pos..pos + coord_len])
            .map_err(|e| format!("coord deserialize: {e}"))?;
        pos += coord_len;

        if pos + 4 + 1 > bytes.len() {
            return Err("truncated cell payload len".into());
        }
        let payload_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let compressed_flag = bytes[pos];
        pos += 1;
        if pos + payload_len > bytes.len() {
            return Err("truncated cell payload".into());
        }
        let payload = &bytes[pos..pos + payload_len];
        pos += payload_len;

        let cell_bytes = if compressed_flag == 1 {
            decompress_bytes(payload)?
        } else {
            payload.to_vec()
        };
        let cells: [realmatter_core::cell::Cell; CHUNK_VOLUME] = opts
            .deserialize(&cell_bytes)
            .map_err(|e| format!("cells deserialize: {e}"))?;

        if pos >= bytes.len() {
            return Err("truncated dirty flag".into());
        }
        let dirty = bytes[pos] != 0;
        pos += 1;

        let mut chunk = realmatter_core::chunk::Chunk::new(coord);
        chunk.cells = cells;
        chunk.dirty = dirty;
        chunk.coord = coord;
        chunks.push((coord, chunk));
    }

    if pos + 4 > bytes.len() {
        return Err("truncated recovery len".into());
    }
    let rec_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if pos + rec_len > bytes.len() {
        return Err("truncated recovery".into());
    }
    let recovery: RecoveryMetadata = opts
        .deserialize(&bytes[pos..pos + rec_len])
        .map_err(|e| format!("recovery deserialize: {e}"))?;
    pos += rec_len;

    if pos + 4 > bytes.len() {
        return Err("truncated checksum".into());
    }
    let stored_crc = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
    pos += 4;
    let computed = crc32(&bytes[..bytes.len() - 4]);
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
    })
}

pub fn to_world(save: &SaveData) -> Result<realmatter_core::world::World, String> {
    use realmatter_core::material::MaterialRegistry;
    let registry = MaterialRegistry::new(save.materials.clone());
    let mut world = realmatter_core::world::World::new(save.header.config.clone(), registry);
    world.tick_count = save.header.tick_count;
    world.time_accumulator = save.header.time_accumulator;
    for (coord, chunk) in &save.chunks {
        world.chunks_mut().insert(*coord, chunk.clone());
    }
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
