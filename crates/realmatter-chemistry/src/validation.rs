//! Reaction schema validation.

use crate::schema::ReactionSchema;

#[derive(Clone, Debug)]
pub struct ValidationError {
    pub message: String,
}

pub fn validate_schema(s: &ReactionSchema) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    if s.equation.reactants.is_empty() {
        errors.push(ValidationError { message: "no reactants".into() });
    }
    if s.equation.products.is_empty() {
        errors.push(ValidationError { message: "no products".into() });
    }
    if s.params.pre_exponential < 0.0 {
        errors.push(ValidationError { message: "pre_exponential negative".into() });
    }
    if s.params.activation_energy < 0.0 {
        errors.push(ValidationError { message: "activation_energy negative".into() });
    }
    errors
}
