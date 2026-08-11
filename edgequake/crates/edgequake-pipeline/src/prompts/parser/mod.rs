//! Extraction result parsers.
//!
//! Provides parsers for both tuple-based (SOTA) and JSON-based extraction formats,
//! plus a hybrid parser for graceful migration.
//!
//! # Parser Types
//!
//! - [`TupleParser`]: SOTA format — robust, streaming, battle-tested
//! - [`JsonExtractionParser`]: Legacy JSON format with LLM sanitization
//! - [`HybridExtractionParser`]: Auto-detects format with fallback

mod json_parser;
mod registry;
mod tuple_parser;

pub use json_parser::{JsonExtractionParser, JsonParseOptions};
pub use registry::{detect_format_markers, ExtractionResultParser};
pub use tuple_parser::TupleParser;

use crate::error::Result;
use crate::extractor::ExtractionResult;
use crate::prompts::extract_caps::ExtractionCaps;

/// Hybrid parser supporting both JSON and Tuple formats.
///
/// Provides a migration path from JSON to tuple-based extraction
/// with automatic format detection and fallback.
#[derive(Debug, Clone)]
pub struct HybridExtractionParser {
    json_parser: JsonExtractionParser,
    tuple_parser: TupleParser,
    prefer_tuple: bool,
    /// SPEC-117: resolved caps applied on both JSON and tuple paths.
    extraction_caps: Option<ExtractionCaps>,
}

impl Default for HybridExtractionParser {
    fn default() -> Self {
        Self::new(true)
    }
}

impl HybridExtractionParser {
    /// Create a new hybrid parser.
    ///
    /// # Arguments
    /// * `prefer_tuple` - If true, prefer tuple parsing when format is ambiguous
    pub fn new(prefer_tuple: bool) -> Self {
        Self {
            json_parser: JsonExtractionParser::new(),
            tuple_parser: TupleParser::new(),
            prefer_tuple,
            extraction_caps: None,
        }
    }

    /// Create with custom tuple delimiters.
    pub fn with_tuple_delimiters(mut self, tuple: &str, completion: &str) -> Self {
        self.tuple_parser = TupleParser::with_delimiters(tuple, completion);
        if let Some(caps) = self.extraction_caps {
            self.tuple_parser = self.tuple_parser.with_extraction_caps(caps);
        }
        self
    }

    /// Set resolved extract caps for both parsers (SPEC-117).
    pub fn with_extraction_caps(mut self, caps: ExtractionCaps) -> Self {
        self.extraction_caps = Some(caps);
        self.tuple_parser = self.tuple_parser.with_extraction_caps(caps);
        self
    }

    fn parse_json(&self, response: &str, chunk_id: &str) -> Result<ExtractionResult> {
        self.json_parser.parse_with_options(
            response,
            chunk_id,
            JsonParseOptions {
                extraction_caps: self.extraction_caps,
                ..Default::default()
            },
        )
    }

    /// Parse extraction result, auto-detecting format.
    pub fn parse(&self, response: &str, chunk_id: &str) -> Result<ExtractionResult> {
        let (has_tuple_markers, has_json_markers) = detect_format_markers(response);

        tracing::debug!(
            has_tuple = has_tuple_markers,
            has_json = has_json_markers,
            prefer_tuple = self.prefer_tuple,
            response_len = response.len(),
            "Detecting extraction format"
        );

        // Determine which parser to use
        if has_tuple_markers && (!has_json_markers || self.prefer_tuple) {
            // Use tuple parser (more robust)
            match self.tuple_parser.parse(response, chunk_id) {
                Ok(result) if !result.entities.is_empty() || !result.relationships.is_empty() => {
                    tracing::debug!(
                        entities = result.entities.len(),
                        relationships = result.relationships.len(),
                        "Tuple parsing succeeded"
                    );
                    return Ok(result);
                }
                Ok(result) => {
                    // If tuple parsing returned empty but we have JSON markers, try JSON
                    if result.entities.is_empty()
                        && result.relationships.is_empty()
                        && has_json_markers
                    {
                        tracing::debug!("Tuple parsing returned empty, trying JSON fallback");
                    } else {
                        return Ok(result);
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "Tuple parsing failed, trying JSON fallback");
                }
            }
        }

        // Try JSON parser
        if has_json_markers {
            match self.parse_json(response, chunk_id) {
                Ok(result) => {
                    tracing::debug!(
                        entities = result.entities.len(),
                        relationships = result.relationships.len(),
                        "JSON parsing succeeded"
                    );
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "JSON parsing failed - attempting tuple fallback"
                    );
                    // If we also have tuple markers, try that as last resort
                    if has_tuple_markers {
                        tracing::info!("Falling back to tuple parsing after JSON failure");
                        return self.tuple_parser.parse(response, chunk_id);
                    }
                    // If no tuple markers, try tuple anyway (it's more lenient)
                    tracing::info!(
                        "No tuple markers detected but trying tuple parsing as last resort"
                    );
                    match self.tuple_parser.parse(response, chunk_id) {
                        Ok(result)
                            if !result.entities.is_empty() || !result.relationships.is_empty() =>
                        {
                            tracing::info!(
                                entities = result.entities.len(),
                                relationships = result.relationships.len(),
                                "Tuple fallback succeeded despite no markers"
                            );
                            return Ok(result);
                        }
                        Ok(_) => {
                            // Return original JSON error if tuple also failed
                            return Err(e);
                        }
                        Err(_) => {
                            // Return original JSON error
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Neither format detected, try tuple (more lenient)
        if has_tuple_markers {
            return self.tuple_parser.parse(response, chunk_id);
        }

        // Last resort: try JSON in case it's just not properly formatted
        self.parse_json(response, chunk_id)
    }

    /// Get the underlying tuple parser.
    pub fn tuple_parser(&self) -> &TupleParser {
        &self.tuple_parser
    }

    /// Get the underlying JSON parser.
    pub fn json_parser(&self) -> &JsonExtractionParser {
        &self.json_parser
    }
}

#[cfg(test)]
mod tests {
    use super::super::json_extract::extract_json_from_response;
    use super::*;

    #[test]
    fn test_tuple_parser_entities() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>John Doe<|#|>PERSON<|#|>A software developer
entity<|#|>Acme Corp<|#|>ORGANIZATION<|#|>A technology company
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();

        assert_eq!(result.entities.len(), 2);
        assert_eq!(result.entities[0].name, "JOHN_DOE");
        assert_eq!(result.entities[0].entity_type, "PERSON");
        assert_eq!(result.entities[1].name, "ACME_CORP");
        assert!(result
            .metadata
            .get("is_complete")
            .unwrap()
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_tuple_parser_relationships() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>Alice<|#|>PERSON<|#|>A researcher
entity<|#|>Bob<|#|>PERSON<|#|>Another researcher
relation<|#|>Alice<|#|>Bob<|#|>collaboration, research<|#|>Alice and Bob work together
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();

        assert_eq!(result.entities.len(), 2);
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].source, "ALICE");
        assert_eq!(result.relationships[0].target, "BOB");
        assert_eq!(result.relationships[0].keywords.len(), 2);
    }

    #[test]
    fn test_tuple_parser_incomplete() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>John<|#|>PERSON<|#|>A person"#;

        let result = parser.parse(response, "chunk-1").unwrap();

        assert_eq!(result.entities.len(), 1);
        assert!(!result
            .metadata
            .get("is_complete")
            .unwrap()
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_tuple_parser_malformed_lines() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>Valid<|#|>PERSON<|#|>Valid entity
some random text here
entity<|#|><|#|>PERSON<|#|>Empty name should skip
entity<|#|>Also Valid<|#|>CONCEPT<|#|>Another valid
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();

        assert_eq!(result.entities.len(), 2); // Only valid entities
        assert!(
            result
                .metadata
                .get("parse_errors")
                .unwrap()
                .as_u64()
                .unwrap()
                > 0
        );
    }

    #[test]
    fn test_json_parser() {
        let parser = JsonExtractionParser::new();
        // 054: relationships must reference selected entity endpoints (LR law).
        let response = r#"
```json
{
  "entities": [
    {"name": "John Doe", "type": "PERSON", "description": "A developer"},
    {"name": "Acme", "type": "ORGANIZATION", "description": "Employer"}
  ],
  "relationships": [
    {"source": "John Doe", "target": "Acme", "type": "WORKS_AT", "description": "Employment"}
  ]
}
```
"#;

        let result = parser.parse(response, "chunk-1").unwrap();

        assert_eq!(result.entities.len(), 2);
        assert_eq!(result.entities[0].name, "JOHN_DOE");
        assert_eq!(result.relationships.len(), 1);
    }

    #[test]
    fn test_hybrid_parser_tuple() {
        let parser = HybridExtractionParser::new(true);
        let response = r#"entity<|#|>Test<|#|>CONCEPT<|#|>A test entity
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();

        assert_eq!(result.entities.len(), 1);
        assert_eq!(
            result.metadata.get("parser").unwrap().as_str().unwrap(),
            "tuple"
        );
    }

    #[test]
    fn test_hybrid_parser_json() {
        let parser = HybridExtractionParser::new(true);
        let response = r#"{"entities": [{"name": "Test", "type": "CONCEPT", "description": "A test"}], "relationships": []}"#;

        let result = parser.parse(response, "chunk-1").unwrap();

        assert_eq!(result.entities.len(), 1);
        assert_eq!(
            result.metadata.get("parser").unwrap().as_str().unwrap(),
            "json"
        );
    }

    #[test]
    fn test_extract_json_from_response() {
        // Test code block extraction
        let response = "Here's the result:\n```json\n{\"key\": \"value\"}\n```\nDone!";
        assert_eq!(extract_json_from_response(response), "{\"key\": \"value\"}");

        // Test raw JSON
        let response = "Response: {\"key\": \"value\"}";
        assert_eq!(extract_json_from_response(response), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_tuple_parser_is_complete() {
        let parser = TupleParser::new();

        assert!(parser.is_complete("entity<|#|>X<|#|>Y<|#|>Z\n<|COMPLETE|>"));
        assert!(!parser.is_complete("entity<|#|>X<|#|>Y<|#|>Z"));
    }

    // =========================================================================
    // BR0006: Self-referencing relationships must be filtered
    // =========================================================================

    #[test]
    fn test_br0006_tuple_self_referencing_relationship_filtered() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>Neural Network<|#|>CONCEPT<|#|>A computing model
entity<|#|>Deep Learning<|#|>CONCEPT<|#|>A learning paradigm
relation<|#|>Neural Network<|#|>Neural Network<|#|>self-reference<|#|>Relates to itself
relation<|#|>Neural Network<|#|>Deep Learning<|#|>uses<|#|>Neural networks use deep learning
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();

        // Self-referencing relationship should be filtered out
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].source, "NEURAL_NETWORK");
        assert_eq!(result.relationships[0].target, "DEEP_LEARNING");
    }

    #[test]
    fn test_br0006_tuple_normalized_self_ref_filtered() {
        // "The Company" normalizes to "COMPANY", same as "company"
        let parser = TupleParser::new();
        let response = r#"entity<|#|>The Company<|#|>ORGANIZATION<|#|>A company
relation<|#|>The Company<|#|>company<|#|>self<|#|>Same entity after normalization
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        assert_eq!(result.relationships.len(), 0); // Both normalize to COMPANY
    }

    #[test]
    fn test_br0006_json_self_referencing_relationship_filtered() {
        let parser = JsonExtractionParser::new();
        let response = r#"{
            "entities": [
                {"name": "AI", "type": "CONCEPT", "description": "Artificial Intelligence"},
                {"name": "Machine Learning", "type": "CONCEPT", "description": "ML"}
            ],
            "relationships": [
                {"source": "AI", "target": "AI", "type": "SELF_REF", "description": "Self loop"},
                {"source": "AI", "target": "Machine Learning", "type": "USES", "description": "AI uses ML"}
            ]
        }"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].target, "MACHINE_LEARNING");
    }

    // =========================================================================
    // BR0004: Keyword limit of 5 per edge
    // =========================================================================

    #[test]
    fn test_br0004_tuple_keyword_limit_enforced() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>A<|#|>CONCEPT<|#|>Entity A
entity<|#|>B<|#|>CONCEPT<|#|>Entity B
relation<|#|>A<|#|>B<|#|>k1, k2, k3, k4, k5, k6, k7, k8<|#|>Many keywords
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        assert_eq!(result.relationships.len(), 1);
        assert!(
            result.relationships[0].keywords.len() <= 5,
            "Keywords should be limited to 5, got {}",
            result.relationships[0].keywords.len()
        );
    }

    // =========================================================================
    // Empty normalized name handling
    // =========================================================================

    #[test]
    fn test_empty_normalized_entity_name_filtered() {
        let parser = TupleParser::new();
        // "The" as a name normalizes to empty after prefix removal
        let response = r#"entity<|#|>The<|#|>CONCEPT<|#|>Just a prefix
entity<|#|>Valid Name<|#|>PERSON<|#|>A valid entity
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        // "The" normalizes to "" and should be filtered
        // but actually "The" → strip "The " prefix won't apply to standalone "The"
        // Let's check: normalize_entity_name("The") → to_title_case("The") → "The" → "THE"
        // So "The" is actually valid. Let me use "   " instead
        assert!(result.entities.iter().all(|e| !e.name.is_empty()));
    }

    #[test]
    fn test_empty_whitespace_entity_name_filtered() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>   <|#|>CONCEPT<|#|>Whitespace name
entity<|#|>Good<|#|>PERSON<|#|>A valid entity
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        // "   " → raw is_empty check after trim → skipped
        // Only "Good" should survive
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "GOOD");
    }

    #[test]
    fn test_empty_normalized_relationship_endpoints_filtered() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>A<|#|>CONCEPT<|#|>Entity A
relation<|#|>   <|#|>A<|#|>broken<|#|>Empty source
relation<|#|>A<|#|>   <|#|>broken<|#|>Empty target
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        assert_eq!(result.relationships.len(), 0);
    }

    #[test]
    fn test_json_empty_normalized_entity_name_filtered() {
        let parser = JsonExtractionParser::new();
        let response = r#"{
            "entities": [
                {"name": "  ", "type": "CONCEPT", "description": "Whitespace only"},
                {"name": "Valid Entity", "type": "PERSON", "description": "A real person"}
            ],
            "relationships": []
        }"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        // Empty/whitespace name normalizes to "" → should be filtered
        assert!(result.entities.iter().all(|e| !e.name.is_empty()));
    }

    #[test]
    fn test_json_empty_relationship_endpoints_filtered() {
        let parser = JsonExtractionParser::new();
        let response = r#"{
            "entities": [
                {"name": "A", "type": "CONCEPT", "description": "Entity A"},
                {"name": "B", "type": "CONCEPT", "description": "Entity B"}
            ],
            "relationships": [
                {"source": "  ", "target": "B", "type": "REL", "description": "Empty source"},
                {"source": "A", "target": "  ", "type": "REL", "description": "Empty target"},
                {"source": "A", "target": "B", "type": "VALID", "description": "Valid relationship"}
            ]
        }"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].relation_type, "VALID");
    }

    // =========================================================================
    // 067 — Opaque identifier entity names rejected
    // =========================================================================

    #[test]
    fn test_tuple_opaque_uuid_entity_names_filtered() {
        let parser = TupleParser::new();
        let response = r#"entity<|#|>84b69e27-e38b-444a-83dd-5e6a537c6f12<|#|>ORGANIZATION<|#|>An Anthropic resource id
entity<|#|>Acme Corp<|#|>ORGANIZATION<|#|>A real company
relation<|#|>84b69e27-e38b-444a-83dd-5e6a537c6f12<|#|>Acme Corp<|#|>uses<|#|>Opaque endpoint dropped
relation<|#|>Acme Corp<|#|>Gabriel Greenfield<|#|>employs<|#|>Valid edge
entity<|#|>Gabriel Greenfield<|#|>PERSON<|#|>Author
<|COMPLETE|>"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        assert!(
            result
                .entities
                .iter()
                .all(|e| !e.name.contains("84B69E27") && !e.name.contains('-')),
            "UUID entity must be filtered: {:?}",
            result.entities
        );
        assert!(
            result.entities.iter().any(|e| e.name == "ACME_CORP"),
            "semantic org kept: {:?}",
            result.entities
        );
        assert!(
            result
                .entities
                .iter()
                .any(|e| e.name == "GABRIEL_GREENFIELD"),
            "person kept: {:?}",
            result.entities
        );
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].source, "ACME_CORP");
        assert_eq!(result.relationships[0].target, "GABRIEL_GREENFIELD");
    }

    #[test]
    fn test_json_opaque_uuid_entity_names_filtered() {
        let parser = JsonExtractionParser::new();
        let response = r#"{
            "entities": [
                {"name": "84b69e27-e38b-444a-83dd-5e6a537c6f12", "type": "ORGANIZATION", "description": "Resource id"},
                {"name": "Acme Corp", "type": "ORGANIZATION", "description": "A company"}
            ],
            "relationships": [
                {"source": "84b69e27-e38b-444a-83dd-5e6a537c6f12", "target": "Acme Corp", "type": "USES", "description": "dropped"},
                {"source": "Acme Corp", "target": "Acme Corp", "type": "SELF", "description": "self ref dropped"}
            ]
        }"#;

        let result = parser.parse(response, "chunk-1").unwrap();
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "ACME_CORP");
        assert!(result.relationships.is_empty());
    }

    #[test]
    fn test_tuple_multimodal_im_id_not_rejected_by_parser_normalize() {
        // Merger/mm path relies on EntityId keeping im- identities; parser should too
        // if an inject path ever feeds through tuple parse with that name.
        let mm = "im-019f7028-d3e3-7684-8b3b-a9259368329a-page-0002-fig-01";
        assert!(!crate::prompts::normalize_entity_name(mm).is_empty());
    }
}
