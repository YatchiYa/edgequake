//! Shared JSON-format extraction prompts (LLMExtractor + GleaningExtractor).
//!
//! Single source of truth for JSON output schema text — avoids drift between
//! `extractor/llm.rs` and `extractor/gleaning.rs`.

use super::entity_type_policy::{json_entity_types_prompt_section, EntityExtractionSchema};
use super::extract_caps::ExtractionCaps;
use super::language::json_language_instruction;

/// Canonical JSON output format section shared by all JSON extractors.
pub const JSON_OUTPUT_FORMAT_SECTION: &str = r#"## Output Format
Respond with valid JSON in this exact format:
{
  "entities": [
    {"name": "Entity Name", "type": "ENTITY_TYPE", "description": "Brief description"}
  ],
  "relationships": [
    {"source": "Source Entity", "target": "Target Entity", "type": "RELATIONSHIP_TYPE", "description": "Brief description"}
  ]
}"#;

/// Build the primary JSON entity-extraction user prompt.
pub fn json_extraction_prompt(
    text: &str,
    schema: &EntityExtractionSchema,
    language: &str,
) -> String {
    let types_section = json_entity_types_prompt_section(schema);
    let caps = ExtractionCaps::from_env();
    let limits = caps.prompt_quantity_limits_section();
    let language_section = json_language_instruction(language);

    format!(
        r#"Extract entities and relationships from the following text.

{types_section}

{limits}

{language_section}

## Naming Rules
- Use human-readable semantic names for entities (title case when case-insensitive).
- Do **not** use UUIDs, GUIDs, ULIDs, MongoDB ObjectIds, hex hashes, AWS ARNs, or other opaque machine/resource IDs as entity `name`. Prefer the human referent; opaque IDs may appear only in `description`.

{JSON_OUTPUT_FORMAT_SECTION}

## Text to Analyze
{text}

## JSON Response"#
    )
}

/// Build the gleaning (re-extraction) JSON prompt for missed entities.
pub fn json_gleaning_prompt(
    text: &str,
    previous_entities: &[String],
    schema: &EntityExtractionSchema,
    language: &str,
) -> String {
    let prev_entities_str = previous_entities.join(", ");
    let types_section = json_entity_types_prompt_section(schema);
    let caps = ExtractionCaps::from_env();
    let limits = caps.prompt_quantity_limits_section();
    let language_section = json_language_instruction(language);

    format!(
        r#"MANY entities and relationships were missed in the last extraction.
Please identify any ADDITIONAL entities and relationships that were not already captured.

## Already Identified Entities
{prev_entities_str}

## Instructions
Look for entities and relationships that were missed in the previous extraction.
Focus on:
- Implicit entities (mentioned indirectly)
- Additional relationships between known entities
- Contextual entities (locations, concepts, methods, natural objects)
- Do **not** add UUIDs, GUIDs, hashes, ARNs, or other opaque machine IDs as entity names

{types_section}

{limits}

{language_section}

{JSON_OUTPUT_FORMAT_SECTION}

## Text to Re-Analyze
{text}

## JSON Response"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::EntityExtractionSchema;

    #[test]
    fn json_extraction_prompt_includes_types_and_format() {
        let schema = EntityExtractionSchema::server_default();
        let prompt = json_extraction_prompt("Alice works at Acme.", &schema, "English");
        assert!(prompt.contains("Text to Analyze"));
        assert!(prompt.contains("Alice works at Acme."));
        assert!(prompt.contains("\"entities\""));
        assert!(prompt.contains("JSON Response"));
        assert!(prompt.contains("Naming Rules"));
        assert!(prompt.contains("UUID"));
    }

    #[test]
    fn json_extraction_prompt_includes_quantity_caps() {
        let schema = EntityExtractionSchema::server_default();
        let prompt = json_extraction_prompt("Alice works at Acme.", &schema, "English");
        assert!(prompt.contains("Quantity Limits"));
        assert!(prompt.contains("40"));
        assert!(prompt.contains("100"));
    }

    #[test]
    fn json_gleaning_prompt_lists_previous_entities() {
        let schema = EntityExtractionSchema::server_default();
        let prompt = json_gleaning_prompt(
            "Some text.",
            &["ALICE".into(), "ACME".into()],
            &schema,
            "English",
        );
        assert!(prompt.contains("ALICE, ACME"));
        assert!(prompt.contains("Text to Re-Analyze"));
        assert!(prompt.contains("\"relationships\""));
        assert!(!prompt.to_lowercase().contains("dates,"));
        assert!(prompt.contains("Quantity Limits"));
    }

    #[test]
    fn json_gleaning_prompt_includes_strict_entity_types() {
        let schema = EntityExtractionSchema {
            types: vec!["API_OR_INTERFACE".into(), "OTHER".into()],
            strict: true,
        };
        let prompt = json_gleaning_prompt("x", &[], &schema, "English");
        assert!(prompt.contains("API_OR_INTERFACE"));
        assert!(prompt.contains("STRICT") || prompt.contains("ONLY"));
    }

    #[test]
    fn spec096_json_prompt_includes_language() {
        let schema = EntityExtractionSchema::server_default();
        let prompt = json_extraction_prompt("Alice works at Acme.", &schema, "Chinese");
        assert!(prompt.contains("Chinese"));
        assert!(prompt.contains("Output Language"));
        assert!(prompt.contains("\"entities\""));
        assert!(prompt.contains("\"name\""));
        assert!(prompt.contains("JSON object/array keys exactly as specified in English"));
    }

    #[test]
    fn spec096_gleaning_prompt_includes_language() {
        let schema = EntityExtractionSchema::server_default();
        let prompt = json_gleaning_prompt("text", &["A".into()], &schema, "Chinese");
        assert!(prompt.contains("Chinese"));
        assert!(prompt.contains("Output Language"));
        let instruction = json_language_instruction("Chinese");
        assert!(prompt.contains(&instruction));
    }

    #[test]
    fn spec096_json_prompt_default_english() {
        let schema = EntityExtractionSchema::server_default();
        let prompt = json_extraction_prompt("Hello.", &schema, "English");
        assert!(prompt.contains("English"));
        assert!(prompt.contains("Output Language"));
        assert!(prompt.contains("\"entities\""));
    }
}
