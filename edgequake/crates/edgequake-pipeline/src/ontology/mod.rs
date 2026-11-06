//! Workspace ontology (SPEC-184): the company's types, relations and reference terms
//! guide extraction, and existing graphs can be realigned to them.
//!
//! Without `metadata.ontology` nothing here runs (LAW-184-1).

mod compiled;
mod extractor;
pub mod import;
mod model;
pub mod plan;

pub use compiled::{CanonicalizeStats, CompiledOntology};
pub use extractor::OntologyExtractor;
pub use model::{
    schema_projection, validate, OntologyEntityType, OntologyRelationType, OntologySource,
    OntologyTerm, SchemaProjection, Validated, ValidationMode, WorkspaceOntology, MAX_ALIASES,
    MAX_ENTITY_TYPES, MAX_RELATION_EDGES, MAX_RELATION_TYPES, MAX_SYNONYMS, MAX_TERMS,
    METADATA_KEY, SCHEMA_BEFORE_KEY,
};
