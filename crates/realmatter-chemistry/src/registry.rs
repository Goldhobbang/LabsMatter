//! Data-driven chemical reaction registry.

use crate::core::material::MaterialId;
use crate::schema::{ReactionId, ReactionSchema};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct ReactionDefinition {
    pub schema: ReactionSchema,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ReactionRegistry {
    by_id: BTreeMap<ReactionId, ReactionDefinition>,
    /// Fast lookup by reactant pair (material_a, material_b) -> reaction ids.
    by_pair: BTreeMap<(MaterialId, MaterialId), Vec<ReactionId>>,
}

impl ReactionRegistry {
    pub fn new() -> Self {
        Self {
            by_id: BTreeMap::new(),
            by_pair: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, schema: ReactionSchema) {
        let id = schema.id;
        // Index pairs.
        if !schema.equation.reactants.is_empty() && schema.equation.reactants.len() >= 2 {
            let a = schema.equation.reactants[0].0;
            let b = schema.equation.reactants[1].0;
            let key = if a < b { (a, b) } else { (b, a) };
            self.by_pair.entry(key).or_default().push(id);
        } else if schema.equation.reactants.len() == 1 && !schema.equation.products.is_empty() {
            // Unary-like; still allow lookup by (a,a).
            let a = schema.equation.reactants[0].0;
            self.by_pair.entry((a, a)).or_default().push(id);
        }
        self.by_id.insert(id, ReactionDefinition { schema, enabled: true });
    }

    pub fn get(&self, id: ReactionId) -> Option<&ReactionDefinition> {
        self.by_id.get(&id)
    }

    pub fn find_for_pair(&self, a: MaterialId, b: MaterialId) -> Vec<ReactionId> {
        let key = if a < b { (a, b) } else { (b, a) };
        self.by_pair.get(&key).cloned().unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ReactionDefinition> {
        self.by_id.values()
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Create default registry with common reactions.
    ///
    /// Material IDs are the canonical builtin IDs (see WASM `StaticRegistry`
    /// and `web/src/materials.js`): fuel wood (7), oxidizer proxy empty (0),
    /// iron (8), salt (11), water (3). Products are documentary — the engine
    /// applies flags/temperature/lifetime effects, not stoichiometric swaps.
    pub fn default_set() -> Self {
        let mut r = Self::new();
        // Combustion: wood (flammable) + air proxy (empty). The engine gates
        // on flammability + ignition temperature, so this entry indexes
        // wood/air pairs for the combustion path.
        r.add(ReactionSchema {
            id: 1,
            name: "combustion.generic".into(),
            equation: ReactionEquation { reactants: vec![(7,1),(0,1)], products: vec![(12,1)] },
            params: ReactionParams { activation_energy: 15000.0, pre_exponential: 1000.0, enthalpy: -50000.0, rate_multiplier: 1.0, min_temp: 300.0, max_temp: f32::INFINITY, combustion: true, ignition_temp: 560.0, requires_wet: false },
            tags: 1,
        });
        // Oxidation (slow): iron + iron neighbors.
        r.add(ReactionSchema {
            id: 2,
            name: "oxidation.slow".into(),
            equation: ReactionEquation { reactants: vec![(8,1),(8,1)], products: vec![(8,2)] },
            params: ReactionParams { activation_energy: 50000.0, pre_exponential: 1.0, enthalpy: -1000.0, rate_multiplier: 0.01, min_temp: 293.15, max_temp: f32::INFINITY, combustion: false, ignition_temp: f32::INFINITY, requires_wet: false },
            tags: 2,
        });
        // Dissolution: salt in water.
        r.add(ReactionSchema {
            id: 3,
            name: "dissolution".into(),
            equation: ReactionEquation { reactants: vec![(11,1),(3,1)], products: vec![(3,2)] },
            params: ReactionParams { activation_energy: 1000.0, pre_exponential: 10.0, enthalpy: 0.0, rate_multiplier: 1.0, min_temp: 273.15, max_temp: f32::INFINITY, combustion: false, ignition_temp: f32::INFINITY, requires_wet: true },
            tags: 4,
        });
        r
    }
}
use crate::schema::{ReactionEquation, ReactionParams};
