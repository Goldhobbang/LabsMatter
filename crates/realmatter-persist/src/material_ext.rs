//! Material definition loading, validation, and restricted declarative extension format.

use realmatter_core::material::{Material, Phase};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialDef {
    pub id: u16,
    pub name: String,
    #[serde(default = "default_density")]
    pub density: f32,
    #[serde(default)]
    pub thermal_conductivity: f32,
    #[serde(default = "default_specific_heat")]
    pub specific_heat: f32,
    #[serde(default = "default_melting")]
    pub melting_point: f32,
    #[serde(default = "default_boiling")]
    pub boiling_point: f32,
    #[serde(default)]
    pub phase: PhaseDef,
    #[serde(default = "default_color")]
    pub color: [u8; 4],
    #[serde(default)]
    pub reactivity_tags: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PhaseDef {
    #[default]
    Solid,
    Liquid,
    Gas,
    Plasma,
}

impl From<PhaseDef> for Phase {
    fn from(p: PhaseDef) -> Self {
        match p {
            PhaseDef::Solid => Phase::Solid,
            PhaseDef::Liquid => Phase::Liquid,
            PhaseDef::Gas => Phase::Gas,
            PhaseDef::Plasma => Phase::Plasma,
        }
    }
}

fn default_density() -> f32 {
    1000.0
}
fn default_specific_heat() -> f32 {
    4184.0
}
fn default_melting() -> f32 {
    273.15
}
fn default_boiling() -> f32 {
    373.15
}
fn default_color() -> [u8; 4] {
    [128, 128, 128, 255]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialManifest {
    #[serde(default = "default_manifest_version")]
    pub version: u32,
    pub materials: Vec<MaterialDef>,
}

fn default_manifest_version() -> u32 {
    1
}

#[derive(Clone, Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

pub fn validate_material(def: &MaterialDef) -> Vec<ValidationError> {
    let mut errs = Vec::new();
    if def.name.trim().is_empty() {
        errs.push(ValidationError {
            field: "name".into(),
            message: "must not be empty".into(),
        });
    }
    if def.name.len() > 64 {
        errs.push(ValidationError {
            field: "name".into(),
            message: "must be <= 64 chars".into(),
        });
    }
    if !def.density.is_finite() || def.density < 0.0 {
        errs.push(ValidationError {
            field: "density".into(),
            message: "must be finite and >= 0".into(),
        });
    }
    if !def.thermal_conductivity.is_finite() || def.thermal_conductivity < 0.0 {
        errs.push(ValidationError {
            field: "thermal_conductivity".into(),
            message: "must be finite and >= 0".into(),
        });
    }
    if !def.specific_heat.is_finite() || def.specific_heat <= 0.0 {
        errs.push(ValidationError {
            field: "specific_heat".into(),
            message: "must be finite and > 0".into(),
        });
    }
    if !def.melting_point.is_finite() {
        errs.push(ValidationError {
            field: "melting_point".into(),
            message: "must be finite".into(),
        });
    }
    if !def.boiling_point.is_finite() {
        errs.push(ValidationError {
            field: "boiling_point".into(),
            message: "must be finite".into(),
        });
    }
    if def.melting_point > def.boiling_point {
        errs.push(ValidationError {
            field: "melting_point".into(),
            message: "must be <= boiling_point".into(),
        });
    }
    errs
}

pub fn validate_manifest(manifest: &MaterialManifest) -> Vec<ValidationError> {
    let mut errs = Vec::new();
    if manifest.version != 1 {
        errs.push(ValidationError {
            field: "version".into(),
            message: format!("unsupported manifest version {}", manifest.version),
        });
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (i, m) in manifest.materials.iter().enumerate() {
        for e in validate_material(m) {
            errs.push(ValidationError {
                field: format!("materials[{i}].{}", e.field),
                message: e.message,
            });
        }
        if !ids.insert(m.id) {
            errs.push(ValidationError {
                field: format!("materials[{i}].id"),
                message: format!("duplicate id {}", m.id),
            });
        }
        if !names.insert(m.name.clone()) {
            errs.push(ValidationError {
                field: format!("materials[{i}].name"),
                message: format!("duplicate name {}", m.name),
            });
        }
        if m.id == 0 {
            errs.push(ValidationError {
                field: format!("materials[{i}].id"),
                message: "id 0 is reserved for void".into(),
            });
        }
    }
    errs
}

pub fn load_materials_json(json: &str) -> Result<Vec<MaterialDef>, Vec<ValidationError>> {
    let manifest: MaterialManifest = serde_json::from_str(json).map_err(|e| {
        vec![ValidationError {
            field: "json".into(),
            message: format!("parse error: {e}"),
        }]
    })?;
    let errs = validate_manifest(&manifest);
    if !errs.is_empty() {
        return Err(errs);
    }
    Ok(manifest.materials)
}

pub fn to_material(def: &MaterialDef) -> Material {
    Material {
        id: def.id,
        name: def.name.clone(),
        density: def.density,
        thermal_conductivity: def.thermal_conductivity,
        specific_heat: def.specific_heat,
        melting_point: def.melting_point,
        boiling_point: def.boiling_point,
        phase: def.phase.into(),
        color: def.color,
        reactivity_tags: def.reactivity_tags,
    }
}

pub fn merge_extensions(
    base: &[Material],
    ext: &[Material],
) -> Result<Vec<Material>, Vec<ValidationError>> {
    let mut errs = Vec::new();
    let base_ids: BTreeSet<u16> = base.iter().map(|m| m.id).collect();
    let base_names: BTreeSet<String> = base.iter().map(|m| m.name.clone()).collect();
    for m in ext {
        if base_ids.contains(&m.id) {
            errs.push(ValidationError {
                field: "id".into(),
                message: format!("extension reuses existing id {}", m.id),
            });
        }
        if base_names.contains(&m.name) {
            errs.push(ValidationError {
                field: "name".into(),
                message: format!("extension reuses existing name {}", m.name),
            });
        }
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for m in ext {
        if !ids.insert(m.id) {
            errs.push(ValidationError {
                field: "id".into(),
                message: format!("duplicate extension id {}", m.id),
            });
        }
        if !names.insert(m.name.clone()) {
            errs.push(ValidationError {
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
