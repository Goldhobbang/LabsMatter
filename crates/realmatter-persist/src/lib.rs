//! REALMATTER persistence: save/load, serialization, storage helpers.
//!
//! Owned by A2 (Persistence merged). Implements serialization against
//! `realmatter_core` + `realmatter_world` 2D APIs.

pub mod format;
pub mod material_ext;

use std::collections::BTreeSet;
use std::sync::Arc;

use format::{decode, encode, migrate, to_world, SaveData, FORMAT_VERSION, MAGIC};
use realmatter_core::coord::ChunkCoord;
use realmatter_core::error::{RealmatterError, Result};
use realmatter_core::material::{MaterialProps, MaterialRegistry};
use realmatter_core as core;
use realmatter_world::World;

/// Persistence engine for 2D worlds.
pub struct PersistEngine {
    pub materials: Arc<dyn MaterialRegistry>,
    pub sequence: u64,
}

impl PersistEngine {
    pub fn new(materials: Arc<dyn MaterialRegistry>) -> Self {
        Self {
            materials,
            sequence: 0,
        }
    }

    pub fn with_material_props(materials: Vec<MaterialProps>) -> Self {
        let registry = Arc::new(SimpleMaterialRegistry::new(materials));
        Self::new(registry)
    }

    pub fn from_material_manifest(json: &str) -> std::result::Result<Self, String> {
        let defs = material_ext::load_materials_json(json).map_err(|errs| {
            errs.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        let mats: Vec<MaterialProps> = defs.iter().map(material_ext::to_material_props).collect();
        Ok(Self::with_material_props(mats))
    }

    pub fn validate_material_manifest_json(json: &str) -> Vec<String> {
        match serde_json::from_str::<material_ext::MaterialManifest>(json) {
            Ok(m) => material_ext::validate_manifest(&m).iter().map(|e| e.to_string()).collect(),
            Err(e) => vec![format!("json parse error: {e}")],
        }
    }

    /// Save full world state.
    pub fn save_full(&mut self, world: &mut World, label: Option<String>) -> Result<Vec<u8>> {
        self.save_full_with_deco(world, label, None)
    }

    /// Save full world state including the decoration layer.
    pub fn save_full_with_deco(
        &mut self,
        world: &mut World,
        label: Option<String>,
        deco: Option<Vec<u32>>,
    ) -> Result<Vec<u8>> {
        self.sequence += 1;
        let mut save = format::SaveData::from_world(world, label, self.sequence, false, 0);
        save.materials = self.materials.all_props();
        save.deco = deco;
        format::encode(&save).map_err(|e| RealmatterError::Serialization(e))
    }

    /// Save only dirty chunks (incremental).
    pub fn save_incremental(
        &mut self,
        world: &mut World,
        dirty_chunks: &BTreeSet<ChunkCoord>,
        parent_sequence: u64,
        label: Option<String>,
    ) -> Result<Vec<u8>> {
        self.sequence += 1;
        let mut save = format::SaveData::from_world(world, label, self.sequence, true, parent_sequence);
        save.materials = self.materials.all_props();
        save.chunks.retain(|(c, _)| dirty_chunks.contains(c));
        format::encode(&save).map_err(|e| RealmatterError::Serialization(e))
    }

    /// Load a raw save blob into a world.
    pub fn load_blob(&self, bytes: &[u8]) -> Result<World> {
        let raw = format::decompress_blob(bytes).map_err(|e| RealmatterError::Serialization(e))?;
        let _ = format::migrate(&raw, format::FORMAT_VERSION).map_err(|e| RealmatterError::Serialization(e))?;
        let mut save = format::decode(&raw).map_err(|e| RealmatterError::Serialization(e))?;
        if save.materials.is_empty() {
            save.materials = self.materials.all_props();
        }
        let world = format::to_world(&save).map_err(|e| RealmatterError::Serialization(e))?;
        Ok(world)
    }

    pub fn export(&mut self, world: &mut World, label: Option<String>) -> Result<Vec<u8>> {
        let blob = self.save_full(world, label)?;
        format::compress_blob(&blob).map_err(|e| RealmatterError::Serialization(e))
    }

    /// Import compressed save blob.
    pub fn import(&self, bytes: &[u8]) -> Result<World> {
        let raw = format::decompress_blob(bytes).map_err(|e| RealmatterError::Serialization(e))?;
        self.load_blob(&raw)
    }

    /// Validate a blob's header/magic/version/crc.
    pub fn validate_blob(bytes: &[u8]) -> std::result::Result<(), String> {
        let raw = format::decompress_blob(bytes)?;
        if raw.len() < 8 {
            return Err("blob too short".into());
        }
        if raw[0..4] != format::MAGIC {
            return Err("bad magic".into());
        }
        let version = u32::from_le_bytes(raw[4..8].try_into().unwrap());
        if version < 1 || version > format::FORMAT_VERSION {
            return Err(format!("unsupported version {version}"));
        }
        let _ = format::decode(&raw)?;
        Ok(())
    }

    pub fn apply_extension_materials(&mut self, ext: &[MaterialProps]) -> std::result::Result<(), String> {
        let base = self.materials.all_props();
        let merged = merge_extensions_props(&base, ext).map_err(|errs| {
            errs.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        self.materials = Arc::new(SimpleMaterialRegistry::new(merged));
        Ok(())
    }

    pub fn apply_extension_json(&mut self, json: &str) -> std::result::Result<(), String> {
        let defs = material_ext::load_materials_json(json).map_err(|errs| {
            errs.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        let mats: Vec<MaterialProps> = defs.iter().map(material_ext::to_material_props).collect();
        self.apply_extension_materials(&mats)
    }
}

#[derive(Clone)]
struct SimpleMaterialRegistry {
    by_id: std::collections::BTreeMap<realmatter_core::material::MaterialId, MaterialProps>,
}

impl SimpleMaterialRegistry {
    fn new(mats: Vec<MaterialProps>) -> Self {
        let mut by_id = std::collections::BTreeMap::new();
        for m in mats {
            by_id.insert(m.id, m);
        }
        Self { by_id }
    }
}

impl realmatter_core::material::MaterialRegistry for SimpleMaterialRegistry {
    fn get(&self, id: realmatter_core::material::MaterialId) -> Option<&MaterialProps> {
        self.by_id.get(&id)
    }
    fn len(&self) -> usize {
        self.by_id.len()
    }
    fn all_props(&self) -> Vec<MaterialProps> {
        self.by_id.values().cloned().collect()
    }
}

/// Merge extension material props (append-only; no id reuse).
fn merge_extensions_props(
    base: &[MaterialProps],
    ext: &[MaterialProps],
) -> std::result::Result<Vec<MaterialProps>, Vec<material_ext::ValidationError>> {
    let mut errs = Vec::new();
    let base_ids: std::collections::BTreeSet<realmatter_core::material::MaterialId> = base.iter().map(|m| m.id).collect();
    let base_names: BTreeSet<String> = base.iter().map(|m| m.name.to_string()).collect();
    for m in ext {
        if base_ids.contains(&m.id) {
            errs.push(material_ext::ValidationError {
                field: "id".into(),
                message: format!("extension reuses existing id {}", m.id),
            });
        }
        if base_names.contains(&m.name.to_string()) {
            errs.push(material_ext::ValidationError {
                field: "name".into(),
                message: format!("extension reuses existing name {}", m.name),
            });
        }
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for m in ext {
        if !ids.insert(m.id) {
            errs.push(material_ext::ValidationError {
                field: "id".into(),
                message: format!("duplicate extension id {}", m.id),
            });
        }
        if !names.insert(m.name.to_string()) {
            errs.push(material_ext::ValidationError {
                field: "name".into(),
                message: format!("duplicate extension name {}", m.name),
            });
        }
    }
    if !errs.is_empty() {
        return Err(errs);
    }
    let mut out = base.to_vec();
    out.extend_from_slice(ext);
    out.sort_by_key(|m| m.id);
    Ok(out)
}








pub use format::{compress_blob, decompress_blob, crc32};
pub use material_ext::{load_materials_json, validate_manifest, MaterialDef, MaterialManifest};
