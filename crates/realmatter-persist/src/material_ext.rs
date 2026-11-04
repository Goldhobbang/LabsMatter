//! Material definition loading, validation, and restricted declarative extension format.
//!
//! Maps to `realmatter_core::material::{MaterialProps, MaterialState}` (2D).

use realmatter_core::material::{MaterialProps, MaterialState};
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
    pub state: StateDef,
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    #[serde(default)]
    pub flammable: bool,
    #[serde(default)]
    pub ignition_temp: f32,
    #[serde(default)]
    pub viscosity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StateDef {
    #[default]
    Solid,
    Liquid,
    Gas,
    Plasma,
    Powder,
}

impl From<StateDef> for MaterialState {
    fn from(p: StateDef) -> Self {
        match p {
            StateDef::Solid => MaterialState::Solid,
            StateDef::Liquid => MaterialState::Liquid,
            StateDef::Gas => MaterialState::Gas,
            StateDef::Plasma => MaterialState::Plasma,
            StateDef::Powder => MaterialState::Powder,
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
fn default_color() -> [f32; 4] {
    [0.5, 0.5, 0.5, 1.0]
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

pub fn to_material_props(def: &MaterialDef) -> MaterialProps {
    MaterialProps {
        id: def.id,
        name: def.name.clone(),
        state: def.state.into(),
        density: def.density,
        specific_heat: def.specific_heat,
        thermal_conductivity: def.thermal_conductivity,
        melting_point: def.melting_point,
        boiling_point: def.boiling_point,
        viscosity: def.viscosity,
        color: def.color,
        flammable: def.flammable,
        ignition_temp: def.ignition_temp,
    }
}
