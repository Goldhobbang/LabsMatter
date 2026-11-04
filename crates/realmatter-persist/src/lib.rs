//! REALMATTER persistence: save/load, serialization, storage helpers.
//!
//! Implements `realmatter_core::Persistence` for `PersistEngine`.
//! Owns save format versioning, migrations, compression, material loading,
//! restricted declarative extension format, incremental saves, recovery metadata.
//!
//! Offline only. No arbitrary code execution.

pub mod format;
pub mod material_ext;

use format::{
    compress_blob, crc32, decode, decompress_blob, encode, migrate, to_world, SaveData,
    FORMAT_VERSION, MAGIC,
};
use material_ext::{load_materials_json, merge_extensions, validate_manifest, MaterialDef, MaterialManifest};
use realmatter_core::material::Material;
use realmatter_core::material::MaterialRegistry;
use realmatter_core::Persistence;
use realmatter_core::SimError;
use realmatter_core::SimResult;
use realmatter_core::World;
use realmatter_core::WorldConfig;
use std::collections::BTreeSet;

pub struct PersistEngine {
    pub materials: MaterialRegistry,
    pub sequence: u64,
}

impl PersistEngine {
    pub fn new(materials: MaterialRegistry) -> Self {
        Self {
            materials,
            sequence: 0,
        }
    }

    pub fn with_materials(materials: Vec<Material>) -> Self {
        Self::new(MaterialRegistry::new(materials))
    }

    pub fn from_material_manifest(json: &str) -> Result<Self, String> {
        let defs = load_materials_json(json).map_err(|errs| {
            errs.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        let mats: Vec<Material> = defs.iter().map(material_ext::to_material).collect();
        Ok(Self::with_materials(mats))
    }

    pub fn validate_material_manifest_json(json: &str) -> Vec<String> {
        match serde_json::from_str::<MaterialManifest>(json) {
            Ok(m) => validate_manifest(&m).iter().map(|e| e.to_string()).collect(),
            Err(e) => vec![format!("json parse error: {e}")],
        }
    }

    pub fn save_full(&mut self, world: &World, label: Option<String>) -> SimResult<Vec<u8>> {
        self.sequence += 1;
        let save = SaveData::from_world(world, label, self.sequence, false, 0);
        encode(&save).map_err(SimError::Serialization)
    }

    pub fn save_incremental(
        &mut self,
        world: &World,
        dirty_chunks: &BTreeSet<realmatter_core::coord::ChunkCoord>,
        parent_sequence: u64,
        label: Option<String>,
    ) -> SimResult<Vec<u8>> {
        self.sequence += 1;
        let mut save = SaveData::from_world(world, label, self.sequence, true, parent_sequence);
        save.chunks
            .retain(|(c, _)| dirty_chunks.contains(c));
        encode(&save).map_err(SimError::Serialization)
    }

    pub fn load_blob(&self, bytes: &[u8]) -> SimResult<World> {
        let raw = decompress_blob(bytes).map_err(SimError::Serialization)?;
        let _ = migrate(&raw, FORMAT_VERSION);
        let save = decode(&raw).map_err(SimError::Serialization)?;
        let mut world = to_world(&save).map_err(SimError::Serialization)?;
        if world.materials.is_empty() && !self.materials.is_empty() {
            world.materials = self.materials.clone();
        }
        Ok(world)
    }

    pub fn export(&mut self, world: &World, label: Option<String>) -> SimResult<Vec<u8>> {
        let blob = self.save_full(world, label)?;
        compress_blob(&blob).map_err(SimError::Serialization)
    }

    pub fn import(&self, bytes: &[u8]) -> SimResult<World> {
        let raw = decompress_blob(bytes).map_err(SimError::Serialization)?;
        self.load_blob(&raw)
    }

    pub fn validate_blob(bytes: &[u8]) -> Result<(), String> {
        let raw = decompress_blob(bytes)?;
        if raw.len() < 8 {
            return Err("blob too short".into());
        }
        if raw[0..4] != MAGIC {
            return Err("bad magic".into());
        }
        let version = u32::from_le_bytes(raw[4..8].try_into().unwrap());
        if version < 1 || version > FORMAT_VERSION {
            return Err(format!("unsupported version {version}"));
        }
        let _ = decode(&raw)?;
        Ok(())
    }

    pub fn apply_extension_materials(&mut self, ext: &[Material]) -> Result<(), String> {
        let base: Vec<Material> = self.materials.iter().cloned().collect();
        let merged = merge_extensions(&base, ext).map_err(|errs| {
            errs.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        self.materials = MaterialRegistry::new(merged);
        Ok(())
    }

    pub fn apply_extension_json(&mut self, json: &str) -> Result<(), String> {
        let defs = load_materials_json(json).map_err(|errs| {
            errs.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        let mats: Vec<Material> = defs.iter().map(material_ext::to_material).collect();
        self.apply_extension_materials(&mats)
    }
}

impl Persistence for PersistEngine {
    fn save(&self, world: &World) -> SimResult<Vec<u8>> {
        let save = SaveData::from_world(world, None, self.sequence, false, 0);
        encode(&save).map_err(SimError::Serialization)
    }

    fn load(&self, bytes: &[u8]) -> SimResult<World> {
        self.load_blob(bytes)
    }
}

pub use format::{compress_blob, decompress_blob, crc32};
pub use material_ext::{load_materials_json, merge_extensions, validate_manifest, MaterialDef, MaterialManifest};

#[cfg(test)]
mod tests {
    use super::*;
    use realmatter_core::cell::Cell;
    use realmatter_core::coord::CellCoord;
    use realmatter_core::particle::Particle;

    fn make_world() -> World {
        let mut w = World::new(WorldConfig::default(), MaterialRegistry::new(vec![]));
        let coord = CellCoord { x: 1, y: 2, z: 3 };
        let mut cell = w.cell_mut(coord);
        cell.particle = Particle {
            material: 1,
            temperature: 300.0,
            velocity: [0.1, 0.2, 0.3],
            flags: realmatter_core::particle::ParticleFlags(0),
            lifetime: 0.0,
            custom: 42,
        };
        w.tick_count = 7;
        w.time_accumulator = 0.123;
        w
    }

    #[test]
    fn roundtrip_full() {
        let w = make_world();
        let mut eng = PersistEngine::with_materials(vec![]);
        let blob = eng.save_full(&w, Some("test".into())).unwrap();
        let w2 = eng.load_blob(&blob).unwrap();
        assert_eq!(w2.tick_count, 7);
        let c = w2.cell(CellCoord { x: 1, y: 2, z: 3 }).unwrap();
        assert_eq!(c.particle.material, 1);
        assert_eq!(c.particle.custom, 42);
    }

    #[test]
    fn export_import() {
        let w = make_world();
        let mut eng = PersistEngine::with_materials(vec![]);
        let blob = eng.export(&w, None).unwrap();
        let w2 = eng.import(&blob).unwrap();
        assert_eq!(w2.tick_count, 7);
    }

    #[test]
    fn corrupted_magic() {
        let w = make_world();
        let mut eng = PersistEngine::with_materials(vec![]);
        let mut blob = eng.save_full(&w, None).unwrap();
        blob[0] ^= 0xFF;
        assert!(eng.load_blob(&blob).is_err());
    }

    #[test]
    fn corrupted_checksum() {
        let w = make_world();
        let mut eng = PersistEngine::with_materials(vec![]);
        let mut blob = eng.save_full(&w, None).unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0xFF;
        assert!(eng.load_blob(&blob).is_err());
    }

    #[test]
    fn validate_blob_ok() {
        let w = make_world();
        let mut eng = PersistEngine::with_materials(vec![]);
        let blob = eng.save_full(&w, None).unwrap();
        assert!(PersistEngine::validate_blob(&blob).is_ok());
    }

    #[test]
    fn material_manifest_load() {
        let json = r#"{"version":1,"materials":[{"id":1,"name":"sand","density":1600.0}]}"#;
        let defs = load_materials_json(json).unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "sand");
    }

    #[test]
    fn material_manifest_invalid() {
        let json = r#"{"version":1,"materials":[{"id":0,"name":"void"}]}"#;
        let res = load_materials_json(json);
        assert!(res.is_err());
    }

    #[test]
    fn incremental_save() {
        let w = make_world();
        let mut eng = PersistEngine::with_materials(vec![]);
        let dirty = BTreeSet::new();
        let blob = eng
            .save_incremental(&w, &dirty, 0, Some("inc".into()))
            .unwrap();
        let save = decode(&blob).unwrap();
        assert!(save.recovery.incremental);
        assert_eq!(save.chunks.len(), 0);
    }

    #[test]
    fn extension_merge() {
        let base = vec![Material {
            id: 1,
            name: "a".into(),
            density: 1.0,
            thermal_conductivity: 1.0,
            specific_heat: 1.0,
            melting_point: 273.15,
            boiling_point: 373.15,
            phase: realmatter_core::material::Phase::Solid,
            color: [0, 0, 0, 255],
            reactivity_tags: 0,
        }];
        let ext = vec![Material {
            id: 2,
            name: "b".into(),
            density: 2.0,
            thermal_conductivity: 1.0,
            specific_heat: 1.0,
            melting_point: 273.15,
            boiling_point: 373.15,
            phase: realmatter_core::material::Phase::Liquid,
            color: [0, 0, 0, 255],
            reactivity_tags: 0,
        }];
        let merged = merge_extensions(&base, &ext).unwrap();
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn extension_reuse_rejected() {
        let base = vec![Material {
            id: 1,
            name: "a".into(),
            density: 1.0,
            thermal_conductivity: 1.0,
            specific_heat: 1.0,
            melting_point: 273.15,
            boiling_point: 373.15,
            phase: realmatter_core::material::Phase::Solid,
            color: [0, 0, 0, 255],
            reactivity_tags: 0,
        }];
        let ext = vec![Material {
            id: 1,
            name: "b".into(),
            density: 2.0,
            thermal_conductivity: 1.0,
            specific_heat: 1.0,
            melting_point: 273.15,
            boiling_point: 373.15,
            phase: realmatter_core::material::Phase::Liquid,
            color: [0, 0, 0, 255],
            reactivity_tags: 0,
        }];
        assert!(merge_extensions(&base, &ext).is_err());
    }
}
